FROM rust:1.72-alpine3.17 as builder
RUN apk add --no-cache musl-dev
WORKDIR /usr/src/sqlpage
RUN cargo init .
COPY Cargo.toml Cargo.lock ./
RUN cargo build --profile superoptimized
COPY . .
RUN touch src/main.rs && \
    cargo build --profile superoptimized

FROM alpine:3.17
RUN rm -rf /var/lib/apt/lists/* && \
    addgroup -S sqlpage && adduser -S sqlpage -G sqlpage
COPY --from=builder /usr/src/sqlpage/target/superoptimized/sqlpage /usr/local/bin/sqlpage
WORKDIR /var/www
COPY --from=builder /usr/src/sqlpage/index.sql .
USER sqlpage
EXPOSE 8080
CMD ["sqlpage"]