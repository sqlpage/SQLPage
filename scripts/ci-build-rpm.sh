#!/bin/bash
set -euo pipefail

rpmdev-setuptree

VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
RPM_VERSION=$(echo "$VERSION" | sed 's/-/~/')

sed -i "s/^Version:.*/Version:        ${RPM_VERSION}/" rpm/sqlpage.spec
cp rpm/sqlpage.spec ~/rpmbuild/SPECS/

git archive --format=tar.gz --prefix="SQLPage-${RPM_VERSION}/" \
    -o ~/rpmbuild/SOURCES/v${RPM_VERSION}.tar.gz HEAD

rpmbuild -ba --nodeps ~/rpmbuild/SPECS/sqlpage.spec

rpmlint ~/rpmbuild/RPMS/*/sqlpage*.rpm || true
rpmlint ~/rpmbuild/SRPMS/sqlpage*.rpm || true
rpm -qilp ~/rpmbuild/RPMS/*/sqlpage*.rpm

echo "✓ RPM package built successfully"
