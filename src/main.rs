// SPDX-FileCopyrightText: 2023 Ilya Bizyaev <me@ilyabiz.com>

// SPDX-License-Identifier: Apache-2.0

mod cache;
mod config;
mod errors;

use anyhow::Result;
use futures::future;
use log::LevelFilter;
use rss::Channel;
use std::future::IntoFuture;
use teloxide::prelude::*;
use teloxide::types::ParseMode;
use teloxide::utils::html::escape;

use cache::UrlCache;
use config::Config;

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

async fn handle_feed(feed: &config::Feed, bot: &Bot, owner_id: &str) -> Result<()> {
    log::info!("Processing feed {}", &feed.url);
    let mut url_cache = UrlCache::new(&feed.chat_id, &feed.url, feed.url_cache_size);
    url_cache.load().await?;

    let feed_data = fetch_feed(&feed.url)
        .await
        .unwrap_or_else(|e| panic!("Could not fetch the feed: {}", e));
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
                        let message_text = format_item(item, &feed.post_format);
                        Some(
                            bot.send_message(feed.chat_id.clone(), message_text)
                                .parse_mode(ParseMode::Html),
                        )
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
    let config = Config::parse_from_file("config.json")
        .unwrap_or_else(|e| panic!("Could not read config: {}", e));

    let log_filter = if config.general.debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Warn
    };
    pretty_env_logger::formatted_builder()
        .filter_level(log_filter)
        .init();
    log::info!("Debug logging enabled");

    log::info!("Starting the bot…");
    let bot = Bot::from_env();
    log::info!("The bot has started");

    log::info!("Starting the tasks…");
    let feed_tasks: Vec<_> = config
        .feeds
        .iter()
        .map(|feed| handle_feed(feed, &bot, &config.general.owner_id))
        .collect();
    future::join_all(feed_tasks)
        .await
        .into_iter()
        .collect::<Result<Vec<()>, _>>()
        .log_on_error()
        .await;
    log::info!("The tasks have finished");

    Ok(())
}
