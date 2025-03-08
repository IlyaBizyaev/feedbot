# SPDX-FileCopyrightText: 2025 Ilya Bizyaev <me@ilyabiz.com>

# SPDX-License-Identifier: Apache-2.0

project := "feedbot"
arch := "x86_64-unknown-linux-musl"
build_type := "release"

target_dir := `cargo metadata --format-version 1 | jq -r '.target_directory'`
build_type_flag := if build_type == "release" { "--release" } else { "" }
target_dir_for_build := target_dir / arch / build_type
built_binary := target_dir_for_build / project

src_dir := justfile_directory()
final_binary := src_dir / project

default:
    just -l

license:
    cargo deny check

reuse:
    reuse --root {{src_dir}} lint

prod:
    # TODO: use https://github.com/rust-lang/rust/issues/120953 once stabilized.
    RUSTFLAGS="-C link-arg=-Wl,--compress-debug-sections=zlib-gabi" cargo build {{build_type_flag}} --target={{arch}}
    rm -f {{final_binary}}
    cp {{built_binary}} {{final_binary}}
    echo "Final binary size:" $(du -h {{final_binary}} | cut -f1)
    notify-send -a {{project}} "Build done!"
