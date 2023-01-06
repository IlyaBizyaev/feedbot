// SPDX-FileCopyrightText: 2023 Ilya Bizyaev <me@ilyabiz.com>

// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;

use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct General {
    pub owner_id: String,
    pub debug: bool,
}

#[derive(Debug, Deserialize)]
pub struct Feed {
    pub url: String,
    pub chat_id: String,
    pub post_format: String,
    pub url_cache_size: usize,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub general: General,
    pub feeds: Vec<Feed>,
}

impl Config {
    pub fn parse_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let config = serde_json::from_reader(reader)?;
        Ok(config)
    }
}
