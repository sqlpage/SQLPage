#!/bin/bash
# Script for local macOS signing and notarization testing

set -euo pipefail

# Check if we're on macOS
if [[ "$(uname)" != "Darwin" ]]; then
    echo "This script must be run on macOS"
    exit 1
fi

# Check if required tools are available
if ! command -v codesign &> /dev/null; then
    echo "codesign not found. Please install Xcode command line tools."
    exit 1
fi

if ! command -v notarytool &> /dev/null; then
    echo "notarytool not found. Please install Xcode command line tools."
    exit 1
fi

if ! command -v stapler &> /dev/null; then
    echo "stapler not found. Please install Xcode command line tools."
    exit 1
fi

# Build the binary
echo "Building SQLPage for aarch64-apple-darwin..."
cargo build --profile superoptimized --locked --target aarch64-apple-darwin --features "odbc-static"

# Check if the binary exists
BINARY_PATH="target/aarch64-apple-darwin/superoptimized/sqlpage"
if [[ ! -f "$BINARY_PATH" ]]; then
    echo "Binary not found at $BINARY_PATH"
    exit 1
fi

# Sign the binary
echo "Signing the binary..."
codesign --force --options runtime --entitlements sqlpage.entitlements --sign "$APPLE_SIGNING_IDENTITY" --timestamp "$BINARY_PATH"

# Verify the signature
echo "Verifying the signature..."
codesign --verify --deep --strict --verbose=2 "$BINARY_PATH"

# Create a zip archive for notarization
echo "Creating zip archive for notarization..."
ditto -c -k --keepParent "$BINARY_PATH" sqlpage-macos.zip

# Notarize the binary
if [[ -n "${APPLE_NOTARIZATION_APPLE_ID:-}" && -n "${APPLE_NOTARIZATION_PASSWORD:-}" && -n "${APPLE_NOTARIZATION_TEAM_ID:-}" ]]; then
    echo "Submitting for notarization..."
    xcrun notarytool submit sqlpage-macos.zip \
        --apple-id "$APPLE_NOTARIZATION_APPLE_ID" \
        --password "$APPLE_NOTARIZATION_PASSWORD" \
        --team-id "$APPLE_NOTARIZATION_TEAM_ID" \
        --wait
    
    # Staple the notarization ticket
    echo "Stapling the notarization ticket..."
    xcrun stapler staple "$BINARY_PATH"
else
    echo "Skipping notarization. Set APPLE_NOTARIZATION_APPLE_ID, APPLE_NOTARIZATION_PASSWORD, and APPLE_NOTARIZATION_TEAM_ID to enable notarization."
fi

# Final verification
echo "Final verification..."
codesign --verify --deep --strict --verbose=2 "$BINARY_PATH"
spctl -a -v "$BINARY_PATH"

echo "macOS signing and notarization complete!"