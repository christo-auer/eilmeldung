use directories::*;
use once_cell::sync::Lazy;

pub const CONFIG_FILE: &str = "config.json";

pub static PROJECT_DIRS: Lazy<ProjectDirs> =
    Lazy::new(|| ProjectDirs::from("org", "christo-auer", "eilmeldung").unwrap());
