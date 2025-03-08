// SPDX-FileCopyrightText: 2023-2025 Ilya Bizyaev <me@ilyabiz.com>

// SPDX-License-Identifier: Apache-2.0

mod bot_config;
mod cache;

use std::future::IntoFuture;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use figment::{
    Figment,
    providers::{Env, Format, Json},
};
use futures::future;
use tokio::fs;
use url::form_urlencoded;

use rss::Channel;
use teloxide::prelude::*;
use teloxide::types::ParseMode;
use teloxide::utils::html::escape;

use bot_config::BotConfig;
use cache::UrlCache;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
/// A Telegram bot to forward posts from RSS channels.
struct Args {
    /// Config file location.
    #[arg(long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Writable cache directory location.
    #[arg(long, value_name = "DIR", default_value_os_t = PathBuf::from("cache"))]
    cache: PathBuf,

    /// Dry-run mode: skip updating the cache and publishing the posts.
    #[arg(long, default_value_t = false)]
    dry_run: bool,
}

async fn fetch_feed(feed_url: &str) -> Result<Channel> {
    let content = reqwest::get(feed_url).await?.bytes().await?;
    let channel = Channel::read_from(&content[..])?;
    Ok(channel)
}

fn format_item(item: &rss::Item, post_format: &str) -> String {
    post_format
        .replace("\\n", "\n")
        .replace("$title", &escape(item.title().unwrap_or("")))
        .replace("$url", &escape(item.link().unwrap()))
        .replace("$date", &escape(item.pub_date().unwrap_or("")))
        .replace("$author", &escape(item.author().unwrap_or("")))
}

fn make_cache_filename(cache_dir: &Path, chat_id: &str, feed_url: &str) -> PathBuf {
    let feed_url_encoded: String = form_urlencoded::byte_serialize(feed_url.as_bytes()).collect();
    cache_dir.join(format!("{}-{}.txt", chat_id, feed_url_encoded))
}

async fn handle_feed(
    cache_dir: PathBuf,
    feed: &bot_config::Feed,
    bot: &Bot,
    owner_id: UserId,
    dry_run: bool,
) -> Result<()> {
    log::info!("Processing feed {}", &feed.url);

    let cache_filename = make_cache_filename(&cache_dir, &feed.chat_id, &feed.url);
    let mut url_cache = UrlCache::new(cache_filename, feed.url_cache_size);
    url_cache.load().await?;

    let feed_data = fetch_feed(&feed.url)
        .await
        .context("Could not fetch the feed")?;
    let item_tasks: Vec<_> = feed_data
        .items
        .iter()
        .filter_map(|item| {
            let url = item.link();
            match url {
                None => Some(
                    bot.send_message(owner_id.to_owned(), format!("No URL in item {:?}", item)),
                ),
                Some(url) => match url_cache.insert(url) {
                    Err(cache_error) => Some(bot.send_message(
                        owner_id.to_owned(),
                        format!("Malformed item URL '{}': {}", url, cache_error),
                    )),
                    Ok(false) => {
                        log::info!("Known URL: {}, nothing to do", url);
                        None
                    }
                    Ok(true) => {
                        log::info!("Posting {} to {}", url, feed.chat_id);
                        let message_text = format_item(item, &feed.post_format);
                        if dry_run {
                            None
                        } else {
                            Some(
                                bot.send_message(feed.chat_id.clone(), message_text)
                                    .parse_mode(ParseMode::Html),
                            )
                        }
                    }
                },
            }
        })
        .map(|req| req.into_future())
        .collect();
    future::join_all(item_tasks)
        .await
        .into_iter()
        .collect::<Result<Vec<Message>, _>>()?;

    url_cache.save().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .try_init()
        .context("Failed to initialize the logger")?;

    log::info!("Parsing the settings…");
    let args = Args::parse();
    let bot_config = if let Some(config_file_path) = args.config {
        Figment::new().merge(Json::file(config_file_path))
    } else {
        Figment::new().merge(Env::prefixed("FEEDBOT_"))
    };
    let bot_config: BotConfig = bot_config
        .extract()
        .context("Failed to deserialize the config")?;

    if bot_config.feeds.is_empty() {
        log::warn!("No feeds are configured, nothing to do.");
        return Ok(());
    }

    if !args.dry_run {
        fs::create_dir_all(args.cache.as_path())
            .await
            .with_context(|| {
                format!(
                    "Could not create the cache directory ('{}')",
                    args.cache.display()
                )
            })?;
    }

    log::info!("Starting the bot…");
    let bot = Bot::from_env();
    log::info!("The bot has started");

    log::info!("Starting the tasks…");
    let feed_tasks: Vec<_> = bot_config
        .feeds
        .iter()
        .map(|feed| {
            handle_feed(
                args.cache.clone(),
                feed,
                &bot,
                bot_config.general.owner_id,
                args.dry_run,
            )
        })
        .collect();
    let feed_task_results = future::join_all(feed_tasks).await;
    let feed_task_errors = feed_task_results
        .into_iter()
        .filter_map(|result| result.err())
        .collect::<Vec<_>>();
    for e in feed_task_errors {
        log::error!("Task failed: {:#?}", e);
    }
    log::info!("The tasks have finished");

    Ok(())
}
