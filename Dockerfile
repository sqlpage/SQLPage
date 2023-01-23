FROM rust:1.66.1-alpine3.17 as builder
RUN apk add --no-cache musl-dev
WORKDIR /usr/src/sqlpage
RUN cargo init .
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release
COPY . .
RUN touch src/main.rs
RUN cargo build --release

FROM alpine:3.17
RUN rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/sqlpage/target/release/sqlpage /usr/local/bin/sqlpage
RUN addgroup -S sqlpage && adduser -S sqlpage -G sqlpage
WORKDIR /var/www
COPY --from=builder /usr/src/sqlpage/index.sql .
USER sqlpage
EXPOSE 8080
CMD ["sqlpage"]