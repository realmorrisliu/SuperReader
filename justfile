# SuperReader task entry point (see docs/tooling.md).

set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

default:
    @just --list

build:
    cargo build -p spike-shell --release

test:
    cargo test -p spike-report -p spike-epub --lib

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

lint:
    cargo clippy -p spike-report --lib -- -D warnings

doc:
    RUSTDOCFLAGS="-D warnings" cargo doc -p spike-report -p spike-epub --no-deps

deny:
    cargo deny check

verify *ARGS:
    scripts/verify {{ARGS}}

clean-worktrees *ARGS:
    scripts/cleanup-worktrees.sh {{ARGS}}
