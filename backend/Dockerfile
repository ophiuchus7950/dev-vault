# DevVault API — multi-stage build.
#
# The sqlx query macros are verified at compile time. To build without a live
# database we rely on the committed offline metadata (`.sqlx/`), regenerated with
# `cargo sqlx prepare` whenever queries change.

FROM rust:1-slim AS builder
ENV SQLX_OFFLINE=true
WORKDIR /app
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# workspace manifest + lockfile + sources (context is the repo root)
# backend/.sqlx holds the committed offline query metadata, copied along with backend/.
COPY Cargo.toml Cargo.lock ./
COPY backend ./backend
RUN cargo build --release --bin devvault

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -m -u 10001 devvault

WORKDIR /app
COPY --from=builder /app/target/release/devvault /usr/local/bin/devvault

USER devvault
ENV BIND_ADDR=0.0.0.0:3000
EXPOSE 3000
CMD ["devvault"]
