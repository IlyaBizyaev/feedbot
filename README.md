<!--
SPDX-FileCopyrightText: 2025-2026 Ilya Bizyaev <me@ilyabiz.com>
SPDX-License-Identifier: Apache-2.0
-->

# feedbot

A simple Telegram bot to forward posts from RSS channels.

## Build and run

Install Rust and Cargo, then build the executable:

```sh
cargo build --release
```

Once configured, run the bot with:

```sh
./target/release/feedbot --config config.json
```

Each launch checks the configured feeds once and exits. For the initial launch, you may want to use the `--dry-run` mode to record the feed's existing items but avoid posting them.

## Docker Compose

* Place the bot's configuration in `./feedbot/config.json`, relative to [`docker-compose.yml`](compose.yaml). The configuration must be readable by the container's user (UID 10001).
* Set `FEEDBOT_TELOXIDE_TOKEN=your-bot-token` in the Compose project's `.env` file. Restrict access to `.env` with `chmod 600 .env`.
* Before the first run, create the cache directory. If migrating an existing deployment, stop its bot and schedule first, then copy the contents of its cache directory before launching the container:

```sh
mkdir -p feedbot/cache
# When migrating an existing deployment:
# cp -a /path/to/old/cache/. feedbot/cache/
sudo chown -R 10001:10001 feedbot/cache
```

Pull the image or build it from source, then check the feeds once in dry-run mode:

```sh
sudo docker compose pull feedbot
# Or, from the source checkout: sudo docker compose build feedbot
sudo docker compose run --pull never --rm --no-deps -T feedbot --dry-run
```

This records existing items in the cache without publishing posts. Omit `--dry-run` if you want to publish posts.

The cache files live in `./feedbot/cache` on the host, mounted at `/data` in the container. They persist across reboots and container removal; include this directory in your backups. Run only one instance against that cache at a time.

To upgrade, run `sudo docker compose pull feedbot`.

### Scheduled Docker runs

* Set `WorkingDirectory` in `feedbot-docker.service` to your Compose project directory.
* Install [`feedbot-docker.service`](systemd/feedbot-docker.service) and [`feedbot-docker.timer`](systemd/feedbot-docker.timer) into `/etc/systemd/system/`, then enable the timer:

```sh
sudo systemctl daemon-reload
sudo systemctl enable --now feedbot-docker.timer
```

View logs with `sudo journalctl -u feedbot-docker.service`.

## Configuration

1) Provide a JSON configuration file (`--config`):

   ```json
   {
       "general": {
           "owner_id": 1234567
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

   ```sh
   export FEEDBOT_GENERAL='{owner_id=1234567}'
   export FEEDBOT_FEEDS='[{url="https://example.com/index.xml", chat_id="123123123"}]'
   ```

   `--config` takes precedence over `FEEDBOT_*` environment variables.

2) Set the `TELOXIDE_TOKEN` environment variable to your bot's Telegram API token.

3) Make sure that the bot has a writable `--cache` directory: it will use it to store the URL cache (defaults to `./cache`).

4) Use something like Cron to schedule regular bot launches (see [`systemd`](systemd) for a sample systemd configuration).

### Feed options and post formatting

Each feed requires `url` and `chat_id`. The following fields are optional:

| Field | Default | Description |
| --- | --- | --- |
| `post_format` | `$title\n\n$url` | Template for each Telegram message. |
| `url_cache_size` | `1000` | Maximum number of remembered URLs per feed and destination chat. |

Post templates support `$title`, `$url`, `$date`, `$author`, and Telegram-supported HTML formatting. Both newline characters and literal `\n` sequences produce line breaks.

Missing titles, dates, and authors become empty strings. Items without a URL are reported and skipped.

### Cache and first launch

Persist the bot's cache directory between launches to avoid reposting known URLs. With an empty cache, all valid linked items currently in the feed are eligible for posting, including older entries.

## Dependencies

See [`Cargo.toml`](./Cargo.toml) for the full list of dependencies.

## License

This project is licensed under the Apache 2.0 license and follows the REUSE Specification. See [`LICENSES`](./LICENSES) for the full license texts.
