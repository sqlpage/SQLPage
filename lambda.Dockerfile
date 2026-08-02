FROM rust:1.95-alpine AS builder
RUN rustup component add clippy rustfmt
RUN apk add --no-cache musl-dev zip
WORKDIR /usr/src/sqlpage
RUN cargo init .
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release
COPY . .
RUN cargo build --release --features lambda-web
RUN   mv target/release/sqlpage bootstrap && \
      strip --strip-all bootstrap && \
      size bootstrap && \
      zip -9 -j deploy.zip bootstrap src/index.sql && \
      zip -9 deploy.zip sqlpage/

FROM public.ecr.aws/lambda/provided:al2023 AS runner
COPY --from=builder /usr/src/sqlpage/bootstrap /main
COPY --from=builder /usr/src/sqlpage/src/index.sql ./index.sql
RUN mkdir -p ./sqlpage
ENTRYPOINT ["/main"]
