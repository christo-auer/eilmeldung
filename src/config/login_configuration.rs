use std::{
    process::{Command, Stdio},
    str::FromStr,
};

use crate::prelude::*;
use itertools::Itertools;
use news_flash::models::{
    ApiSecret, BasicAuth, DirectLogin, LoginData, OAuthData, PasswordLogin, PluginID, TokenLogin,
};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct LoginConfiguration {
    pub login_type: LoginType,
    pub provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<Secret>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<Secret>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth_client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth_client_secret: Option<Secret>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub basic_auth_user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub basic_auth_password: Option<Secret>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LoginType {
    NoLogin,
    DirectPassword,
    DirectToken,
    OAuth,
}

#[derive(Clone, Debug)]
pub enum Secret {
    Verbatim(String),
    Command(Vec<String>),
}

impl FromStr for Secret {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if s.trim().starts_with("cmd:") {
            let (_, command) = s
                .trim()
                .split_once(":")
                .ok_or(ConfigError::SecretParseError)?;
            Secret::Command(shell_words::split(command).map_err(|_| ConfigError::SecretParseError)?)
        } else {
            Secret::Verbatim(s.to_owned())
        })
    }
}

impl Secret {
    pub fn get_secret(&self) -> Result<String, ConfigError> {
        Ok(match self {
            Secret::Verbatim(secret) => secret.clone(),
            Secret::Command(args) => {
                let Some((cmd, args)) = args.split_first() else {
                    return Err(ConfigError::SecretCommandExecutionError(
                        "pass command is empty".to_owned(),
                    ));
                };
                let child = Command::new(cmd)
                    .stdin(Stdio::null())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .args(args)
                    .spawn()
                    .map_err(|err| ConfigError::SecretCommandExecutionError(err.to_string()))?;

                let output = child
                    .wait_with_output()
                    .map_err(|err| ConfigError::SecretCommandExecutionError(err.to_string()))?;

                if !output.status.success() {
                    return Err(ConfigError::SecretCommandExecutionError(
                        String::from_utf8(output.stderr).map_err(|_| {
                            ConfigError::SecretCommandExecutionError(
                                "cannot read stderr from password command".to_owned(),
                            )
                        })?,
                    ));
                }

                let pass = String::from_utf8(output.stdout).map_err(|_| {
                    ConfigError::SecretCommandExecutionError(
                        "cannot read stdin from password command".to_owned(),
                    )
                })?;

                // trim trailing news lines
                pass.trim_end_matches(['\r', '\n']).to_owned()
            }
        })
    }

    pub fn get_secret_option(secret: Option<&Self>) -> Result<Option<String>, ConfigError> {
        secret.map(Self::get_secret).transpose()
    }
}

impl<'de> serde::de::Deserialize<'de> for Secret {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Secret::from_str(&String::deserialize(deserializer)?)
            .map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

impl serde::ser::Serialize for Secret {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Secret::Verbatim(secret) => serializer.serialize_str(secret),
            Secret::Command(args) => {
                let cmd = args.first().unwrap_or(&"".to_owned()).to_owned();

                let args_str = args
                    .iter()
                    .skip(1)
                    .map(|arg| shell_words::quote(arg))
                    .join(" ");

                serializer.serialize_str(&format!("{cmd} {args_str}"))
            }
        }
    }
}

fn unwrap_or_config_error<T>(val: Option<&T>, err: &str) -> Result<T, ConfigError>
where
    T: Clone,
{
    Ok(val
        .ok_or(ConfigError::LoginConfigurationInvalid(err.to_owned()))?
        .to_owned())
}

impl LoginConfiguration {
    pub fn to_login_data(&self) -> Result<LoginData, ConfigError> {
        Ok(match self.login_type {
            LoginType::NoLogin => LoginData::None(PluginID::new(&self.provider)),
            LoginType::DirectPassword => self.to_direct_login()?,
            LoginType::DirectToken => self.to_direct_token()?,
            LoginType::OAuth => self.to_oauth()?,
        })
    }

    fn to_direct_login(&self) -> Result<LoginData, ConfigError> {
        let password = unwrap_or_config_error(
            Secret::get_secret_option(self.password.as_ref())?.as_ref(),
            "password needed",
        )?;

        Ok(LoginData::Direct(DirectLogin::Password(
            news_flash::models::PasswordLogin {
                id: PluginID::new(&self.provider),
                url: self.url.to_owned(),
                user: unwrap_or_config_error(self.user.as_ref(), "user name needed")?,
                password,
                basic_auth: self.to_basic_auth()?,
            },
        )))
    }

