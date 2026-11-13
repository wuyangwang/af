#!/usr/bin/env bash
set -euo pipefail

BIN="af"
VERSION="${1:-}"
TARGET="${2:-$(rustc -vV | sed -n 's/^host: //p')}"

if [[ -z "$VERSION" ]]; then
  VERSION="v$(grep -m1 '^version' Cargo.toml | sed -E 's/version = "([^"]+)"/\1/')"
fi

echo "Building $BIN $VERSION for $TARGET"
cargo build -p af-cli --release --target "$TARGET"

NAME="$BIN-$VERSION-$TARGET"
DIST="dist/$NAME"
mkdir -p "$DIST"

BIN_PATH="target/$TARGET/release/$BIN"
if [[ "$TARGET" == *windows* ]]; then
  BIN_PATH+=".exe"
fi

cp "$BIN_PATH" "$DIST/"
cp README.md LICENSE "$DIST/"

ARCHIVE="dist/$NAME.tar.gz"
if [[ "$TARGET" == *windows* ]]; then
  ARCHIVE="dist/$NAME.zip"
  (cd dist && zip -r "$(basename "$ARCHIVE")" "$NAME")
  (cd dist && shasum -a 256 "$(basename "$ARCHIVE")" > "$(basename "$ARCHIVE").sha256")
else
  tar -C dist -czf "$ARCHIVE" "$NAME"
  (cd dist && shasum -a 256 "$(basename "$ARCHIVE")" > "$(basename "$ARCHIVE").sha256")
fi

echo "Created $ARCHIVE"
cat "$ARCHIVE.sha256"
