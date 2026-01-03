use clap::Parser;
use getset::Getters;
use log::LevelFilter;

#[derive(Parser, Debug, Getters)]
#[command(version, about)]
#[getset(get = "pub")]
pub struct CliArgs {
    /// Log file (must be writable)
    #[arg(long)]
    log_file: Option<String>,

    /// Log level (OFF, ERROR, WARN, INFO, DEBUG, TRACE)
    #[arg(long)]
    log_level: Option<LevelFilter>,

    /// Directory with config files (config.toml, etc.)
    #[arg(short, long)]
    config_dir: Option<String>,

    /// Directory with state files (database, etc.)
    #[arg(short, long)]
    state_dir: Option<String>,

    /// Print login data as TOML (for use as automatic login setup in config.toml)
    #[arg(long)]
    print_login_data: bool,

    /// Show secrets when printing login data
    #[arg(long)]
    show_secrets: bool,
}