    fn to_direct_token(&self) -> Result<LoginData, ConfigError> {
        let token = unwrap_or_config_error(
            Secret::get_secret_option(self.token.as_ref())?.as_ref(),
            "token needed",
        )?;

        Ok(LoginData::Direct(DirectLogin::Token(
            news_flash::models::TokenLogin {
                id: PluginID::new(&self.provider),
                url: self.url.to_owned(),
                token,
                basic_auth: self.to_basic_auth()?,
            },
        )))
    }

    fn to_oauth(&self) -> Result<LoginData, ConfigError> {
        let api_secret = match (
            self.oauth_client_id.as_ref(),
            self.oauth_client_secret.as_ref(),
        ) {
            (None, None) => None,
            (Some(client_id), Some(client_secret)) => Some(ApiSecret {
                client_id: client_id.to_owned(),
                client_secret: client_secret.get_secret()?,
            }),

            _ => {
                return Err(ConfigError::LoginConfigurationInvalid(
                    "either both, oauth_client_id and oauth_client_secret, must be defined or none"
                        .to_owned(),
                ));
            }
        };

        Ok(LoginData::OAuth(OAuthData {
            id: PluginID::new(&self.provider),
            url: unwrap_or_config_error(self.url.as_ref(), "url needed")?,
            custom_api_secret: api_secret,
        }))
    }

    fn to_basic_auth(&self) -> Result<Option<BasicAuth>, ConfigError> {
        let password = Secret::get_secret_option(self.basic_auth_password.as_ref())?;

        Ok(self.basic_auth_user.as_ref().map(|user| BasicAuth {
            user: user.to_owned(),
            password,
        }))
    }

    fn from_none(plugin_id: PluginID) -> Self {
        Self {
            login_type: LoginType::NoLogin,
            provider: plugin_id.as_str().to_owned(),
            ..Default::default()
        }
    }

    fn from_direct_password(direct_login: &PasswordLogin) -> Self {
        Self {
            login_type: LoginType::DirectPassword,
            provider: direct_login.id.as_str().to_owned(),
            url: direct_login.url.to_owned(),
            user: Some(direct_login.user.to_owned()),
            password: Some(Secret::Verbatim(direct_login.password.to_owned())),
            ..Default::default()
        }
        .with_basic_auth(direct_login.basic_auth.as_ref())
    }

    fn with_basic_auth(self, basic_auth: Option<&BasicAuth>) -> Self {
        Self {
            basic_auth_user: basic_auth
                .map(|basic_auth| basic_auth.user.to_owned())
                .clone(),
            basic_auth_password: basic_auth
                .and_then(|basic_auth| basic_auth.password.to_owned())
                .map(Secret::Verbatim),
            ..self
        }
    }

    fn from_direct_token(token_login: &TokenLogin) -> Self {
        Self {
            login_type: LoginType::DirectToken,
            provider: token_login.id.as_str().to_owned(),
            url: token_login.url.to_owned(),
            token: Some(Secret::Verbatim(token_login.token.to_owned())),
            ..Self::default()
        }
        .with_basic_auth(token_login.basic_auth.as_ref())
    }

    fn from_oauth(oauth_data: &OAuthData) -> Self {
        Self {
            login_type: LoginType::OAuth,
            provider: oauth_data.id.as_str().to_owned(),
            url: Some(oauth_data.url.to_owned()),
            oauth_client_id: oauth_data
                .custom_api_secret
                .as_ref()
                .map(|api_secret| api_secret.client_id.to_owned()),
            oauth_client_secret: oauth_data
                .custom_api_secret
                .as_ref()
                .map(|api_secret| Secret::Verbatim(api_secret.client_secret.to_owned())),

            ..Self::default()
        }
    }

    fn redact(pass: Option<Secret>) -> Option<Secret> {
        pass.map(|_| Secret::Verbatim("*******".to_owned()))
    }

    fn redact_secrets(self) -> Self {
        Self {
            password: Self::redact(self.password),
            oauth_client_secret: Self::redact(self.oauth_client_secret),
            basic_auth_password: Self::redact(self.basic_auth_password),
            token: Self::redact(self.token),
            ..self
        }
    }

    pub fn as_toml(&self, show_secrets: bool) -> Result<String, ConfigError> {
        if show_secrets {
            toml::ser::to_string(&Self::redact_secrets(self.clone()))
                .map_err(|err| ConfigError::LoginConfigurationInvalid(err.to_string()))
        } else {
            toml::ser::to_string(self)
                .map_err(|err| ConfigError::LoginConfigurationInvalid(err.to_string()))
        }
    }
}

