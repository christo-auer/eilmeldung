use std::path::PathBuf;

use crate::prelude::*;
use clap::{Args, Parser};
use getset::Getters;
use log::LevelFilter;
use news_flash::{
    NewsFlash,
    models::{Category, CategoryID, Feed},
};
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

    /// Directory with config files (config.toml, etc.)
    #[arg(short, long)]
    config_dir: Option<String>,

    /// Directory with state files (database, etc.)
    #[arg(short, long)]
    state_dir: Option<String>,

    /// Show secrets when printing login data
    #[arg(long)]
    show_secrets: bool,

    #[command(flatten)]
    action: CliAction,
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

    let login_data = news_flash.get_login_data().await.unwrap();
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
    let (mut feeds, feed_mapping) = news_flash.get_feeds()?;
    let feed_for_feed_id = NewsFlashUtils::generate_id_map(&feeds, |feed| feed.feed_id.to_owned());
    let feed_mapping_for_feed_id =
        NewsFlashUtils::generate_id_map(&feed_mapping, |mapping| mapping.feed_id.to_owned());
    let (mut categories, category_mapping) = news_flash.get_categories()?;
    let category_for_category_id =
        NewsFlashUtils::generate_id_map(&categories, |category| category.category_id.to_owned());
    let category_mapping_for_category_id =
        NewsFlashUtils::generate_id_map(&category_mapping, |category_mapping| {
            category_mapping.category_id.to_owned()
        });

    sort_feeds_and_categories(
        &mut feeds,
        &mut categories,
        &feed_mapping_for_feed_id,
        &category_mapping_for_category_id,
    );

    let all_unread: i64 = new_articles.values().sum();

    if all_unread > 0 && config.cli.sync_output_all {
        // println!(config.cli.sync_output_format.replac);
        println!(
            "{}",
            config
                .cli
                .sync_output_format
                .replace("{label}", &config.cli.all_label_format)
                .replace("{count}", &all_unread.to_string())
        );
    }

    if config.cli.sync_output_feeds {
        feeds
            .iter()
            .filter(|feed| *new_articles.get(&feed.feed_id).unwrap_or(&0) > 0)
            .for_each(|feed| {
                println!(
                    "{}",
                    config
                        .cli
                        .sync_output_format
                        .replace("{label}", &config.cli.feed_label_format)
                        .replace(
                            "{category}",
                            feed_mapping_for_feed_id
                                .get(&feed.feed_id)
                                .and_then(|mapping| category_for_category_id
                                    .get(&mapping.category_id)
                                    .map(|category| &*category.label))
                                .unwrap_or_default(),
                        )
                        .replace(
                            "{label}",
                            feed_for_feed_id
                                .get(&feed.feed_id)
                                .map(|feed| &feed.label)
                                .unwrap_or(&"".to_owned()),
                        )
                        .replace(
                            "{count}",
                            &new_articles.get(&feed.feed_id).unwrap_or(&0).to_string()
                        )
                );
            });
    }

    Ok(true)
}

fn sort_feeds_and_categories(
    feeds: &mut [Feed],
    categories: &mut [Category],
    feed_mapping_for_feed_id: &std::collections::HashMap<
        news_flash::models::FeedID,
        news_flash::models::FeedMapping,
    >,
    category_mapping_for_category_id: &std::collections::HashMap<
        news_flash::models::CategoryID,
        news_flash::models::CategoryMapping,
    >,
) {
    let category_cmp = |c1: Option<&CategoryID>, c2: Option<&CategoryID>| {
        let sort_index_for_c1 = c1.and_then(|c_id| {
            category_mapping_for_category_id
                .get(c_id)
                .map(|mapping| &mapping.sort_index)
        });
        let sort_index_for_c2 = c2.and_then(|c_id| {
            category_mapping_for_category_id
                .get(c_id)
                .map(|mapping| &mapping.sort_index)
        });

        sort_index_for_c1.cmp(&sort_index_for_c2)
    };

    categories.sort_by(|c1, c2| category_cmp(Some(&c1.category_id), Some(&c2.category_id)));

    feeds.sort_by(|f1, f2| {
        let feed_mapping_for_f1 = feed_mapping_for_feed_id.get(&f1.feed_id);
        let feed_mapping_for_f2 = feed_mapping_for_feed_id.get(&f2.feed_id);

        category_cmp(
            feed_mapping_for_f1.map(|mapping| &mapping.category_id),
            feed_mapping_for_f2.map(|mapping| &mapping.category_id),
        )
        .then(
            feed_mapping_for_f1
                .map(|feed_mapping| feed_mapping.sort_index)
                .cmp(&feed_mapping_for_f2.map(|feed_mapping| feed_mapping.sort_index)),
        )
    });
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

    Ok(false)
}
