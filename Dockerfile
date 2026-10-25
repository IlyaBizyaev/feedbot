# SPDX-FileCopyrightText: 2026 Ilya Bizyaev <me@ilyabiz.com>
# SPDX-License-Identifier: Apache-2.0

FROM rust:1-alpine3.24 AS builder

RUN apk add --no-cache build-base
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked

FROM alpine:3.24

RUN apk add --no-cache ca-certificates \
    && addgroup -S -g 10001 feedbot \
    && adduser -S -D -H -u 10001 -G feedbot feedbot \
    && mkdir /data \
    && chown feedbot:feedbot /data
WORKDIR /app
COPY --from=builder /build/target/release/feedbot /usr/local/bin/feedbot
USER feedbot
ENTRYPOINT ["feedbot", "--config", "/app/config.json", "--cache", "/data"]
