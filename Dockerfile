#build the application
FROM rust:1.85-bookworm AS builder

#install dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

#cache dependencies
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
CMD ["/usr/local/bin/email-news"]
