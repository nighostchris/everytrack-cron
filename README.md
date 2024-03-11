# Everytrack Cron

Service that handles periodic job for Everytrack.

# Table of Contents

- [Pre-requisite](#pre-requisite)
- [Usage Guide](#usage-guide)
  - [Database Setup](#database-setup)

## Pre-requisite

Rust

```bash
❯ rustup --version
rustup 1.26.0 (5af9b9484 2023-04-05)
❯ rustc --version
rustc 1.76.0 (07dca489a 2024-02-04)
```

[Docker Desktop - Mac M1 version .dmg](https://desktop.docker.com/mac/main/arm64/Docker.dmg?utm_source=docker&utm_medium=webreferral&utm_campaign=docs-driven-download-mac-arm64)

## Usage Guide

Create a file `.env` and copy the environment variables from `.env.example`

Adjust the variable values according to your needs

Run the project setup script after setting up local database

```bash
./scripts/init.sh
# So that we can use sqlx offline mode while we are developing
cargo sqlx prepare --database-url postgresql://root:root@localhost:5432/root
```

Start the server by running

```bash
# Please don't use this in production
cargo watch -q -c -w src/ -x run
# Use this in production instead
cargo run
```
