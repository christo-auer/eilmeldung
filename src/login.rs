use std::{cmp::Ordering, fmt::Display};

use crate::prelude::*;

use inquire::{Confirm, Password, Select, Text, min_length, validator::Validation};
use log::info;
use news_flash::{
    NewsFlash,
    models::{
        BasicAuth, DirectLogin, DirectLoginGUI, LoginData, LoginGUI, OAuthData, OAuthLoginGUI,
        PasswordLogin, PluginInfo, ServicePrice, ServiceType, TokenLogin, Url,
    },
};
use reqwest::Client;
use termimad::MadSkin;

const LOGIN_TYPE_PASSWORD: &str = "Username/Password";
const LOGIN_TYPE_TOKEN: &str = "Token";

pub struct LoginSetup {
    skin: MadSkin,
}

struct PluginInfoDisplayWrapper {
    pub plugin_info: PluginInfo,
}

impl Display for PluginInfoDisplayWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.plugin_info.name,)
    }
}

impl LoginSetup {
    pub fn new() -> Self {
        let mut s = Self {
            skin: MadSkin::default(),
        };
        s.skin
            .bold
            .set_fg(termimad::crossterm::style::Color::Magenta);
        s
    }

    fn print_summary(&self, plugin_info: &PluginInfo, login_data: &LoginData) {
        let summary_basic_auth = |basic_auth: &Option<BasicAuth>| -> Vec<String> {
            match basic_auth {
                Some(basic_auth) => {
                    let user = format!("enabled: user *{}*", &basic_auth.user);
                    vec!["**Basic Auth**".into(), user]
                }
                Option::None => vec!["**Basic Auth**".into(), "disabled".into()],
            }
        };

        use LoginData::*;
        match login_data {
            None(_) => {
                self.print_table(
                    Option::None,
                    "|-|",
                    &vec![vec!["No Login Required for this Provider"]],
                );
            }
            Direct(DirectLogin::Password(password_login)) => {
                self.print_table(
                    Option::None,
                    "|-|-|",
                    &vec![
                        vec!["**Provider**", &plugin_info.name],
                        vec!["**URL**", password_login.url.as_deref().unwrap_or("none")],
                        vec!["**User**", &password_login.user],
                        vec!["**Password**", "************"],
                        summary_basic_auth(&password_login.basic_auth)
                            .iter()
                            .map(String::as_str)
                            .collect(),
                    ],
                );
                // self.print_basic_auth(&password_login.basic_auth);
            }
            Direct(DirectLogin::Token(token_login)) => {
                self.print_table(
                    Option::None,
                    "|-|-|",
                    &vec![
                        vec!["**Provider**", &plugin_info.name],
                        vec!["**URL**", token_login.url.as_deref().unwrap_or("none")],
                        vec!["**Token**", &token_login.token],
                        summary_basic_auth(&token_login.basic_auth)
                            .iter()
                            .map(String::as_str)
                            .collect(),
                    ],
                );
            }

            OAuth(oauth_data) => {
                println!("  URL:      {}", oauth_data.url);

                println!("  Custom API Secret:");
                match oauth_data.custom_api_secret.as_ref() {
                    Option::None => println!("    No Custom API Secret"),
                    Some(api_secret) => {
                        println!("    Client ID:     {}", api_secret.client_id);
                        println!(
                            "    Client Secret: {}",
                            "*".repeat(api_secret.client_secret.len())
                        );
                    }
                }
            }
        }
    }

    fn to_line(&self, entries: &[impl Display], formatter: impl Fn(String) -> String) -> String {
        format!(
            "|{}\n",
            entries
                .iter()
                .map(|entry| formatter(entry.to_string()))
                .collect::<Vec<String>>()
                .join("|")
        )
    }

    fn print_table(
        &self,
        header: Option<&Vec<&str>>,
        alignment: &str,
        rows: &Vec<Vec<impl Display>>,
    ) {
        let mut table_str: String = String::new();

        if let Some(header) = header {
            table_str.push_str(&format!("{alignment}\n"));
            table_str.push_str(&self.to_line(header, |entry| format!("**{entry}**")));
        }

        table_str.push_str(&format!("{alignment}\n"));
        for row in rows {
            table_str.push_str(&self.to_line(row, |entry| entry.to_string()));
        }
        table_str.push_str("|-\n");

        self.skin.print_text(&table_str);
    }

