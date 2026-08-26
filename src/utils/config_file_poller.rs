use std::{fs, path::PathBuf, time::SystemTime};

#[derive(getset::Getters)]
pub struct ConfigFilePoller {
    #[getset(get = "pub")]
    config_file_path: PathBuf,
    last_modified: Option<SystemTime>,
}

impl ConfigFilePoller {
    pub fn new(config_file_path: PathBuf) -> Self {
        let mut poller = Self {
            config_file_path,
            last_modified: None,
        };

        poller.last_modified = Self::get_current_mtime(&poller);

        poller
    }

    fn get_current_mtime(&self) -> Option<SystemTime> {
        let Ok(metadata) = fs::metadata(&self.config_file_path) else {
            return None;
        };

        metadata.modified().ok()
    }

    pub fn config_file_modified(&mut self) -> bool {
        let current_modified = self.get_current_mtime();

        if current_modified != self.last_modified {
            self.last_modified = current_modified;
            return true;
        }

        false
    }
}
