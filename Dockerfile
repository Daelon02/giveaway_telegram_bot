# === Build stage ===
FROM rust:1.77 as builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY ./src ./src

RUN cargo build --release

# === Runtime stage ===
FROM debian:bookworm-slim

RUN useradd -m appuser

COPY --from=builder /app/target/release/wallet-service /usr/local/bin/wallet-service

RUN chown appuser:appuser /usr/local/bin/wallet-service
USER appuser

WORKDIR /home/appuser

ENV RUST_LOG=info

CMD ["wallet-service"]