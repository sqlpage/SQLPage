FROM --platform=$BUILDPLATFORM rust:1.90-slim AS builder
WORKDIR /usr/src/sqlpage
ARG TARGETARCH
ARG BUILDARCH
RUN apt-get update && \
    mkdir -p /opt/sqlpage-libs && \
    if [ "$TARGETARCH" = "$BUILDARCH" ]; then \
        rustup target list --installed > TARGET && \
        echo gcc > LINKER && \
        apt-get install -y gcc libgcc-s1 cmake pkg-config && \
        LIBMULTIARCH=$(gcc -print-multiarch); \
        LIBDIR="/lib/$LIBMULTIARCH"; \
        USRLIBDIR="/usr/lib/$LIBMULTIARCH"; \
        HOST_TRIPLE=$(gcc -dumpmachine); \
    elif [ "$TARGETARCH" = "arm64" ]; then \
        echo aarch64-unknown-linux-gnu > TARGET && \
        echo aarch64-linux-gnu-gcc > LINKER && \
        dpkg --add-architecture arm64 && apt-get update && \
        apt-get install -y gcc-aarch64-linux-gnu libgcc-s1-arm64-cross pkg-config && \
        LIBDIR="/lib/aarch64-linux-gnu"; \
        USRLIBDIR="/usr/lib/aarch64-linux-gnu"; \
        HOST_TRIPLE="aarch64-linux-gnu"; \
    elif [ "$TARGETARCH" = "arm" ]; then \
        echo armv7-unknown-linux-gnueabihf > TARGET && \
        echo arm-linux-gnueabihf-gcc > LINKER && \
        dpkg --add-architecture armhf && apt-get update && \
        apt-get install -y gcc-arm-linux-gnueabihf libgcc-s1-armhf-cross cmake libclang1 clang pkg-config && \
        cargo install --force --locked bindgen-cli && \
        SYSROOT=$(arm-linux-gnueabihf-gcc -print-sysroot); \
        echo "--sysroot=$SYSROOT -I$SYSROOT/usr/include -I$SYSROOT/usr/include/arm-linux-gnueabihf" > BINDGEN_EXTRA_CLANG_ARGS; \
        LIBDIR="/lib/arm-linux-gnueabihf"; \
        USRLIBDIR="/usr/lib/arm-linux-gnueabihf"; \
        HOST_TRIPLE="arm-linux-gnueabihf"; \
    else \
        echo "Unsupported cross compilation target: $TARGETARCH"; \
        exit 1; \
    fi && \
    cp $LIBDIR/libgcc_s.so.1 /opt/sqlpage-libs/ && \
    rustup target add $(cat TARGET) && \
    cargo init .

# Build dependencies (creates a layer that avoids recompiling dependencies on every build)
COPY Cargo.toml Cargo.lock ./
RUN BINDGEN_EXTRA_CLANG_ARGS=$(cat BINDGEN_EXTRA_CLANG_ARGS || true) \
    cargo build \
     --target $(cat TARGET) \
     --config target.$(cat TARGET).linker='"'$(cat LINKER)'"' \
     --features odbc-static \
     --profile superoptimized

# Build the project
COPY . .
RUN touch src/main.rs && \
    cargo build \
        --target $(cat TARGET) \
        --config target.$(cat TARGET).linker='"'$(cat LINKER)'"' \
        --features odbc-static \
        --profile superoptimized && \
    mv target/$(cat TARGET)/superoptimized/sqlpage sqlpage.bin

FROM busybox:glibc
RUN addgroup --gid 1000 --system sqlpage && \
    adduser --uid 1000 --system --no-create-home --ingroup sqlpage sqlpage && \
    mkdir -p /etc/sqlpage && \
    touch /etc/sqlpage/sqlpage.db && \
    chown -R sqlpage:sqlpage /etc/sqlpage/sqlpage.db
ENV SQLPAGE_WEB_ROOT=/var/www
ENV SQLPAGE_CONFIGURATION_DIRECTORY=/etc/sqlpage
WORKDIR /var/www
COPY --from=builder /usr/src/sqlpage/sqlpage.bin /usr/local/bin/sqlpage
# Provide runtime helper libs in system lib directory for the glibc busybox base
COPY --from=builder /opt/sqlpage-libs/* /lib/
USER sqlpage
COPY --from=builder --chown=sqlpage:sqlpage /usr/src/sqlpage/sqlpage/sqlpage.db sqlpage/sqlpage.db
EXPOSE 8080
CMD ["/usr/local/bin/sqlpage"]