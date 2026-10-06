#!/bin/bash
# Script for local macOS signing and notarization testing
#
# Prerequisites:
#   1. Build the frontend assets first: npm ci && npm run build
#   2. Install the Rust target: rustup target add aarch64-apple-darwin
#   3. Import your Developer ID certificate into the login keychain:
#      security import sqlpage.p12 -k ~/Library/Keychains/login.keychain-db -P "<p12-password>" -T /usr/bin/codesign
#
# Required environment variables for notarization:
#   APPLE_SIGNING_IDENTITY          - e.g. "Developer ID Application: Your Name (TEAMID)"
#   APPLE_NOTARIZATION_APPLE_ID     - Your Apple ID email
#   APPLE_NOTARIZATION_PASSWORD     - App-specific password
#   APPLE_NOTARIZATION_TEAM_ID      - Your 10-character Team ID

set -euo pipefail

# Check if we're on macOS
if [[ "$(uname)" != "Darwin" ]]; then
    echo "This script must be run on macOS"
    exit 1
fi

# Check if required tools are available (use xcrun --find as tools may not be on PATH)
if ! xcrun --find codesign &> /dev/null; then
    echo "codesign not found. Please install Xcode command line tools:"
    echo "  xcode-select --install"
    exit 1
fi

if ! xcrun --find notarytool &> /dev/null; then
    echo "notarytool not found. Please install Xcode command line tools:"
    echo "  xcode-select --install"
    exit 1
fi

if ! xcrun --find stapler &> /dev/null; then
    echo "stapler not found. Please install Xcode command line tools:"
    echo "  xcode-select --install"
    exit 1
fi

# Check that the signing identity is set
if [[ -z "${APPLE_SIGNING_IDENTITY:-}" ]]; then
    echo "APPLE_SIGNING_IDENTITY is not set."
    echo "Example: export APPLE_SIGNING_IDENTITY=\"Developer ID Application: Your Name (TEAMID)\""
    exit 1
fi

# Check that frontend assets exist (build.rs requires them)
if [[ ! -f frontend/dist/tabler-sprite.svg ]]; then
    echo "Frontend assets not found. Building them now..."
    npm ci
    npm run build
fi

# Install the ARM target if on Intel Mac
if [[ "$(uname -m)" == "x86_64" ]]; then
    echo "Intel Mac detected, ensuring aarch64-apple-darwin target is installed..."
    rustup target add aarch64-apple-darwin || true
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
codesign --force --options runtime --entitlements .github/macos/entitlements.plist --sign "$APPLE_SIGNING_IDENTITY" --timestamp "$BINARY_PATH"

# Verify the signature
echo "Verifying the signature..."
codesign --verify --deep --strict --verbose=2 "$BINARY_PATH"

# Create a zip archive for notarization
echo "Creating zip archive for notarization..."
ditto -c -k --keepParent "$BINARY_PATH" sqlpage-macos.zip

# Notarize the binary (if credentials are provided)
if [[ -n "${APPLE_NOTARIZATION_APPLE_ID:-}" && -n "${APPLE_NOTARIZATION_PASSWORD:-}" && -n "${APPLE_NOTARIZATION_TEAM_ID:-}" ]]; then
    echo "Submitting for notarization..."
    xcrun notarytool submit sqlpage-macos.zip \
        --apple-id "$APPLE_NOTARIZATION_APPLE_ID" \
        --password "$APPLE_NOTARIZATION_PASSWORD" \
        --team-id "$APPLE_NOTARIZATION_TEAM_ID" \
        --wait

    # Staple the notarization ticket to the zip (stapler requires a container format)
    echo "Stapling the notarization ticket..."
    xcrun stapler staple sqlpage-macos.zip

    # Extract the stapled binary
    echo "Extracting stapled binary..."
    rm -rf stapled && mkdir stapled
    ditto -x -k sqlpage-macos.zip stapled/
    cp stapled/sqlpage "$BINARY_PATH"
    chmod +x "$BINARY_PATH"

    # Final verification including Gatekeeper assessment
    echo "Final verification..."
    codesign --verify --deep --strict --verbose=2 "$BINARY_PATH"
    spctl -a -v "$BINARY_PATH"
else
    echo "Skipping notarization. Set APPLE_NOTARIZATION_APPLE_ID, APPLE_NOTARIZATION_PASSWORD, and APPLE_NOTARIZATION_TEAM_ID to enable notarization."
    echo "Note: The binary is signed but not notarized. Gatekeeper will still show a warning."
fi

echo "macOS signing complete!"
