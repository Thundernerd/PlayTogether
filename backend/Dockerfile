# syntax=docker/dockerfile:1

# Keep in sync with backend/rust-toolchain.toml.
FROM rust:1.99.1-bookworm AS builder

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends libsqlite3-dev pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# The image tag already provides the pinned rustc; this installs the pinned
# components (rustfmt, clippy) in a cached layer. If the tag ever drifts from
# rust-toolchain.toml, rustup installs the pinned version here instead.
COPY rust-toolchain.toml ./
RUN rustup toolchain install && rustup show active-toolchain

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libsqlite3-0 libssl3 gosu \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

RUN useradd -r -s /bin/false appuser \
    && mkdir -p /data \
    && chown appuser:appuser /data

COPY --from=builder /app/target/release/playtogether-backend /app/playtogether-backend
COPY docker-entrypoint.sh /docker-entrypoint.sh
RUN chmod +x /docker-entrypoint.sh

ENV DATABASE_URL=sqlite:/data/playtogether.db?mode=rwc \
    BIND_ADDR=0.0.0.0:8080

EXPOSE 8080

VOLUME ["/data"]

ENTRYPOINT ["/docker-entrypoint.sh"]
CMD ["./playtogether-backend"]
