<!--
SPDX-FileCopyrightText: 2025 Ilya Bizyaev <me@ilyabiz.com>
SPDX-License-Identifier: Apache-2.0
-->

# feedbot

A simple Telegram bot to forward posts from RSS channels.

## Configuration

1) Provide a JSON configuration file (`--config`):

```json
{
    "general": {
        "owner_id": "1234567"
    },
    "feeds": [
        {
            "url": "https://example.com/index.xml",
            "chat_id": "123123123",
            "post_format": "$title\n\n$url"
        }
    ]
}
```

Or, alternatively, set the environment variables:

```ini
FEEDBOT_GENERAL='{owner_id=1234567}'
FEEDBOT_FEEDS='[{url="https://example.com/index.xml", chat_id="123123123"}]'
```

2) Set the `TELOXIDE_TOKEN` environment variable to your bot's Telegram API token.

3) Make sure that the bot has a writable `--cache` directory: it will use it to store the URL cache (defaults to `./cache`).

4) Use something like Cron to schedule regular bot launches (see [`systemd`](systemd) for a sample
systemd configuration).

## Dependencies

See [`Cargo.toml`](./Cargo.toml) for the full list of dependencies.

## License

This project is licensed under the Apache 2.0 license and follows the REUSE Specification. See
[`LICENSES`](./LICENSES) for the full license texts.
