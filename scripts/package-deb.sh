#!/usr/bin/env bash
set -euo pipefail

VERSION="0.1.0"
ARCH="amd64"
PKG_DIR="/tmp/deskstamp_${VERSION}_${ARCH}"

rm -rf "$PKG_DIR"
mkdir -p "$PKG_DIR/DEBIAN"
mkdir -p "$PKG_DIR/usr/bin"
mkdir -p "$PKG_DIR/usr/share/applications"
mkdir -p "$PKG_DIR/usr/share/metainfo"
mkdir -p "$PKG_DIR/usr/share/icons/hicolor/scalable/apps"

# Binary
cp target/release/deskstamp "$PKG_DIR/usr/bin/deskstamp"
strip "$PKG_DIR/usr/bin/deskstamp"
chmod 755 "$PKG_DIR/usr/bin/deskstamp"

# Desktop & Metainfo
cp data/io.github.gabrielbaiano.Deskstamp.desktop "$PKG_DIR/usr/share/applications/"
cp data/io.github.gabrielbaiano.Deskstamp.metainfo.xml "$PKG_DIR/usr/share/metainfo/"

# Icons
cp data/icons/hicolor/scalable/apps/io.github.gabrielbaiano.Deskstamp.svg "$PKG_DIR/usr/share/icons/hicolor/scalable/apps/"
for size in 16 24 32 48 64 128 256 512; do
  mkdir -p "$PKG_DIR/usr/share/icons/hicolor/${size}x${size}/apps"
  cp "data/icons/hicolor/${size}x${size}/apps/io.github.gabrielbaiano.Deskstamp.png" "$PKG_DIR/usr/share/icons/hicolor/${size}x${size}/apps/"
done

# DEBIAN/control
cat << CONTROL > "$PKG_DIR/DEBIAN/control"
Package: deskstamp
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${ARCH}
Maintainer: GabrielBaiano <gabrielngama@gmail.com>
Description: Lightweight Wayland screen watermark overlay
 Deskstamp is a native Wayland desktop watermark overlay designed
 for Pop!_OS COSMIC and other modern Wayland compositors.
CONTROL

dpkg-deb --build --root-owner-group "$PKG_DIR" "deskstamp_${VERSION}_${ARCH}.deb"
echo "Built deskstamp_${VERSION}_${ARCH}.deb successfully!"
