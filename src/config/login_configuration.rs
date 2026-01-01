use std::{
    process::{Command, Stdio},
    str::FromStr,
};

use crate::prelude::*;
use news_flash::models::{ApiSecret, BasicAuth, DirectLogin, LoginData, OAuthData, PluginID};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct LoginConfiguration {
    pub login_type: LoginType,
    pub provider: String,
    pub user: Option<String>,
    pub url: Option<String>,

    pub password: Option<PassCommand>,

    pub token: Option<PassCommand>,

    pub oauth_client_id: Option<String>,
    pub oauth_client_secret: Option<PassCommand>,

    pub basic_auth_user: Option<String>,
    pub basic_auth_password: Option<PassCommand>,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub enum LoginType {
    NoLogin,
    DirectPassword,
    DirectToken,
    OAuth,
}

#[derive(Clone, Debug)]
pub enum PassCommand {
    Password(String),
    Command(Vec<String>),
}

impl FromStr for PassCommand {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if s.trim().starts_with("cmd:") {
            let (_, command) = s
                .trim()
                .split_once(":")
                .ok_or(ConfigError::PassCommandParseError)?;
            PassCommand::Command(
                shell_words::split(command).map_err(|_| ConfigError::PassCommandParseError)?,
            )
        } else {
            PassCommand::Password(s.to_owned())
        })
    }
}

impl PassCommand {
    pub fn get_pass(&self) -> Result<String, ConfigError> {
        Ok(match self {
            PassCommand::Password(pass) => pass.clone(),
            PassCommand::Command(args) => {
                let Some((cmd, args)) = args.split_first() else {
                    return Err(ConfigError::PassCommandCommandExecutionError(
                        "pass command is empty".to_owned(),
                    ));
                };
                let child = Command::new(cmd)
                    .stdin(Stdio::null())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .args(args)
                    .spawn()
                    .map_err(|err| {
                        ConfigError::PassCommandCommandExecutionError(err.to_string())
                    })?;

                let output = child.wait_with_output().map_err(|err| {
                    ConfigError::PassCommandCommandExecutionError(err.to_string())
                })?;

                if !output.status.success() {
                    return Err(ConfigError::PassCommandCommandExecutionError(
                        String::from_utf8(output.stderr).map_err(|_| {
                            ConfigError::PassCommandCommandExecutionError(
                                "cannot read stderr from password command".to_owned(),
                            )
                        })?,
                    ));
                }

                let pass = String::from_utf8(output.stdout).map_err(|_| {
                    ConfigError::PassCommandCommandExecutionError(
                        "cannot read stdin from password command".to_owned(),
                    )
                })?;

                // trim trailing news lines
                pass.trim_end_matches(['\r', '\n']).to_owned()
            }
        })
    }

    pub fn get_pass_option(pass_cmd: Option<&Self>) -> Result<Option<String>, ConfigError> {
        pass_cmd.map(Self::get_pass).transpose()
    }
}

impl<'de> serde::de::Deserialize<'de> for PassCommand {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        PassCommand::from_str(&String::deserialize(deserializer)?)
            .map_err(|e| serde::de::Error::custom(e.to_string()))
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
            PassCommand::get_pass_option(self.password.as_ref())?.as_ref(),
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
            PassCommand::get_pass_option(self.token.as_ref())?.as_ref(),
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
                client_secret: client_secret.get_pass()?,
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
        let password = PassCommand::get_pass_option(self.basic_auth_password.as_ref())?;

        Ok(self.basic_auth_user.as_ref().map(|user| BasicAuth {
            user: user.to_owned(),
            password,
        }))
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
    fn test_pass_command_parsing_password(#[case] s: &str) {
        use PassCommand as P;
        let Ok(P::Password(pass)) = PassCommand::from_str(s) else {
            panic!("expected Password");
        };

        assert_eq!(pass, s);
    }

    #[rstest]
    #[case("cmd:pass private/eilmeldung", vec!["pass", "private/eilmeldung"])]
    #[case("cmd:/home/user/pass.sh", vec!["/home/user/pass.sh"])]
    #[case(" cmd:   pass  ", vec!["pass"])]
    #[case("cmd:", vec![])]
    fn test_pass_command_parsing_command(#[case] s: &str, #[case] args: Vec<&str>) {
        use PassCommand as P;
        let Ok(P::Command(command)) = PassCommand::from_str(s) else {
            panic!("expected Command");
        };

        assert_eq!(command, args);
    }

    #[rstest]
    #[case("cmd:pass \"private/eilmeldung")]
    #[case("cmd:/home/user/pass.sh\'")]
    fn test_pass_command_parsing_command_fail(#[case] s: &str) {
        assert_matches!(
            PassCommand::from_str(s),
            Err(ConfigError::PassCommandParseError)
        );
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
    fn test_pass_command_execute_command(#[case] cmd: &str, #[case] pass: &str) {
        let pass_cmd = PassCommand::from_str(cmd).unwrap();
        let pass_value = pass_cmd.get_pass();
        assert_matches!(pass_value, Ok(..));
        assert_eq!(pass_value.unwrap(), pass);
    }

    #[test]
    fn test_pass_command_execute_command_fail() {
        let pass_cmd = PassCommand::from_str("cmd:./test/outputpass.sh somerror").unwrap();
        let pass_value = pass_cmd.get_pass();
        assert_matches!(
            pass_value,
            Err(ConfigError::PassCommandCommandExecutionError(..))
        );
        let ConfigError::PassCommandCommandExecutionError(message) = pass_value.unwrap_err() else {
            panic!("expected PassCommandCommandExecutionError with error message");
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
            password: Some(PassCommand::from_str("cmd:echo \"secret\"").unwrap()),
            token: None,

            oauth_client_id: None,
            oauth_client_secret: None,

            basic_auth_user: Some("username".to_owned()),
            basic_auth_password: Some(PassCommand::from_str("cmd:echo \"secret\"").unwrap()),
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
            oauth_client_secret: Some(PassCommand::from_str("cmd:echo \"secret\"").unwrap()),

            basic_auth_user: Some("username".to_owned()),
            basic_auth_password: Some(PassCommand::from_str("cmd:echo \"secret\"").unwrap()),
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
                .map(|cmd| PassCommand::from_str(cmd).unwrap()),
            token: token
                .unwrap_or(Some("cmd:echo \"secret2\""))
                .map(|cmd| PassCommand::from_str(cmd).unwrap()),

            oauth_client_id: oauth_client_id
                .unwrap_or(Some("someclientid"))
                .map(str::to_owned),

            oauth_client_secret: oauth_client_secret
                .unwrap_or(Some("cmd:echo \"secret3\""))
                .map(|cmd| PassCommand::from_str(cmd).unwrap()),

            basic_auth_user: basic_auth_user
                .unwrap_or(Some("username"))
                .map(str::to_owned),
            basic_auth_password: basic_auth_password
                .unwrap_or(Some("cmd:echo \"secret4\""))
                .map(|cmd| PassCommand::from_str(cmd).unwrap()),
        };
        let login_data_res = login_configuration.to_login_data();

        assert_matches!(
            login_data_res,
            Err(ConfigError::LoginConfigurationInvalid(_))
        );
    }
}
