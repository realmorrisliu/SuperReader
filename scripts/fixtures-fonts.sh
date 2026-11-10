#!/usr/bin/env bash
# OFL test fonts for the typesetter. Not committed. curl and shasum only.
set -euo pipefail

dest="${SPIKE_FIXTURES:-/tmp/superreader-fixtures}"
mkdir -p "$dest"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

# EB Garamond has no GitHub Release asset. v0.016 is sources, and the nightly
# zip is replaced in place. These are the published TTFs at that commit.
eb_commit="106a4a6d377987459ae5e68673a4570f13b957fb"
eb_base="https://raw.githubusercontent.com/octaviopardo/EBGaramond12/${eb_commit}/fonts/ttf"
# Noto Serif CJK SC Regular from the Serif2.003 release tag.
noto_url="https://raw.githubusercontent.com/notofonts/noto-cjk/Serif2.003/Serif/OTF/SimplifiedChinese/NotoSerifCJKsc-Regular.otf"

fetch() {
    local name="$1" url="$2" sum="$3"
    local out="$dest/$name"
    if [[ -f "$out" ]] && printf '%s  %s\n' "$sum" "$out" | shasum -a 256 -c - >/dev/null; then
        return 0
    fi
    curl -fsSL -o "$tmp/$name" "$url"
    printf '%s  %s\n' "$sum" "$tmp/$name" | shasum -a 256 -c -
    mv "$tmp/$name" "$out"
}

fetch "EBGaramond-Regular.ttf" \
    "${eb_base}/EBGaramond-Regular.ttf" \
    "2028dc06d3c130b4761693481436a32a8e35ed500bf58c25c53de004106125b8"
fetch "EBGaramond-Italic.ttf" \
    "${eb_base}/EBGaramond-Italic.ttf" \
    "d4ad1d0a9390d26d6d3f176117a1d121441edb8c8632bfca52cfad084b9059cb"
fetch "NotoSerifCJKsc-Regular.otf" \
    "$noto_url" \
    "2a2eae2628df83556c54018c41e20fa532c1b862c5256ae8b3f23feb918d12ca"

echo "fixture fonts $dest"
