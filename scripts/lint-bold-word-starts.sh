#!/bin/sh
# Fail when Bold Word Starts copies the claimed preset or its product names.
set -eu
cd "$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"

rule=.cursor/rules/bold-word-starts-ip.mdc
names=$(awk '/^banned-names:/ { $1 = ""; sub(/^ /, ""); print; exit }' "$rule")
if [ -z "$names" ]; then
    echo "bold-word-starts-ip: banned-names line missing from $rule" >&2
    exit 1
fi
pattern=$(printf '%s\n' "$names" | tr ' ' '|')

if grep -R -I -i -n -E "$pattern" . \
    --exclude-dir=.git \
    --exclude-dir=target \
    --exclude-dir=.nsh \
    --exclude=bold-word-starts-ip.mdc; then
    echo "bold-word-starts-ip: banned name outside $rule" >&2
    exit 1
fi

src=spikes/bold-word-starts/src/lib.rs
if grep -E -i -n 'freq|period|zipf' "$src"; then
    echo "bold-word-starts-ip: frequency or period input in $src" >&2
    exit 1
fi

probes=spikes/bold-word-starts/tests/probes.rs
ip=spikes/bold-word-starts/tests/ip.rs
grep -q '1, 2, 2, 2, 3, 2, 3, 4, 4, 5, 5, 6, 6, 7, 7' "$probes"
grep -q 'count \* 5, len \* 2' "$ip"
grep -q 'count \* 5, len \* 3' "$ip"
grep -q 'uniform_one_then_two_table' "$ip"

cargo test --manifest-path spikes/Cargo.toml -p spike-bold-word-starts
