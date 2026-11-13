#!/usr/bin/env bash
set -euo pipefail

version=$(cargo metadata --no-deps --locked --format-version 1 | jq -r '.packages[] | select(.name == "whois42d-ng") | .version')
architecture=$(dpkg --print-architecture)
export SOURCE_DATE_EPOCH
SOURCE_DATE_EPOCH=$(git log -1 --format=%ct)

cat > debian/changelog <<EOF
whois42d-ng (${version}-1) stable; urgency=medium

  * Release v${version}.

 -- Moraxyc <i@qaq.li>  $(date -u -R -d "@${SOURCE_DATE_EPOCH}")
EOF
# Rust is supplied by rustup, not the distribution package database.
dpkg-buildpackage --build=binary --no-sign -d
mkdir -p dist
cp "../whois42d-ng_${version}-1_${architecture}.deb" dist/
