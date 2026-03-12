# Stage 1: Build
FROM rust:1.84-bookworm AS builder

RUN rustup target add wasm32-unknown-unknown && \
    cargo install dioxus-cli --version 0.7.3

WORKDIR /app
COPY . .

RUN dx build --package open-eyes-dashboard --platform web --release

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/open-eyes-dashboard /app/open-eyes-dashboard
COPY --from=builder /app/target/dx/open-eyes-dashboard/release/web/public /app/public
COPY config.toml /app/config.toml

RUN mkdir -p /app/data

ENV OPEN_EYES_CONFIG=/app/config.toml
EXPOSE 8080

CMD ["/app/open-eyes-dashboard"]
