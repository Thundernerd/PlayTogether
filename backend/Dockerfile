# syntax=docker/dockerfile:1

FROM rust:1-bookworm AS builder

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends libsqlite3-dev pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libsqlite3-0 libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

RUN useradd -r -s /bin/false appuser \
    && mkdir -p /data \
    && chown appuser:appuser /data

COPY --from=builder /app/target/release/playtogether-backend /app/playtogether-backend

USER appuser

ENV DATABASE_URL=sqlite:/data/playtogether.db?mode=rwc \
    BIND_ADDR=0.0.0.0:8080

EXPOSE 8080

VOLUME ["/data"]

CMD ["./playtogether-backend"]
