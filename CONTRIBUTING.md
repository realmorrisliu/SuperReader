# Contributing

SuperReader is in pre-alpha. Issues and PRs are welcome; expect churn in the spike shell and in `scripts/verify`.

## Build and test

Use **`just`** from the repository root ([`docs/tooling.md`](docs/tooling.md)):

- **Toolchain:** Rust **1.98.1** (`rust-toolchain.toml`).
- **Fixtures:** `spikes/fetch-fixtures.sh` before PDF or full-shell work.
- **Shell:** `just build`.
- **Headless tests:** `just test`.
- **Formatting / lint / doc / deny:** `just fmt`, `just lint`, `just doc`, `just deny`.

Do not add new Python tooling; extend `just`, shell, or Rust (`xtask`) instead.

NSH is pinned at **0.4.0** for the desktop shell and `nsh test`. See [docs/nsh.md](docs/nsh.md).

## Verification

Behavior changes should be justified with **`scripts/verify`**, which drives the real app (window chrome, reading layout, PDF/EPUB walks). At minimum, run the subset that covers your change, for example `scripts/verify --subset cache` for line-break cache work. Full runs belong in calm load conditions; see the verify skill under `.cursor/skills/verify-superreader/`.

Record substantive full-run results in `docs/verify/summary.json` when appropriate. Per-run details stay under `docs/verify/runs/` (gitignored except summaries you intentionally commit).

## Pull requests

Open a PR against `main`. Prefer **squash merge**. In the PR body:

- **Why** — user or agent-visible problem.
- **What changed** — concise technical summary.
- **Verification** — what you ran (`just test`, `just lint`, `just verify` subset or full, timings if relevant).

Keep diffs focused. Do not add AGPL or GPL dependencies (including the `epub` crate or MuPDF). `scripts/verify` enforces a `no-agpl-gpl` doctor check.

## Security

See [SECURITY.md](SECURITY.md).
