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
    pub user: Option<Secret>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<Secret>,

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

            let (command, mut args) = prepare_command(command)
                .map_err(|report| ConfigError::SecretCommandParseError(report.to_string()))?;

            args.insert(0, command);

            Secret::Command(args)
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
        let user = unwrap_or_config_error(
            Secret::get_secret_option(self.user.as_ref())?.as_ref(),
            "user name needed",
        )?;

        Ok(LoginData::Direct(DirectLogin::Password(
            news_flash::models::PasswordLogin {
                id: PluginID::new(&self.provider),
                url: Secret::get_secret_option(self.url.as_ref())?,
                user,
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
                url: Secret::get_secret_option(self.url.as_ref())?,
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

        let url = unwrap_or_config_error(
            Secret::get_secret_option(self.url.as_ref())?.as_ref(),
            "url needed",
        )?;

        Ok(LoginData::OAuth(OAuthData {
            id: PluginID::new(&self.provider),
            url,
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
            url: direct_login
                .url
                .as_deref()
                .map(|u| Secret::Verbatim(u.to_owned())),
            user: Some(Secret::Verbatim(direct_login.user.to_owned())),
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
            url: token_login
                .url
                .as_deref()
                .map(|u| Secret::Verbatim(u.to_owned())),
            token: Some(Secret::Verbatim(token_login.token.to_owned())),
            ..Self::default()
        }
        .with_basic_auth(token_login.basic_auth.as_ref())
    }

    fn from_oauth(oauth_data: &OAuthData) -> Self {
        Self {
            login_type: LoginType::OAuth,
            provider: oauth_data.id.as_str().to_owned(),
            url: Some(Secret::Verbatim(oauth_data.url.to_owned())),
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
            url: Self::redact(self.url),
            user: Self::redact(self.user),
            password: Self::redact(self.password),
            oauth_client_secret: Self::redact(self.oauth_client_secret),
            basic_auth_password: Self::redact(self.basic_auth_password),
            token: Self::redact(self.token),
            ..self
        }
    }

    pub fn as_toml(&self, show_secrets: bool) -> Result<String, ConfigError> {
        if show_secrets {
            toml::ser::to_string(self)
                .map_err(|err| ConfigError::LoginConfigurationInvalid(err.to_string()))
        } else {
            toml::ser::to_string(&Self::redact_secrets(self.clone()))
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
    #[case("cmd:")]
    fn test_secret_parsing_command_fail(#[case] s: &str) {
        assert_matches!(
            Secret::from_str(s),
            Err(ConfigError::SecretCommandParseError(_))
        );
    }
}
