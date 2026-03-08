FROM rust:1.91-slim AS builder
WORKDIR /usr/src/sqlpage
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev unixodbc-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && mkdir -p src/webserver && cargo build --release 2>/dev/null || true
COPY . .
RUN touch src/main.rs && cargo build --release
RUN cp target/release/sqlpage /usr/local/bin/sqlpage

FROM debian:trixie-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates unixodbc && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/bin/sqlpage /usr/local/bin/sqlpage
COPY --from=builder /usr/src/sqlpage/sqlpage/sqlpage.db /etc/sqlpage/sqlpage.db
ENV SQLPAGE_WEB_ROOT=/var/www
ENV SQLPAGE_CONFIGURATION_DIRECTORY=/etc/sqlpage
WORKDIR /var/www
EXPOSE 8080
CMD ["sqlpage"]
