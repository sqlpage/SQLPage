FROM --platform=$BUILDPLATFORM rust:1.72-slim as builder
WORKDIR /usr/src/sqlpage
ARG TARGETARCH
ARG BUILDARCH
RUN apt-get update && \
    if [ "$TARGETARCH" = "$BUILDARCH" ]; then \
        rustup target list --installed > TARGET && \
        apt-get install -y gcc libgcc-s1 && \
        cp /lib/*/libgcc_s.so.1 .; \
    elif [ "$TARGETARCH" = "arm64" ]; then \
        echo aarch64-unknown-linux-gnu > TARGET && \
        apt-get install -y gcc-aarch64-linux-gnu libgcc-s1-arm64-cross && \
        cp /usr/aarch64-linux-gnu/lib/libgcc_s.so.1 .; \
    elif [ "$TARGETARCH" = "arm" ]; then \
        echo armv7-unknown-linux-gnueabihf > TARGET && \
        apt-get install -y gcc-arm-linux-gnueabihf libgcc-s1-armhf-cross && \
        cp /usr/arm-linux-gnueabihf/lib/libgcc_s.so.1 .; \
    else \
        echo "Unsupported cross compilation target: $TARGETARCH"; \
        exit 1; \
    fi && \
    rustup target add $(cat TARGET) && \
    cargo init .
COPY Cargo.toml Cargo.lock ./
COPY .cargo ./.cargo
RUN cargo build --target $(cat TARGET) --profile superoptimized
COPY . .
RUN touch src/main.rs && \
    cargo build --target $(cat TARGET) --profile superoptimized
RUN mv target/$(cat TARGET)/superoptimized/sqlpage sqlpage.bin

FROM busybox:glibc
RUN addgroup --system sqlpage && \
    adduser --system --no-create-home --ingroup sqlpage sqlpage
COPY --from=builder /usr/src/sqlpage/sqlpage.bin /usr/local/bin/sqlpage
COPY --from=builder /usr/src/sqlpage/libgcc_s.so.1 /lib/libgcc_s.so.1
WORKDIR /var/www
USER sqlpage
EXPOSE 8080
CMD ["/usr/local/bin/sqlpage"]