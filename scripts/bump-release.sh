#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:-}"
if [[ -z "$VERSION" ]]; then
  VERSION="$(grep -m1 '^version' Cargo.toml | sed -E 's/.*"([^"]+)".*/\1/')"
fi

if [[ ! "$VERSION" =~ ^v?[0-9]+\.[0-9]+\.[0-9]+.*$ ]]; then
  echo "Version must look like 0.1.0 or v0.1.0" >&2
  exit 1
fi

PLAIN="${VERSION#v}"
TAG="v$PLAIN"

# Update workspace version in Cargo.toml
tmp=$(mktemp)
awk -v ver="$PLAIN" '
  BEGIN{inpkg=0}
  /^\[workspace\.package\]/{inpkg=1}
  /^\[/{if($0!~/^\[workspace\.package\]/) inpkg=0}
  inpkg && /^version\s*=/{sub(/"[^"]+"/, "\"" ver "\"")}
  {print}
' Cargo.toml > "$tmp" && mv "$tmp" Cargo.toml

# Refresh lockfile metadata
cargo update -w >/dev/null 2>&1 || true

# Update scoop manifest template version if present
if [[ -f scoop/af.json ]]; then
  perl -0777 -i -pe "s/\"version\"\s*:\s*\"[^\"]+\"/\"version\": \"$PLAIN\"/g" scoop/af.json || true
fi

git add Cargo.toml Cargo.lock scoop/af.json 2>/dev/null || git add Cargo.toml
git commit -m "chore(release): $TAG"
git tag "$TAG"
git push origin HEAD
git push origin "$TAG"

echo "Released $TAG (commit and tag pushed)."
