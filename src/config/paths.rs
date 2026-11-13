use std::sync::LazyLock;

use directories::*;

pub const CONFIG_FILE: &str = "config.toml";

pub static PROJECT_DIRS: LazyLock<ProjectDirs> =
    LazyLock::new(|| ProjectDirs::from("org", "christo-auer", "eilmeldung").unwrap());
