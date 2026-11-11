# SuperReader task entry point (see docs/tooling.md).

manifest := "spikes/Cargo.toml"

set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

default:
    @just --list

build:
    cargo build --manifest-path {{manifest}} -p spike-shell --release

test:
    cargo test --manifest-path {{manifest}} -p spike-report -p spike-epub --lib
    cargo test --manifest-path {{manifest}} -p spike-bold-word-starts

fmt:
    cargo fmt --manifest-path {{manifest}} -p spike-report -p spike-bold-word-starts

fmt-check:
    cargo fmt --manifest-path {{manifest}} -p spike-report -p spike-bold-word-starts -- --check

lint:
    cargo clippy --manifest-path {{manifest}} -p spike-report --lib -- -D warnings
    cargo clippy --manifest-path {{manifest}} -p spike-bold-word-starts --all-targets -- -D warnings

doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path {{manifest}} -p spike-report -p spike-epub -p spike-bold-word-starts --no-deps

bold-word-starts-ip:
    scripts/lint-bold-word-starts.sh

deny:
    cd spikes && cargo deny --config ../deny.toml check

verify *ARGS:
    @scripts/verify-lock.sh {{ARGS}}

icons-fetch:
    scripts/icons-fetch.sh

fixtures-fonts:
    scripts/fixtures-fonts.sh

clean-worktrees *ARGS:
    scripts/cleanup-worktrees.sh {{ARGS}}

app:
    scripts/package-macos.sh

dmg: app
    scripts/make-dmg.sh

install: app
    scripts/install-app.sh
