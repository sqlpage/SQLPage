FROM --platform=$BUILDPLATFORM rust:1.72-slim as builder
WORKDIR /usr/src/sqlpage
ARG TARGETARCH
RUN apt-get update && apt-get install -y musl-tools && \ 
    if [ "$TARGETARCH" = "amd64" ]; then  \
        echo x86_64-unknown-linux-gnu > TARGET; \
        apt-get install -y gcc libgcc-s1; \
        cp /lib/x86_64-linux-gnu/libgcc_s.so.1 libgcc_s.so.1; \
    elif [ "$TARGETARCH" = "arm64" ]; then \
        echo aarch64-unknown-linux-gnu > TARGET; \
        apt-get install -y gcc-aarch64-linux-gnu libgcc-s1-arm64-cross; \
        cp /usr/aarch64-linux-gnu/lib/libgcc_s.so.1 libgcc_s.so.1; \
    elif [ "$TARGETARCH" = "arm" ]; then \
        echo armv7-unknown-linux-gnueabihf > TARGET; \
        apt-get install -y gcc-arm-linux-gnueabihf libgcc-s1-armhf-cross; \
        cp /usr/arm-linux-gnueabihf/lib/libgcc_s.so.1 libgcc_s.so.1; \
    else \
        echo "Unknown platform: $TARGETPLATFORM"; \
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
RUN mv target/$(cat TARGET)/superoptimized/sqlpage sqlpage.bin && \
    strip sqlpage.bin

FROM busybox:glibc
RUN addgroup --system sqlpage && \
    adduser --system --no-create-home --ingroup sqlpage sqlpage
COPY --from=builder /usr/src/sqlpage/sqlpage.bin /usr/local/bin/sqlpage
COPY --from=builder /usr/src/sqlpage/libgcc_s.so.1 /lib/libgcc_s.so.1
WORKDIR /var/www
USER sqlpage
EXPOSE 8080
CMD ["/usr/local/bin/sqlpage"]