#!/bin/bash
set -euo pipefail

TARGETARCH="$1"
BUILDARCH="$2"

if [ "$TARGETARCH" = "$BUILDARCH" ]; then
    # Native build
    rustup target list --installed > /tmp/TARGET
    echo "gcc" > /tmp/LINKER
    apt-get install -y gcc libgcc-s1 cmake pkg-config

    LIBDIR="/lib/$(gcc -print-multiarch)"

elif [ "$TARGETARCH" = "arm64" ]; then
    echo "aarch64-unknown-linux-gnu" > /tmp/TARGET
    echo "aarch64-linux-gnu-gcc" > /tmp/LINKER

    apt-get update
    apt-get install -y gcc-aarch64-linux-gnu libgcc-s1-arm64-cross pkg-config

    LIBDIR="/usr/aarch64-linux-gnu/lib"

    # Also install gcc for aarch64 to get libgcc
    apt-get install -y gcc-aarch64-linux-gnu

elif [ "$TARGETARCH" = "arm" ]; then
    echo "armv7-unknown-linux-gnueabihf" > /tmp/TARGET
    echo "arm-linux-gnueabihf-gcc" > /tmp/LINKER

    apt-get update
    apt-get install -y gcc-arm-linux-gnueabihf libgcc-s1-armhf-cross cmake libclang1 clang pkg-config

    # Ensure gcc-arm-linux-gnueabihf is available
    apt-get install -y gcc-arm-linux-gnueabihf

    cargo install --force --locked bindgen-cli

    SYSROOT=$(arm-linux-gnueabihf-gcc -print-sysroot)
    echo "--sysroot=$SYSROOT -I$SYSROOT/usr/include -I$SYSROOT/usr/include/arm-linux-gnueabihf" > /tmp/BINDGEN_EXTRA_CLANG_ARGS

    LIBDIR="/usr/arm-linux-gnueabihf/lib"
else
    echo "Unsupported cross compilation target: $TARGETARCH"
    exit 1
fi

# Copy libgcc_s.so.1 for runtime
mkdir -p /tmp/sqlpage-libs

cp "$LIBDIR/libgcc_s.so.1" /tmp/sqlpage-libs/

# Add the target
rustup target add "$(cat /tmp/TARGET)"

