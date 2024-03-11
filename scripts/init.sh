#! /bin/bash

echo "Going to install required cargo packages for development"

echo "Installing cargo-watch..."
cargo install cargo-watch & wait

echo "Installing sqlx-cli..."
cargo install sqlx-cli --no-default-features -F postgres & wait
