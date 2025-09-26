FROM --platform=$BUILDPLATFORM rust:1.90-slim AS builder
WORKDIR /usr/src/sqlpage
ARG TARGETARCH
ARG BUILDARCH
RUN apt-get update && \
    if [ "$TARGETARCH" = "$BUILDARCH" ]; then \
        rustup target list --installed > TARGET && \
        echo gcc > LINKER && \
        apt-get install -y gcc libgcc-s1 cmake unixodbc-dev && \
        cp /lib/*/libgcc_s.so.1 .; \
    elif [ "$TARGETARCH" = "arm64" ]; then \
        echo aarch64-unknown-linux-gnu > TARGET && \
        echo aarch64-linux-gnu-gcc > LINKER && \
        dpkg --add-architecture arm64 && apt-get update && \
        apt-get install -y gcc-aarch64-linux-gnu libgcc-s1-arm64-cross unixodbc-dev:arm64 && \
        cp /usr/aarch64-linux-gnu/lib/libgcc_s.so.1 .; \
    elif [ "$TARGETARCH" = "arm" ]; then \
        echo armv7-unknown-linux-gnueabihf > TARGET && \
        echo arm-linux-gnueabihf-gcc > LINKER && \
        dpkg --add-architecture armhf && apt-get update && \
        apt-get install -y gcc-arm-linux-gnueabihf libgcc-s1-armhf-cross cmake libclang1 unixodbc-dev:armhf && \
        cargo install --force --locked bindgen-cli && \
        echo "-I/usr/lib/gcc-cross/arm-linux-gnueabihf/12/include -I/usr/arm-linux-gnueabihf/include" > BINDGEN_EXTRA_CLANG_ARGS; \
        cp /usr/arm-linux-gnueabihf/lib/libgcc_s.so.1 .; \
    else \
        echo "Unsupported cross compilation target: $TARGETARCH"; \
        exit 1; \
    fi && \
    rustup target add $(cat TARGET) && \
    cargo init .

# Build dependencies (creates a layer that avoids recompiling dependencies on every build)
COPY Cargo.toml Cargo.lock ./
RUN BINDGEN_EXTRA_CLANG_ARGS=$(cat BINDGEN_EXTRA_CLANG_ARGS || true) \
    cargo build \
     --target $(cat TARGET) \
     --config target.$(cat TARGET).linker='"'$(cat LINKER)'"' \
     --profile superoptimized

# Build the project
COPY . .
RUN touch src/main.rs && \
    cargo build \
        --target $(cat TARGET) \
        --config target.$(cat TARGET).linker='"'$(cat LINKER)'"' \
        --profile superoptimized && \
    mv target/$(cat TARGET)/superoptimized/sqlpage sqlpage.bin && \
    mkdir -p deps && \
    ldd sqlpage.bin | awk '($3 ~ /^\//) {print $3} ($1 ~ /^\//) {print $1}' | sort -u | xargs -I '{}' cp --parents '{}' deps/

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
COPY --from=builder /usr/src/sqlpage/libgcc_s.so.1 /lib/libgcc_s.so.1
COPY --from=builder /usr/src/sqlpage/deps/ /
USER sqlpage
COPY --from=builder --chown=sqlpage:sqlpage /usr/src/sqlpage/sqlpage/sqlpage.db sqlpage/sqlpage.db
EXPOSE 8080
CMD ["/usr/local/bin/sqlpage"]