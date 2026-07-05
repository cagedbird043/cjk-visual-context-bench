#!/usr/bin/env bash
set -euo pipefail

CACHE_DIR=${CACHE_DIR:-fonts/.cache}
mkdir -p fonts "$CACHE_DIR"

fetch_direct() {
  local id=$1 url=$2 sha=$3 target=$4
  local cache="$CACHE_DIR/${target##*/}.download"
  if [[ ! -f "$cache" ]]; then
    echo "download $id"
    curl -L --fail -o "$cache" "$url"
  fi
  echo "$sha  $cache" | sha256sum -c -
  cp "$cache" "$target"
}

fetch_zip_member() {
  local id=$1 url=$2 sha=$3 member=$4 target=$5
  local zip="$CACHE_DIR/${url##*/}"
  if [[ ! -f "$zip" ]]; then
    echo "download $id"
    curl -L --fail -o "$zip" "$url"
  fi
  echo "$sha  $zip" | sha256sum -c -
  unzip -p "$zip" "$member" > "$target"
}

fetch_direct \
  zpix \
  https://github.com/SolidZORO/zpix-pixel-font/releases/download/v3.1.11/zpix.ttf \
  ed39f02845e8c0b8cdba275432250fb03e8528826f058bc151753bd62b44b744 \
  fonts/zpix.ttf

fetch_zip_member \
  fusion-pixel-8px-monospaced-zh-hans \
  https://github.com/TakWolf/fusion-pixel-font/releases/download/2026.07.01/fusion-pixel-font-8px-monospaced-otf-v2026.07.01.zip \
  f886c9859419962da0b4abaa48c734da98d5ad99eac43f775c44673bab49773a \
  fusion-pixel-8px-monospaced-zh_hans.otf \
  fonts/fusion-pixel-8px-monospaced-zh_hans.otf

fetch_zip_member \
  fusion-pixel-10px-monospaced-zh-hans \
  https://github.com/TakWolf/fusion-pixel-font/releases/download/2026.07.01/fusion-pixel-font-10px-monospaced-otf-v2026.07.01.zip \
  44b59c230f6872ed64924745928cb5219400af82a042da5be66438763f029d40 \
  fusion-pixel-10px-monospaced-zh_hans.otf \
  fonts/fusion-pixel-10px-monospaced-zh_hans.otf

fetch_zip_member \
  fusion-pixel-12px-monospaced-zh-hans \
  https://github.com/TakWolf/fusion-pixel-font/releases/download/2026.07.01/fusion-pixel-font-12px-monospaced-otf-v2026.07.01.zip \
  f0f23f0d26cefd24c94457f9bc7df03bae6401df4ffe3b685b8410d38e75f336 \
  fusion-pixel-12px-monospaced-zh_hans.otf \
  fonts/fusion-pixel-12px-monospaced-zh_hans.otf

sha256sum fonts/zpix.ttf fonts/fusion-pixel-8px-monospaced-zh_hans.otf fonts/fusion-pixel-10px-monospaced-zh_hans.otf fonts/fusion-pixel-12px-monospaced-zh_hans.otf
