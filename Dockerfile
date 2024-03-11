# syntax = docker/dockerfile:1.2
FROM rust:1.76.0 AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef as planner
COPY . .
# Compute a lock-like file for our project
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
# Build our project dependencies
RUN cargo chef cook --release --recipe-path recipe.json
# All layers should be cached if our dependency tree stays the same 
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release --bin everytrack_cron

FROM rust:1.76.0-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/everytrack_cron everytrack_cron
RUN --mount=type=secret,id=_env,dst=/etc/secrets/.env cp /etc/secrets/.env .
CMD ["./everytrack_cron"]
