use config::FileFormat;
use log::{info, trace, warn};
use tokio::{sync::mpsc::UnboundedSender, time::Instant};

use crate::prelude::*;
use std::{
    env, fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

#[derive(getset::Getters)]
pub struct ConfigFileManager {
    #[getset(get = "pub")]
    config_file_path: PathBuf,

    message_sender: UnboundedSender<Message>,

    last_metadata: (Option<u64>, Option<SystemTime>),
    last_poll: Instant,
    poll_interval: Duration,
}

impl ConfigFileManager {
    pub fn build(cli_args: &CliArgs, message_sender: UnboundedSender<Message>) -> Self {
        let mut manager = Self {
            config_file_path: Self::resolve_eilmeldung_config_dir(cli_args).join(CONFIG_FILE),
            last_metadata: (None, None),
            last_poll: Instant::now(),
            poll_interval: Duration::from_secs(1),
            message_sender,
        };
        manager.update_metadata();
        manager
    }

    fn try_path(path: &Path) -> Option<PathBuf> {
        let mut config_file_path = PathBuf::from(path);
        config_file_path.push(CONFIG_FILE);

        if !config_file_path.try_exists().unwrap_or(false) {
            return None;
        }
        Some(PathBuf::from(path))
    }

    fn extend_eilmeldung(prefix: Option<&str>, path: &str) -> PathBuf {
        let mut path_buf = PathBuf::from(path);

        if let Some(prefix) = prefix {
            path_buf.push(prefix);
        };

        path_buf.push("eilmeldung");
        path_buf
    }

    fn resolve_eilmeldung_config_dir(cli_args: &CliArgs) -> PathBuf {
        // CLI has priority
        if let Some(cli_config_path) = cli_args.config_dir() {
            return PathBuf::from(cli_config_path);
        };

        // first try XDG_CONFIG_HOME
        env::var("XDG_CONFIG_HOME")
            .ok()
            .and_then(|path| Self::try_path(&Self::extend_eilmeldung(None, &path)))
            // or $HOME/.config/eilmeldung
            .or_else(|| {
                env::var("HOME").ok().and_then(|home_path| {
                    Self::try_path(&Self::extend_eilmeldung(Some(".config"), &home_path))
                })
            })
            // or OS-dependent path
            .or_else(|| Self::try_path(PROJECT_DIRS.config_dir()))
            // if none worked, revert to "official" one
            .unwrap_or(PathBuf::from(PROJECT_DIRS.config_dir()))
    }

    pub fn load_config(&mut self) -> color_eyre::Result<Config> {
        info!("Trying to load config from {:?}", self.config_file_path);
        self.get_metadata();

        if !self.config_file_path.exists() {
            info!("No config file found, using default config");
            return Ok(Default::default());
        }

        let mut config = match config::Config::builder()
            .add_source(config::File::new(
                &self.config_file_path.to_string_lossy(),
                FileFormat::Toml,
            ))
            .build()
        {
            Ok(config) => config.try_deserialize::<Config>()?,
            Err(err) => {
                warn!("unable to read config file: {err}");
                return Err(color_eyre::eyre::eyre!(err));
            }
        };

        config.validate()?;

        Ok(config)
    }

    fn get_metadata(&self) -> (Option<u64>, Option<SystemTime>) {
        let Ok(metadata) = fs::metadata(&self.config_file_path) else {
            return (None, None);
        };

        (Some(metadata.len()), metadata.modified().ok())
    }

    fn update_metadata(&mut self) -> bool {
        trace!("checking for change");
        let current_metadata = self.get_metadata();
        self.last_poll = Instant::now();

        if current_metadata != self.last_metadata {
            self.last_metadata = current_metadata;
            return true;
        }

        false
    }
}

impl MessageReceiver for ConfigFileManager {
    async fn process_message(&mut self, message: &Message) -> color_eyre::Result<()> {
        if !matches!(message, Message::Event(Event::Tick)) {
            return Ok(());
        }

        if Instant::now().duration_since(self.last_poll) > self.poll_interval
            && self.update_metadata()
        {
            info!("configuration file has changed");
            self.message_sender
                .send(Message::Event(Event::ConfigFileChanged))?;
        }

        Ok(())
    }
}
