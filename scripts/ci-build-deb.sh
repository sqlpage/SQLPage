#!/bin/bash
set -euo pipefail

VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
sed -i "1s/.*/sqlpage ($VERSION-1) unstable; urgency=medium/" debian/changelog

dpkg-buildpackage -us -uc -b -d

mkdir -p build-output
mv ../*.deb build-output/
mv ../*.changes build-output/ 2>/dev/null || true
mv ../*.buildinfo build-output/ 2>/dev/null || true

lintian --no-tag-display-limit build-output/*.deb || true
dpkg-deb --contents build-output/*.deb
dpkg-deb --info build-output/*.deb

echo "✓ DEB package built successfully"