    fn inquire_plugin_info(
        &self,
        preselect: &Option<PluginInfo>,
    ) -> color_eyre::Result<PluginInfo> {
        // let news_flash = self.news_flash_utils.news_flash_lock.read().await;
        let price_to_string = |price: &ServicePrice| {
            use ServicePrice::*;
            match price {
                Paid => "paid",
                PaidPremimum => "paid for premium",
                Free => "free",
            }
        };

        let type_to_string = |service_type: &ServiceType| {
            use ServiceType::*;
            match service_type {
                Local => "local",
                Remote { self_hosted } if *self_hosted => "self-hosted",
                Remote { self_hosted } if !*self_hosted => "cloud",
                _ => unreachable!(),
            }
        };

        let mut plugin_infos = NewsFlash::list_backends()
            .into_values()
            .map(|plugin_info| PluginInfoDisplayWrapper { plugin_info })
            .collect::<Vec<PluginInfoDisplayWrapper>>();

        plugin_infos.sort_by(|pi_a, pi_b| {
            if pi_a.plugin_info.service_type == ServiceType::Local
                && pi_b.plugin_info.service_type != ServiceType::Local
            {
                return Ordering::Less;
            } else if pi_b.plugin_info.service_type == ServiceType::Local
                && pi_a.plugin_info.service_type != ServiceType::Local
            {
                return Ordering::Greater;
            };
            pi_a.plugin_info.name.cmp(&pi_b.plugin_info.name)
        });

        // print the providers with infos
        let header = vec!["Provider", "󰖟 Website", " Price", " Hosting"];
        let rows = plugin_infos
            .iter()
            .map(|wrapper| {
                let pi = &wrapper.plugin_info;
                vec![
                    format!("*{}*", &pi.name),
                    pi.website
                        .as_ref()
                        .map(Url::to_string)
                        .unwrap_or("n/a".into()),
                    price_to_string(&pi.service_price).into(),
                    type_to_string(&pi.service_type).into(),
                ]
            })
            .collect::<Vec<Vec<String>>>();

        self.print_table(Some(&header), "|:-|:-|:-|:-", &rows);

        let preselect_index = match preselect {
            None => 0,
            Some(preselect) => plugin_infos
                .iter()
                .position(|plugin_info_wrapper| plugin_info_wrapper.plugin_info.id == preselect.id)
                .unwrap(),
        };

        Select::new("Select a provider", plugin_infos)
            .with_vim_mode(true)
            .with_starting_cursor(preselect_index)
            .without_filtering()
            .prompt()
            .map(|wrapper| wrapper.plugin_info)
            .map_err(|err| color_eyre::eyre::eyre!(err))
    }

    fn inquire_oauth_login(
        &self,
        _plugin_info: &PluginInfo,
        _oauth_login_gui: &OAuthLoginGUI,
        _preset_direct_login_data: &Option<OAuthData>,
    ) -> color_eyre::Result<LoginData> {
        // if oauth_login_gui.custom_api_secret {
        //     self.skin
        //         .print_text(r#"For this provider you need an API Secret"#);
        //     self.skin.print_inline(
        //         &oauth_login_gui
        //             .custom_api_secret_url
        //             .as_ref()
        //             .map(|url| url.to_string())
        //             .unwrap_or("no url".into()),
        //     );
        // }

        Err(color_eyre::eyre::eyre!(
            "eilmeldung does not yet support OAuth. You can raise an issue of you need it."
        ))
    }

    fn inquire_direct_login(
        &self,
        plugin_info: &PluginInfo,
        direct_login_gui: &DirectLoginGUI,
        preset_direct_login_data: &Option<DirectLogin>,
    ) -> color_eyre::Result<LoginData> {
        let direct_login_type = if direct_login_gui.support_token_login {
            Select::new(
                "How do you want to login?",
                vec![LOGIN_TYPE_PASSWORD, LOGIN_TYPE_TOKEN],
            )
            .with_vim_mode(true)
            .with_starting_cursor(match preset_direct_login_data {
                Some(DirectLogin::Password(_)) => 0,
                Some(DirectLogin::Token(_)) => 1,
                _ => 0,
            })
            .without_filtering()
            .prompt()?
        } else {
            LOGIN_TYPE_PASSWORD
        };

        match direct_login_type {
            LOGIN_TYPE_PASSWORD => {
                let preset_password_login =
                    preset_direct_login_data
                        .as_ref()
                        .and_then(|login_data| match login_data {
                            DirectLogin::Password(password_login) => Some(password_login.clone()),
                            _ => None,
                        });

                Ok(LoginData::Direct(DirectLogin::Password(
                    self.inquire_password_login(
                        plugin_info,
                        direct_login_gui,
                        &preset_password_login,
                    )?,
                )))
            }
            LOGIN_TYPE_TOKEN => {
                let preset_token_login =
                    preset_direct_login_data
                        .as_ref()
                        .and_then(|login_data| match login_data {
                            DirectLogin::Token(token_login) => Some(token_login.clone()),
                            _ => None,
                        });
                Ok(LoginData::Direct(DirectLogin::Token(
                    self.inquire_token_login(plugin_info, direct_login_gui, &preset_token_login)?,
                )))
            }
            _ => unreachable!(),
        }
    }

