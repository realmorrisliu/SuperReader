# Contributing

SuperReader is in pre-alpha. Issues and PRs are welcome; expect churn in the spike shell and in `scripts/verify`.

## Build and test

- **Toolchain:** Rust **1.98.1** (`spikes/rust-toolchain.toml`).
- **Fixtures:** from `spikes/`, run `./fetch-fixtures.sh` before PDF or full-shell work.
- **Shell:** `cd spikes && cargo build -p spike-shell --release`.
- **Headless tests:** `cd spikes && cargo test -p spike-report -p spike-epub --lib`.
- **Formatting:** `cd spikes && cargo fmt --all`.

NSH is pinned at **0.4.0** for the desktop shell and `nsh test`. See [docs/nsh.md](docs/nsh.md).

## Verification

Behavior changes should be justified with **`scripts/verify`**, which drives the real app (window chrome, reading layout, PDF/EPUB walks). At minimum, run the subset that covers your change, for example `scripts/verify --subset cache` for line-break cache work. Full runs belong in calm load conditions; see the verify skill under `.cursor/skills/verify-superreader/`.

Record substantive full-run results in `docs/verify/summary.json` when appropriate. Per-run details stay under `docs/verify/runs/` (gitignored except summaries you intentionally commit).

## Pull requests

Open a PR against `main`. Prefer **squash merge**. In the PR body:

- **Why** — user or agent-visible problem.
- **What changed** — concise technical summary.
- **Verification** — what you ran (`cargo test`, `scripts/verify` subset or full, timings if relevant).

Keep diffs focused. Do not add AGPL or GPL dependencies (including the `epub` crate or MuPDF). `scripts/verify` enforces a `no-agpl-gpl` doctor check.

## Security

See [SECURITY.md](SECURITY.md).
