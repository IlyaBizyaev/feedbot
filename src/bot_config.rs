// SPDX-FileCopyrightText: 2023-2025 Ilya Bizyaev <me@ilyabiz.com>

// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;
use teloxide::types::UserId;

#[derive(Debug, Deserialize)]
pub struct General {
    pub owner_id: UserId,
}

#[derive(Debug, Deserialize)]
pub struct Feed {
    pub url: String,
    pub chat_id: String,
    #[serde(default = "default_post_format")]
    pub post_format: String,
    #[serde(default = "default_cache_size")]
    pub url_cache_size: usize,
}

#[derive(Debug, Deserialize)]
pub struct BotConfig {
    pub general: General,
    pub feeds: Vec<Feed>,
}

fn default_post_format() -> String {
    "$title\n\n$url".to_owned()
}

const fn default_cache_size() -> usize {
    1000
}
