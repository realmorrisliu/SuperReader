# SuperReader task entry point (see docs/tooling.md).

manifest := "spikes/Cargo.toml"

set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

default:
    @just --list

build:
    cargo build --manifest-path {{manifest}} -p spike-shell --release

test:
    cargo test --manifest-path {{manifest}} -p spike-report -p spike-epub --lib

fmt:
    cargo fmt --manifest-path {{manifest}} -p spike-report

fmt-check:
    cargo fmt --manifest-path {{manifest}} -p spike-report -- --check

lint:
    cargo clippy --manifest-path {{manifest}} -p spike-report --lib -- -D warnings

doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path {{manifest}} -p spike-report -p spike-epub --no-deps

deny:
    cd spikes && cargo deny --config ../deny.toml check

verify *ARGS:
    @bash -eu -o pipefail -c '\
    ARGS=("$@"); \
    LOCK_DIR=/tmp/superreader-verify.lock; \
    MAX_LOAD="${SR_VERIFY_MAX_LOAD:-8}"; \
    one_min_load() { \
    if sysctl -n vm.loadavg &>/dev/null; then \
    sysctl -n vm.loadavg | tr -d "{}" | awk "{print \$1}"; \
    else awk "{print \$1}" /proc/loadavg; fi; }; \
    lock_holder() { \
    local pid branch started; \
    pid=$(cat "$LOCK_DIR/pid" 2>/dev/null || echo "?"); \
    branch=$(cat "$LOCK_DIR/branch" 2>/dev/null || echo "?"); \
    started=$(cat "$LOCK_DIR/started" 2>/dev/null || echo "?"); \
    echo "pid ${pid} branch ${branch} since ${started}"; }; \
    clear_stale_lock() { \
    [[ ! -d "$LOCK_DIR" ]] && return 1; \
    if [[ ! -f "$LOCK_DIR/pid" ]]; then \
    echo "clearing verify lock without pid metadata"; rm -rf "$LOCK_DIR"; return 0; fi; \
    local holder; holder=$(cat "$LOCK_DIR/pid"); \
    if kill -0 "$holder" 2>/dev/null; then return 1; fi; \
    echo "clearing stale verify lock (pid ${holder})"; rm -rf "$LOCK_DIR"; return 0; }; \
    release_lock() { \
    [[ -d "$LOCK_DIR" ]] && [[ -f "$LOCK_DIR/pid" ]] && [[ "$(cat "$LOCK_DIR/pid")" == "$$" ]] && rm -rf "$LOCK_DIR"; }; \
    trap release_lock EXIT INT TERM; \
    while true; do \
    clear_stale_lock && continue; \
    if mkdir "$LOCK_DIR" 2>/dev/null; then \
    branch=$(git branch --show-current 2>/dev/null || echo unknown); \
    printf "%s\n" "$$" >"$LOCK_DIR/pid"; \
    printf "%s\n" "$branch" >"$LOCK_DIR/branch"; \
    date -u +%Y-%m-%dT%H:%M:%SZ >"$LOCK_DIR/started"; break; fi; \
    echo "waiting for verify lock (held by $(lock_holder))"; sleep 30; \
    done; \
    while true; do \
    load=$(one_min_load); \
    echo "verify starting: 1m loadavg=${load} (max ${MAX_LOAD})"; \
    awk -v l="$load" -v m="$MAX_LOAD" "BEGIN { exit (l+0<m+0)?0:1 }" && break; \
    sleep 10; done; \
    if [[ "${SR_VERIFY_STUB:-}" == "1" ]]; then \
    echo "verify stub: scripts/verify ${ARGS[*]:-}"; sleep 2; exit 0; fi; \
    exec scripts/verify "${ARGS[@]}"' -- {{ARGS}}

clean-worktrees *ARGS:
    scripts/cleanup-worktrees.sh {{ARGS}}
