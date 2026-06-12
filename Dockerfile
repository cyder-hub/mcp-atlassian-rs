# syntax=docker/dockerfile:1

FROM rust:1.94-bookworm AS build
WORKDIR /build

COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY xtask/ xtask/
RUN cargo build --release --bin workhub

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 app \
    && useradd --uid 10001 --gid app --home-dir /app --shell /usr/sbin/nologin --no-create-home app

WORKDIR /app

COPY --from=build /build/target/release/workhub /usr/local/bin/workhub

EXPOSE 8000

USER app

CMD ["workhub", "streamhttp", "--host", "0.0.0.0", "--port", "8000"]
