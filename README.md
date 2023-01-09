# feedbot

A simple Telegram bot to forward posts from RSS channels.

## Configuration

1) Provide a configuration file named `config.json` in the bot's working directory
(path override: `--config`):

```json
{
    "general": {
        "owner_id": "1234567",
        "debug": false
    },
    "feeds": [
        {
            "url": "https://example.com/index.xml",
            "chat_id": "123123123",
            "post_format": "$title\n\n$url",
            "url_cache_size": 1000
        }
    ]
}
```

2) Set the `TELOXIDE_TOKEN` environment variable to your bot's Telegram API token.

3) Make sure that the `cache` subdirectory of the bot's working directory is writable, as the bot
will use it to store the URL cache (path override: `--cache`).

4) Use something like Cron to schedule regular bot launches (see [`systemd`](systemd) for a sample
systemd configuration).

## Dependencies

See [`Cargo.toml`](./Cargo.toml) for the full list of dependencies.

## License

This project is licensed under the Apache 2.0 license and follows the REUSE Specification. See
[`LICENSES`](./LICENSES) for the full license texts.
