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
    scripts/verify {{ARGS}}

clean-worktrees *ARGS:
    scripts/cleanup-worktrees.sh {{ARGS}}