    fn inquire_username(&self, prompt: &str, user: &str) -> color_eyre::Result<String> {
        Ok(Text::new(prompt)
            .with_initial_value(user)
            .with_validator(min_length!(1, "minimum length of one character needed"))
            .with_placeholder(
                "identification with which you login, e.g., username or email address",
            )
            .prompt()?)
    }

    fn inquire_password(&self, prompt: &str) -> color_eyre::Result<String> {
        Ok(Password::new(prompt)
            .with_validator(min_length!(1, "minimum length of one character needed"))
            .with_display_mode(inquire::PasswordDisplayMode::Masked)
            .prompt()?)
    }

    fn inquire_url(&self, prompt: &str, url: &Option<String>) -> color_eyre::Result<String> {
        // let url_validator = |url_str: &str| reqwest::Url::try_from(url_str);
        //

        Ok(inquire::Text::new(prompt)
            .with_initial_value(url.as_deref().unwrap_or(""))
            .with_validator(|input: &str| match reqwest::Url::try_from(input) {
                Ok(_) => Ok(Validation::Valid),
                Err(_) => Ok(Validation::Invalid("invalid server URL".into())),
            })
            .with_placeholder(
                "URL to connect to, e.g., https://10.0.0.1:1234, https://feeds.service.com, etc.",
            )
            .prompt()?)
    }

    fn inquire_basic_auth(
        &self,
        basic_auth: &Option<BasicAuth>,
    ) -> color_eyre::Result<Option<BasicAuth>> {
        let setup_basic_auth = Confirm::new("Do you want to setup Basic HTTP Authentication for this provider? If unsure, say no")
            .with_default(false)
            .with_help_message("This is usually for self-hosted services for which the HTTP server is additionally secured by HTTP-based authentication")
            .prompt()?;

        if !setup_basic_auth {
            return Ok(None);
        }

        let user = self.inquire_username(
            "Username for HTTP authentication",
            basic_auth.as_ref().map(|ba| ba.user.as_str()).unwrap_or(""),
        )?;

        let password = Password::new("Password for HTTP authentication (optional)")
            .with_display_mode(inquire::PasswordDisplayMode::Masked)
            .prompt_skippable()?;

        Ok(Some(BasicAuth { user, password }))
    }

    fn inquire_token_login(
        &self,
        plugin_info: &PluginInfo,
        direct_login_gui: &DirectLoginGUI,
        preset_token_login: &Option<TokenLogin>,
    ) -> color_eyre::Result<TokenLogin> {
        let mut token_login = preset_token_login.clone().unwrap_or(TokenLogin {
            id: plugin_info.id.clone(),
            url: None,
            token: "".into(),
            basic_auth: None,
        });

        if direct_login_gui.url {
            token_login.url = self.inquire_url("Server URL: ", &token_login.url).ok();
        }

        token_login.token = inquire::Text::new("Token: ")
            .with_default(&token_login.token)
            .with_placeholder("login token by the provider")
            .prompt()?;

        if direct_login_gui.http_auth {
            token_login.basic_auth = self.inquire_basic_auth(&token_login.basic_auth)?;
        }

        Ok(token_login)
    }

    fn inquire_password_login(
        &self,
        plugin_info: &PluginInfo,
        direct_login_gui: &DirectLoginGUI,
        preset_password_login: &Option<PasswordLogin>,
    ) -> color_eyre::Result<PasswordLogin> {
        let mut password_login = preset_password_login.clone().unwrap_or(PasswordLogin {
            id: plugin_info.id.clone(),
            url: None,
            user: "".into(),
            password: "".into(),
            basic_auth: None,
        });

        if direct_login_gui.url {
            password_login.url = self.inquire_url("Server URL: ", &password_login.url).ok();
        }

        password_login.user = self.inquire_username("Provider Username:", &password_login.user)?;
        password_login.password = self.inquire_password("Provider Password:")?;

        if direct_login_gui.http_auth {
            password_login.basic_auth = self.inquire_basic_auth(&password_login.basic_auth)?;
        }

        Ok(password_login)
    }

