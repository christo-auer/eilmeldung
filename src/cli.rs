use std::path::PathBuf;

use crate::prelude::*;
use clap::{Args, Parser};
use getset::Getters;
use log::LevelFilter;
use news_flash::NewsFlash;
use reqwest::Client;

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

    /// Directory with eilmeldung config file (config.toml)
    #[arg(short, long)]
    config_dir: Option<String>,

    /// Directory with news-flash config files (newsflash.json, authentication configuration)
    #[arg(long)]
    news_flash_config_dir: Option<String>,

    /// Directory with news-flash state files (database, etc.)
    #[arg(long)]
    news_flash_state_dir: Option<String>,

    /// Show secrets when printing login data
    #[arg(long)]
    show_secrets: bool,

    #[command(flatten)]
    action: CliAction,

    #[arg(long)]
    quiet: bool,
}

#[derive(Args, Debug, Getters)]
#[group(required = false, multiple = false)]
struct CliAction {
    /// Print login data as TOML (for use as automatic login setup in config.toml)
    #[arg(long)]
    print_login_data: bool,

    /// Sync all feeds, print sync stats and then exit
    #[arg(long)]
    sync: bool,

    /// Print current stats (unread, read, marked articles per feed) and then exit
    #[arg(long)]
    stats: bool,

    /// Export to OPML file (if provider supports OPML export)
    #[arg(long)]
    export_opml: Option<PathBuf>,

    /// Import OPML file (if provider supports OPML import)
    #[arg(long)]
    import_opml: Option<PathBuf>,

    /// Logout of RSS provider (NOTE: this will remove all local data! Use with caution!)
    #[arg(long)]
    logout: bool,
}

async fn print_login_data(cli_args: &CliArgs, news_flash: &NewsFlash) -> color_eyre::Result<bool> {
    if !cli_args.action().print_login_data {
        return Ok(false);
    }

    let Some(login_data) = news_flash.get_login_data().await else {
        print!("no login data found!");
        return Ok(true);
    };

    print!(
        "{}",
        LoginConfiguration::from(login_data).as_toml(*cli_args.show_secrets())?
    );

    Ok(true)
}

async fn sync(
    config: &Config,
    cli_args: &CliArgs,
    news_flash: &NewsFlash,
    client: &Client,
) -> color_eyre::Result<bool> {
    if !cli_args.action().sync {
        return Ok(false);
    }
    let new_articles = news_flash.sync(client, Default::default()).await?;

    if !*cli_args.quiet() {
        println!(
            "{}",
            config
                .cli_sync_stats_format
                .gen_output(news_flash, &new_articles)?
        );
    }

    Ok(true)
}

pub async fn export_opml(cli_args: &CliArgs, news_flash: &NewsFlash) -> color_eyre::Result<bool> {
    let Some(path) = cli_args.action().export_opml.as_ref() else {
        return Ok(false);
    };
    if !cli_args.quiet() {
        termimad::print_text(&format!(
            "**exporting OPML** to *{}*\n",
            path.to_str().unwrap_or_default()
        ));
    }

    let opml = news_flash.export_opml().await?;
    tokio::fs::write(path, opml).await?;
    if !cli_args.quiet() {
        termimad::print_text("**done**");
    }
    Ok(true)
}

pub async fn import_opml(
    cli_args: &CliArgs,
    news_flash: &NewsFlash,
    client: &Client,
) -> color_eyre::Result<bool> {
    let Some(path) = cli_args.action().import_opml.as_ref() else {
        return Ok(false);
    };

    if !cli_args.quiet() {
        termimad::print_text(&format!(
            "**importing OPML** from *{}*\n",
            path.to_str().unwrap_or_default()
        ));
    }
    let opml = tokio::fs::read_to_string(path).await?;
    news_flash.import_opml(&opml, true, client).await?;
    if !cli_args.quiet() {
        termimad::print_text("**done**");
    }
    Ok(true)
}

pub async fn logout(
    cli_args: &CliArgs,
    news_flash: &NewsFlash,
    client: &Client,
) -> color_eyre::Result<bool> {
    if !cli_args.action.logout {
        return Ok(false);
    };

    if !cli_args.quiet() {
        termimad::print_text("**logging out**\n");
    }
    news_flash.logout(client).await?;
    if !cli_args.quiet() {
        termimad::print_text("**done**");
    }
    Ok(true)
}

pub async fn execute_cli_actions(
    config: &Config,
    cli_args: &CliArgs,
    news_flash: &NewsFlash,
    client: &Client,
) -> color_eyre::Result<bool> {
    // print login data
    if print_login_data(cli_args, news_flash).await? {
        return Ok(true);
    }

    // sync
    if sync(config, cli_args, news_flash, client).await? {
        return Ok(true);
    }

    // export opml
    if export_opml(cli_args, news_flash).await? {
        return Ok(true);
    }

    // export opml
    if import_opml(cli_args, news_flash, client).await? {
        return Ok(true);
    }

    // logout opml
    if logout(cli_args, news_flash, client).await? {
        return Ok(true);
    }

    Ok(false)
}
