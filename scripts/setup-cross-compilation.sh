#!/bin/bash
set -euo pipefail

TARGETARCH="$1"
BUILDARCH="$2"
BINDGEN_EXTRA_CLANG_ARGS=""

apt-get update

if [ "$TARGETARCH" = "$BUILDARCH" ]; then
    TARGET="$(rustup target list --installed | head -n1)"
    LINKER="gcc"
    apt-get install -y gcc libgcc-s1 make
    LIBDIR="/lib/$(gcc -print-multiarch)"
elif [ "$TARGETARCH" = "arm64" ]; then
    TARGET="aarch64-unknown-linux-gnu"
    LINKER="aarch64-linux-gnu-gcc"
    apt-get install -y gcc-aarch64-linux-gnu libgcc-s1-arm64-cross make
    LIBDIR="/usr/aarch64-linux-gnu/lib"
elif [ "$TARGETARCH" = "arm" ]; then
    TARGET="armv7-unknown-linux-gnueabihf"
    LINKER="arm-linux-gnueabihf-gcc"
    apt-get install -y gcc-arm-linux-gnueabihf libgcc-s1-armhf-cross make cmake libclang-dev
    cargo install --force --locked bindgen-cli
    SYSROOT=$(arm-linux-gnueabihf-gcc -print-sysroot)
    BINDGEN_EXTRA_CLANG_ARGS="--sysroot=$SYSROOT -I$SYSROOT/usr/include -I$SYSROOT/usr/include/arm-linux-gnueabihf"
    LIBDIR="/usr/arm-linux-gnueabihf/lib"
else
    echo "Unsupported cross compilation target: $TARGETARCH"
    exit 1
fi

mkdir -p /tmp/sqlpage-libs
cp "$LIBDIR/libgcc_s.so.1" /tmp/sqlpage-libs/
rustup target add "$TARGET"

{
    echo "export TARGET='$TARGET'"
    echo "export LINKER='$LINKER'"
    [ -n "$BINDGEN_EXTRA_CLANG_ARGS" ] && printf "export BINDGEN_EXTRA_CLANG_ARGS=%q\n" "$BINDGEN_EXTRA_CLANG_ARGS"
} > /tmp/build-env.sh