impl Default for LoginConfiguration {
    fn default() -> Self {
        Self {
            login_type: LoginType::NoLogin,
            provider: "local_rss".to_owned(),
            user: None,
            url: None,

            password: None,

            token: None,

            oauth_client_id: None,
            oauth_client_secret: None,

            basic_auth_user: None,
            basic_auth_password: None,
        }
    }
}

impl From<LoginData> for LoginConfiguration {
    fn from(login_data: LoginData) -> Self {
        match login_data {
            LoginData::None(plugin_id) => Self::from_none(plugin_id),
            LoginData::Direct(DirectLogin::Password(direct_password)) => {
                Self::from_direct_password(&direct_password)
            }
            LoginData::Direct(DirectLogin::Token(direct_token)) => {
                Self::from_direct_token(&direct_token)
            }
            LoginData::OAuth(oauth_data) => Self::from_oauth(&oauth_data),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use claims::assert_matches;
    use rstest::rstest;

    #[rstest]
    #[case("secret12345")]
    #[case("  secret12345")]
    #[case("secret12345  ")]
    #[case("")]
    fn test_secret_parsing_password(#[case] s: &str) {
        use Secret as P;
        let Ok(P::Verbatim(secret)) = Secret::from_str(s) else {
            panic!("expected Password");
        };

        assert_eq!(secret, s);
    }

    #[rstest]
    #[case("cmd:pass private/eilmeldung", vec!["pass", "private/eilmeldung"])]
    #[case("cmd:/home/user/pass.sh", vec!["/home/user/pass.sh"])]
    #[case(" cmd:   pass  ", vec!["pass"])]
    #[case("cmd:", vec![])]
    fn test_secret_parsing_command(#[case] s: &str, #[case] args: Vec<&str>) {
        use Secret as P;
        let Ok(P::Command(command)) = Secret::from_str(s) else {
            panic!("expected Command");
        };

        assert_eq!(command, args);
    }

    #[rstest]
    #[case("cmd:pass \"private/eilmeldung")]
    #[case("cmd:/home/user/pass.sh\'")]
    fn test_secret_parsing_command_fail(#[case] s: &str) {
        assert_matches!(Secret::from_str(s), Err(ConfigError::SecretParseError));
    }

    #[test]
    fn test_no_login() {
        let login_config = LoginConfiguration {
            login_type: LoginType::NoLogin,
            provider: "abc".to_owned(),
            user: None,
            url: None,
            password: None,
            token: None,

            oauth_client_id: None,
            oauth_client_secret: None,

            basic_auth_user: None,
            basic_auth_password: None,
        };

        let login_data = login_config.to_login_data();

        let Ok(LoginData::None(provider)) = login_data else {
            panic!("expected LoginData::None")
        };

        assert_eq!(provider.as_str(), "abc");
    }

    #[rstest]
    #[case("cmd:echo \"secret\"", "secret")]
    #[case("cmd:  echo \"secret\"", "secret")]
    #[case("cmd:echo 'secret'", "secret")]
    #[case("cmd:./test/outputpass.sh somearg", "secret")]
    #[case("cmd:./test/outputpass.sh somearg someotherarg", "secret")]
    #[case("cmd:./test/outputpass.sh somearg someotherarg", "secret")]
    fn test_secret_execute_command(#[case] cmd: &str, #[case] pass: &str) {
        let secret = Secret::from_str(cmd).unwrap();
        let pass_value = secret.get_secret();
        assert_matches!(pass_value, Ok(..));
        assert_eq!(pass_value.unwrap(), pass);
    }

    #[test]
    fn test_secret_execute_command_fail() {
        let secret = Secret::from_str("cmd:./test/outputpass.sh somerror").unwrap();
        let pass_value = secret.get_secret();
        assert_matches!(
            pass_value,
            Err(ConfigError::SecretCommandExecutionError(..))
        );
        let ConfigError::SecretCommandExecutionError(message) = pass_value.unwrap_err() else {
            panic!("expected SecretCommandExecutionError with error message");
        };

        assert_eq!(message, "this is expected\n");
    }

    #[test]
    fn test_direct_login() {
        let login_configuration = LoginConfiguration {
            login_type: LoginType::DirectPassword,
            provider: "abc".to_owned(),
            user: Some("username".to_owned()),
            url: Some("http://www.rssprovider.com/".to_owned()),
            password: Some(Secret::from_str("cmd:echo \"secret\"").unwrap()),
            token: None,

            oauth_client_id: None,
            oauth_client_secret: None,

            basic_auth_user: Some("username".to_owned()),
            basic_auth_password: Some(Secret::from_str("cmd:echo \"secret\"").unwrap()),
        };
        let login_data_res = login_configuration.to_login_data();

        let Ok(LoginData::Direct(DirectLogin::Password(direct_login_data))) = login_data_res else {
            panic!("expected direct login data");
        };

        assert_eq!(direct_login_data.user, "username");
        assert_eq!(direct_login_data.password, "secret");
        assert_eq!(
            direct_login_data.url.unwrap(),
            "http://www.rssprovider.com/"
        );

        let Some(BasicAuth { user, password }) = direct_login_data.basic_auth else {
            panic!("expected basic auth data");
        };

        assert_eq!(user, "username");
        assert_eq!(password.unwrap(), "secret");
    }

    #[test]
    fn test_oauth() {
        let login_configuration = LoginConfiguration {
            login_type: LoginType::OAuth,
            provider: "abc".to_owned(),
            user: None,
            url: Some("http://www.rssprovider.com/".to_owned()),
            password: None,
            token: None,

            oauth_client_id: Some("someclientid".to_owned()),
            oauth_client_secret: Some(Secret::from_str("cmd:echo \"secret\"").unwrap()),

            basic_auth_user: Some("username".to_owned()),
            basic_auth_password: Some(Secret::from_str("cmd:echo \"secret\"").unwrap()),
        };
        let login_data_res = login_configuration.to_login_data();

        assert_matches!(login_data_res, Ok(LoginData::OAuth(..)));

        let Ok(LoginData::OAuth(oauth_data)) = login_data_res else {
            panic!("expected oauth login data");
        };

        assert_eq!(
            oauth_data.custom_api_secret.as_ref().unwrap().client_id,
            "someclientid"
        );
        assert_eq!(
            oauth_data.custom_api_secret.as_ref().unwrap().client_secret,
            "secret"
        );
        assert_eq!(oauth_data.url, "http://www.rssprovider.com/");
    }

    #[rstest]
    #[rustfmt::skip]
    #[case(LoginType::DirectPassword, Some(None), None,       None,       None,       None, None, None,       None)]
    #[case(LoginType::DirectPassword, None,       None,       Some(None), None,       None, None, None,       None)]
    #[case(LoginType::DirectToken,    None,       None,       None,       Some(None), None, None, None,       None)]
    #[case(LoginType::OAuth,          None,       Some(None), None,       None,       None, None, None,       None)]
    #[case(LoginType::OAuth,          None,       None,       None,       None,       None, None, Some(None), None)]
    #[case(LoginType::OAuth,          None,       None,       None,       None,       None, None, None,       Some(None))]
    fn to_login_data_fails(
        #[case] login_type: LoginType,
        #[case] user: Option<Option<&str>>,
        #[case] url: Option<Option<&str>>,
        #[case] password: Option<Option<&str>>,
        #[case] token: Option<Option<&str>>,
        #[case] basic_auth_user: Option<Option<&str>>,
        #[case] basic_auth_password: Option<Option<&str>>,
        #[case] oauth_client_id: Option<Option<&str>>,
        #[case] oauth_client_secret: Option<Option<&str>>,
    ) {
        let login_configuration = LoginConfiguration {
            login_type,
            provider: "abc".to_owned(),
            user: user.unwrap_or(Some("username")).map(str::to_owned),
            url: url
                .unwrap_or(Some("http://www.rssprovider.com/"))
                .map(str::to_owned),
            password: password
                .unwrap_or(Some("cmd:echo \"secret1\""))
                .map(|cmd| Secret::from_str(cmd).unwrap()),
            token: token
                .unwrap_or(Some("cmd:echo \"secret2\""))
                .map(|cmd| Secret::from_str(cmd).unwrap()),

            oauth_client_id: oauth_client_id
                .unwrap_or(Some("someclientid"))
                .map(str::to_owned),

            oauth_client_secret: oauth_client_secret
                .unwrap_or(Some("cmd:echo \"secret3\""))
                .map(|cmd| Secret::from_str(cmd).unwrap()),

            basic_auth_user: basic_auth_user
                .unwrap_or(Some("username"))
                .map(str::to_owned),
            basic_auth_password: basic_auth_password
                .unwrap_or(Some("cmd:echo \"secret4\""))
                .map(|cmd| Secret::from_str(cmd).unwrap()),
        };
        let login_data_res = login_configuration.to_login_data();

        assert_matches!(
            login_data_res,
            Err(ConfigError::LoginConfigurationInvalid(_))
        );
    }
}
