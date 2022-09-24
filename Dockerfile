FROM rust:1.64-alpine3.16 as builder
RUN rustup component add clippy rustfmt
RUN apk add --no-cache musl-dev
WORKDIR /usr/src/sqlpage
RUN cargo init .
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release
COPY . .
RUN touch src/main.rs
ARG SKIP_CHECK
RUN if [[ -z "$SKIP_CHECK" ]] ; then \
        cargo clippy --release && \
        cargo fmt --all -- --check && \
        cargo test --release; \
    fi
RUN cargo build --release

FROM alpine:3.16
RUN rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/sqlpage/target/release/sqlpage /usr/local/bin/sqlpage
RUN addgroup -S sqlpage && adduser -S sqlpage -G sqlpage
WORKDIR /var/www
COPY --from=builder /usr/src/sqlpage/index.sql .
USER sqlpage
EXPOSE 8080
CMD ["sqlpage"]