FROM rust:1.63-slim-buster as builder
RUN rustup component add clippy rustfmt
WORKDIR /usr/src/sqlpage
RUN cargo init .
COPY Cargo.toml Cargo.lock .
RUN cargo build --release
COPY . .
RUN cargo check --release && \
    cargo clippy --release && \
    cargo fmt --all -- --check && \
    cargo install --path .

FROM debian:buster-slim
RUN apt-get update && apt-get upgrade --yes && apt-get install --yes openssl
RUN rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/sqlpage /usr/local/bin/sqlpage
RUN groupadd -r sqlpage && useradd --no-log-init -r -g sqlpage sqlpage
WORKDIR /var/www
USER sqlpage
EXPOSE 8080
CMD ["sqlpage"]