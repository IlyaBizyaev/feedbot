// SPDX-FileCopyrightText: 2023 Ilya Bizyaev <me@ilyabiz.com>

// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;
use std::collections::VecDeque;
use std::path::PathBuf;

use anyhow::{Context, Result};
use itertools::Itertools;
use tokio::fs::File;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use url::Url;

// This method does not affect the posted URLs, it is only used for in-cache representation.
fn normalize_url(url_str: &str) -> Result<String> {
    // Should fail for relative URLs.
    let url = Url::parse(url_str)
        .with_context(|| format!("Failed to parse the URL to normalize: {}", url_str))?;

    let mut domain = url
        .domain()
        .with_context(|| format!("The URL has no domain: {}", url.as_str()))?;
    domain = domain.strip_prefix("www.").unwrap_or(domain);

    // Starts with "/".
    let mut post_identifier: String = url.path().trim_matches('/').to_owned();
    if post_identifier.is_empty() {
        if let Some(query) = url.query() {
            post_identifier = post_identifier + "?" + query;
        }
    } else {
        for suffix in [".htm", ".html"] {
            if let Some(stripped) = post_identifier.strip_suffix(suffix) {
                post_identifier = stripped.to_owned();
                break;
            }
        }
    }

    let result = domain.to_owned() + "/" + &post_identifier;
    if result != url_str {
        log::debug!("Normalized '{}' to '{}'", url_str, result);
    }

    Ok(result)
}

pub struct UrlCache {
    filename: PathBuf,
    cache_size: usize,
    cache_list: VecDeque<String>,
    cache_set: HashSet<String>,
}

impl UrlCache {
    pub fn new(filename: PathBuf, cache_size: usize) -> Self {
        Self {
            filename,
            cache_size,
            cache_list: VecDeque::with_capacity(cache_size),
            cache_set: HashSet::with_capacity(cache_size),
        }
    }

    pub async fn load(&mut self) -> Result<()> {
        let file: Result<File, io::Error> = File::open(&self.filename).await;
        match file {
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                // The cache file does not exist yet, consider the cache empty.
                self.cache_list.clear();
                self.cache_set.clear();
                Ok(())
            }
            Err(e) => {
                // Other IO errors.
                Err(e).with_context(|| {
                    format!("Failed to open cache file: {}", self.filename.display())
                })
            }
            Ok(mut file) => {
                let mut cache_file_contents = String::new();
                file.read_to_string(&mut cache_file_contents)
                    .await
                    .context("Failed to load a cache line")?;
                let mut lines: VecDeque<String> = cache_file_contents
                    .split('\n')
                    .map(|l| l.to_owned())
                    .filter(|l| !l.is_empty())
                    .collect();
                if lines.len() > self.cache_size {
                    // E.g. if the limit was reduced after we saved the cache.
                    lines = lines.split_off(lines.len() - self.cache_size);
                }
                self.cache_list = lines;
                self.cache_set = HashSet::from_iter(self.cache_list.clone());
                Ok(())
            }
        }
    }

    pub fn insert(&mut self, url: &str) -> Result<bool> {
        let normalized_url = normalize_url(url)?;
        let is_new_url = self.cache_set.insert(normalized_url.clone());

        if is_new_url {
            self.cache_list.push_back(normalized_url);
            if self.cache_list.len() > self.cache_size {
                let old_url = self.cache_list.pop_front().unwrap();
                self.cache_set.remove(&old_url);
            }
        }

        Ok(is_new_url)
    }

    pub async fn save(&self) -> Result<()> {
        let mut file = File::create(&self.filename).await.with_context(|| {
            format!(
                "Failed to open cache file for writing: {}",
                self.filename.display()
            )
        })?;
        file.write_all(self.cache_list.iter().join("\n").as_bytes())
            .await
            .with_context(|| {
                format!(
                    "Failed to write to the cache file: {}",
                    &self.filename.display()
                )
            })?;

        Ok(())
    }
}
