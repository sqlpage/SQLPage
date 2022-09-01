FROM rust:1.63-alpine3.16 as builder
RUN rustup component add clippy rustfmt
RUN apk add --no-cache musl-dev
WORKDIR /usr/src/sqlpage
RUN cargo init .
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release
COPY . .
RUN touch src/main.rs && \
    cargo clippy --release && \
    cargo fmt --all -- --check && \
    cargo test --release && \
    cargo build --release

FROM builder AS lambda-build
RUN   apk add zip
RUN cargo build --release --features lambda-web
RUN   mv target/release/sqlpage bootstrap && \
      strip --strip-all bootstrap && \
      size bootstrap && \
      ldd  bootstrap && \
      zip -9 -r deploy.zip bootstrap index.sql documentation.sql sqlpage
CMD ["./bootstrap"]

FROM alpine:3.16
RUN rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/sqlpage/target/release/sqlpage /usr/local/bin/sqlpage
RUN addgroup -S sqlpage && adduser -S sqlpage -G sqlpage
WORKDIR /var/www
USER sqlpage
EXPOSE 8080
CMD ["sqlpage"]