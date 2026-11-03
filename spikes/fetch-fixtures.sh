#!/bin/sh
# Downloads the public fixtures and the pdfium dylib the spike measures.
set -eu
root="$(cd "$(dirname "$0")/.." && pwd)"
dest="${SPIKE_FIXTURES:-/tmp/superreader-fixtures}"
pdfium="${SPIKE_PDFIUM:-/tmp/pdfium}"
mkdir -p "$dest" "$pdfium"
curl -fsSL -o "$dest/gpt3.pdf" "https://arxiv.org/pdf/2005.14165"
curl -fsSL -A "Mozilla/5.0" -o "$dest/pride-se.epub" \
  "https://standardebooks.org/ebooks/jane-austen/pride-and-prejudice/downloads/jane-austen_pride-and-prejudice.epub?source=download"
curl -fsSL -o "$dest/zhaohua.epub" "https://www.gutenberg.org/ebooks/25271.epub.noimages"
curl -fsSL -L -o "$pdfium/pdfium-mac-arm64.tgz" \
  "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F8125/pdfium-mac-arm64.tgz"
tar -xzf "$pdfium/pdfium-mac-arm64.tgz" -C "$pdfium"
echo "fixtures $dest"
echo "pdfium $pdfium/lib/libpdfium.dylib"