    pub async fn inquire_login_data(
        &self,
        preset_login_data: &Option<LoginData>,
    ) -> color_eyre::Result<LoginData> {
        let mut login_data: Option<LoginData> = preset_login_data.clone();

        let mut selected_plugin_info: Option<PluginInfo> = login_data
            .as_ref()
            .and_then(|login_data| NewsFlash::list_backends().remove(&login_data.id()));

        self.print_header()?;

        self.skin
            .print_inline("\n**Welcome** to **+++ eilmeldung +++**\n\n");
        self.skin.print_text(
            "In the following you can setup the provider you want to use. You can always terminate the setup process by pressing **Ctrl-C** and restart later.\n\n",
        );

        loop {
            selected_plugin_info = Some(self.inquire_plugin_info(&selected_plugin_info)?);

            let plugin_info: &PluginInfo = selected_plugin_info.as_ref().unwrap();

            use LoginGUI::*;
            login_data = Some(match &plugin_info.login_gui {
                Direct(direct_login_gui) => self.inquire_direct_login(
                    plugin_info,
                    direct_login_gui,
                    &login_data.as_ref().and_then(|login_data| match login_data {
                        LoginData::Direct(direct_login_data) => Some(direct_login_data.clone()),
                        _ => Option::None,
                    }),
                )?,
                OAuth(oath_login_gui) => self.inquire_oauth_login(
                    plugin_info,
                    oath_login_gui,
                    &login_data.as_ref().and_then(|login_data| match login_data {
                        LoginData::OAuth(oauth_login_data) => Some(oauth_login_data.clone()),
                        _ => Option::None,
                    }),
                )?,
                None => LoginData::None(plugin_info.id.clone()),
            });

            self.print_summary(plugin_info, login_data.as_ref().unwrap());

            let finished =
                Confirm::new("Are you satisfied with these settings? Select `n` to change them.")
                    .with_default(true)
                    .prompt()?;

            if finished {
                return Ok(login_data.unwrap());
            }
        }
    }

    pub async fn login_and_initial_sync(
        &self,
        news_flash: &NewsFlash,
        login_data: &LoginData,
        client: &Client,
    ) -> color_eyre::eyre::Result<bool> {
        info!("attemping to login with: {:?} ", login_data);
        termimad::print_inline("Attempting to login and synchronize...\n");

        let login_attempt = news_flash
            .login(login_data.clone(), client)
            .await
            .and(news_flash.initial_sync(client, Default::default()).await);

        match login_attempt {
            Err(login_error) => {
                println!(
                    "{}: {}\n",
                    termimad::inline("**Failed to login**"),
                    NewsFlashUtils::error_to_message(&login_error),
                );

                if inquire::Confirm::new("Do you want to try again?")
                    .with_default(true)
                    .prompt()
                    .map_err(|err| color_eyre::eyre::eyre!(err))?
                {
                    Ok(false)
                } else {
                    Err(color_eyre::eyre::eyre!(login_error))
                }
            }

            Ok(_) => {
                info!("login and initial sync successful");
                termimad::print_inline(
                    "Login and initial sync successful. **You are ready to go**!\n",
                );
                inquire::Text::new("Press enter to continue...").prompt_skippable()?;
                Ok(true)
            }
        }
    }

    fn print_header(&self) -> color_eyre::Result<()> {
        let (width, _) = ratatui::crossterm::terminal::size()?;

        if width > 91 {
            println!(
                r#"
   _     _     _         (_) |              | |   | |                      _     _     _   
 _| |_ _| |_ _| |_    ___ _| |_ __ ___   ___| | __| |_   _ _ __   __ _   _| |_ _| |_ _| |_ 
|_   _|_   _|_   _|  / _ \ | | '_ ` _ \ / _ \ |/ _` | | | | '_ \ / _` | |_   _|_   _|_   _|
  |_|   |_|   |_|   |  __/ | | | | | | |  __/ | (_| | |_| | | | | (_| |   |_|   |_|   |_|  
                     \___|_|_|_| |_| |_|\___|_|\__,_|\__,_|_| |_|\__, |                    
                                                                  __/ |                    
                                                                 |___/                     
            "#,
            );
        }

        Ok(())
    }
}
