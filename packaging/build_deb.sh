#!/usr/bin/env bash
# Build a simple Debian package for Knockraven.  This script depends on
# dpkg-deb being installed on the host system.  It builds the release
# binary using Cargo, lays out a minimal Debian package structure and
# invokes dpkg-deb to produce a .deb file in the current directory.

set -euo pipefail

VERSION=${1:-"0.1.0"}
ARCH=$(dpkg --print-architecture 2>/dev/null || echo "amd64")
PACKAGE_NAME="knockraven"

echo "Building release binary..."
cargo build --release

echo "Preparing package directory..."
rm -rf package
install -d package/usr/local/bin package/DEBIAN
install -m 0755 target/release/knockraven package/usr/local/bin/knockraven

cat > package/DEBIAN/control <<EOF
Package: ${PACKAGE_NAME}
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${ARCH}
Maintainer: John Hauger Mitander <john@on1.no>
Description: Multi-protocol port-knocking discovery tool for authorized security testing
EOF

echo "Building .deb..."
dpkg-deb --build package "${PACKAGE_NAME}_${VERSION}_${ARCH}.deb"

echo "Package created: ${PACKAGE_NAME}_${VERSION}_${ARCH}.deb"