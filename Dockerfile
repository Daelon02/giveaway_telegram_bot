# === Build stage ===
FROM rust:1.86 AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY ./src ./src
COPY .env .env

RUN cargo build --release

# === Runtime stage ===
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
 && rm -rf /var/lib/apt/lists/*

RUN useradd -m appuser

COPY --from=builder /app/target/release/telegram_bot /usr/local/bin/telegram_bot

COPY --from=builder /app/.env /usr/local/bin/.env

RUN chown appuser:appuser /usr/local/bin/telegram_bot
RUN chown appuser:appuser /usr/local/bin/.env

USER appuser

WORKDIR /usr/local/bin

ENV RUST_LOG=info

CMD ["telegram_bot"]