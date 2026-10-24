# Build the application and migration runner.
FROM rust:1.94-bookworm AS builder

# Install native build dependencies.
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Verify the active Rust toolchain.
RUN echo "RUSTUP_TOOLCHAIN=$RUSTUP_TOOLCHAIN" \
    && rustc --version \
    && cargo --version \
    && rustup show active-toolchain

# Copy manifests and source files, then compile release binaries.
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked
COPY migration/Cargo.toml migration/Cargo.lock ./migration/
COPY migration/src ./migration/src
RUN cargo build --release --locked --manifest-path migration/Cargo.toml

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --create-home app

WORKDIR /app
COPY --from=builder /app/target/release/Email-news /usr/local/bin/email-news
COPY --from=builder /app/migration/target/release/migration /usr/local/bin/email-news-migrate

USER app
EXPOSE 8004
CMD ["/bin/sh", "-c", "/usr/local/bin/email-news-migrate up && exec /usr/local/bin/email-news"]
