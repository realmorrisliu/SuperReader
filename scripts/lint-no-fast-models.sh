#!/bin/sh
# Fail when tracked agent text names a fast model.
# Scans .cursor/, AGENTS.md, and docs/agents.
# The pstack rule's prohibition sentence is the only allow-listed text.
set -eu
cd "$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"

prohibition='Never pass a model id containing -fast or fast=true to any subagent; when a skill asks for a fast/cheap model, use composer-2.5 non-fast instead.'

list=$(mktemp)
trap 'rm -f "$list"' EXIT
git ls-files -- .cursor AGENTS.md docs/agents >"$list"

if ! awk -v prohibition="$prohibition" '
function hits(text) {
    return text ~ /-fast/ || text ~ /fast[ \t]*:[ \t]*true/ || text ~ /fast[ \t]*=[ \t]*true/
}
{
    path = $0
    n = 0
    while ((getline line < path) > 0) {
        n++
        rest = line
        while ((i = index(rest, prohibition)) > 0) {
            rest = substr(rest, 1, i - 1) substr(rest, i + length(prohibition))
        }
        if (hits(rest)) {
            print path ":" n ":" line
            status = 1
        }
    }
    close(path)
}
END { exit status + 0 }
' "$list" >"${list}.out"; then
    cat "${list}.out" >&2
    echo "lint-no-fast-models: fast model id in tracked agent text" >&2
    rm -f "${list}.out"
    exit 1
fi
rm -f "${list}.out"
echo "lint-no-fast-models: ok"
