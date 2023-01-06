// SPDX-FileCopyrightText: 2023 Ilya Bizyaev <me@ilyabiz.com>

// SPDX-License-Identifier: Apache-2.0

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BotError {
    #[error("URL has no domain: {url:?}")]
    NoDomain { url: String },
}
