#!/bin/bash
set -euo pipefail

mkdir -p release-assets
cp debian-package/*.deb release-assets/
cp rpm-package/*.rpm release-assets/
cp srpm-package/*.rpm release-assets/

cd release-assets
sha256sum * > SHA256SUMS
cat SHA256SUMS

echo "✓ Release assets prepared"
ls -lh
