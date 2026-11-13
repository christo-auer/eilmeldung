use directories::*;
use once_cell::sync::Lazy;

pub const CONFIG_FILE: &str = "config.toml";

// pub const HTTP_CACHE_FILE: &str = "http-cache.redb";

pub static PROJECT_DIRS: Lazy<ProjectDirs> =
    Lazy::new(|| ProjectDirs::from("org", "christo-auer", "eilmeldung").unwrap());
