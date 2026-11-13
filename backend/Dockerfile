# syntax=docker/dockerfile:1

# Keep in step with `channel` in backend/rust-toolchain.toml; the build fails if they differ.
ARG RUST_VERSION=1.99.1

FROM rust:${RUST_VERSION}-bookworm AS builder
ARG RUST_VERSION

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends libsqlite3-dev pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# The image tag provides the pinned rustc; this checks it matches
# rust-toolchain.toml and installs the pinned components (rustfmt, clippy)
# in a cached layer, so cargo doesn't fetch them on every source change.
COPY rust-toolchain.toml ./
RUN pinned="$(sed -n 's/^channel = "\(.*\)"$/\1/p' rust-toolchain.toml)" \
    && if [ "$pinned" != "$RUST_VERSION" ]; then \
         echo "RUST_VERSION ($RUST_VERSION) does not match rust-toolchain.toml ($pinned)" >&2; exit 1; \
       fi \
    && rustup toolchain install \
    && rustup show active-toolchain

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

COPY --from=builder /app/target/release/squadlist-backend /app/squadlist-backend
COPY docker-entrypoint.sh /docker-entrypoint.sh
RUN chmod +x /docker-entrypoint.sh

ENV DATABASE_URL=sqlite:/data/playtogether.db?mode=rwc \
    BIND_ADDR=0.0.0.0:8080

EXPOSE 8080

VOLUME ["/data"]

ENTRYPOINT ["/docker-entrypoint.sh"]
CMD ["./squadlist-backend"]
