<!--
SPDX-FileCopyrightText: 2026 Ilya Bizyaev <me@ilyabiz.com>
SPDX-License-Identifier: Apache-2.0
-->

# AGENTS.md

See [README.md](README.md) for build, configuration, and runtime usage; keep those instructions there.

- Code lives in `src/main.rs` (fetching, formatting, sending), `src/bot_config.rs` (configuration), and `src/cache.rs` (URL normalization and persistence).
- Preserve existing cache filenames and normalized URL semantics unless intentionally changing compatibility: changes can cause reposts. Normalization is for deduplication only; messages use the original URL.
- Keep feed-supplied values HTML-escaped when changing message formatting.
- Validate Rust changes with `cargo fmt --check`, `cargo test`, and `cargo clippy --all-targets -- -D warnings`. Prefer local fixtures and temporary cache directories for tests.
- Do not use a live bot for routine validation: `--dry-run` still writes caches and can send owner notifications.
- Follow the existing SPDX header style in new files. Run `just reuse` for licensing changes and `just license` for dependency changes (requires `reuse` and `cargo-deny` respectively).
