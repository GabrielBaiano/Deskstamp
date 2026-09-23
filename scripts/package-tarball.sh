#!/usr/bin/env bash
set -euo pipefail

VERSION="v0.1.0"
TARGET="x86_64-unknown-linux-gnu"
PKG_NAME="deskstamp-${VERSION}-${TARGET}"
PKG_DIR="/tmp/${PKG_NAME}"

rm -rf "$PKG_DIR"
mkdir -p "$PKG_DIR/data"

cp target/release/deskstamp "$PKG_DIR/deskstamp"
strip "$PKG_DIR/deskstamp"
chmod 755 "$PKG_DIR/deskstamp"

cp -r data/* "$PKG_DIR/data/"
cp LICENSE "$PKG_DIR/LICENSE" 2>/dev/null || true

cat << 'INSTALL' > "$PKG_DIR/install.sh"
#!/usr/bin/env bash
set -euo pipefail

DEST_BIN="${HOME}/.local/bin"
DEST_APPS="${HOME}/.local/share/applications"
DEST_ICONS="${HOME}/.local/share/icons"

mkdir -p "$DEST_BIN" "$DEST_APPS"

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Installing deskstamp to $DEST_BIN..."
install -m755 "$DIR/deskstamp" "$DEST_BIN/deskstamp"

echo "Installing desktop launcher..."
install -m644 "$DIR/data/io.github.gabrielbaiano.Deskstamp.desktop" "$DEST_APPS/"

echo "Installing icons..."
cp -r "$DIR/data/icons/"* "$DEST_ICONS/"

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "$DEST_APPS" 2>/dev/null || true
fi

echo "Done! You can now run 'deskstamp' or launch Deskstamp from your application menu."
INSTALL
chmod +x "$PKG_DIR/install.sh"

cat << 'UNINSTALL' > "$PKG_DIR/uninstall.sh"
#!/usr/bin/env bash
set -euo pipefail

rm -f "${HOME}/.local/bin/deskstamp"
rm -f "${HOME}/.local/share/applications/io.github.gabrielbaiano.Deskstamp.desktop"
echo "Uninstalled Deskstamp."
UNINSTALL
chmod +x "$PKG_DIR/uninstall.sh"

tar -czf "${PKG_NAME}.tar.gz" -C "/tmp" "${PKG_NAME}"
echo "Built ${PKG_NAME}.tar.gz successfully!"
