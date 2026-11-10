#!/bin/sh
set -eu

cd "$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"

rule=.cursor/rules/bold-word-starts-ip.mdc
names=$(awk '/^banned-names:/ { for (i = 2; i <= NF; i++) printf "%s%s", (i == 2 ? "" : "|"), $i; print "" }' "$rule")
if [ -z "$names" ]; then
  echo "IP rule is missing banned-names" >&2
  exit 1
fi

hits=$(grep -R -i -n -E "$names" . \
  --exclude-dir=.git \
  --exclude-dir=target \
  --exclude-dir=.nsh \
  --exclude=bold-word-starts-ip.mdc \
  || true)

if [ -n "$hits" ]; then
  printf '%s\n' "$hits" >&2
  echo "bold-word-starts lint failed" >&2
  exit 1
fi

echo "bold-word-starts lint passed"
