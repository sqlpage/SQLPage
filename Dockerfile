FROM rust:1.63 as builder
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
RUN rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/sqlpage /usr/local/bin/sqlpage
USER sqlpage
CMD ["sqlpage"]