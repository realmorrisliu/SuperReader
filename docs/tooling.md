# Tooling

SuperReader is moving toward **Rust and shell** for repo automation. **`just`** at the repository root is the supported entry point for common tasks.

## `just` recipes

| Recipe | Action |
|--------|--------|
| `just build` | Release `spike-shell` |
| `just test` | Headless lib tests (`spike-report`, `spike-epub`) |
| `just fmt` / `just fmt-check` | `cargo fmt` |
| `just lint` | `clippy` on `spike-report` (`-D warnings`) |
| `just doc` | `cargo doc` with `RUSTDOCFLAGS=-D warnings` |
| `just deny` | `cargo deny check` (licenses + advisories) |
| `just verify` | `scripts/verify` (macOS UI harness) |
| `just clean-worktrees` | `scripts/cleanup-worktrees.sh` |

Requires Rust **1.98.1** (`rust-toolchain.toml`), and on macOS full verify needs NSH **0.4.0** plus fixtures from `spikes/fetch-fixtures.sh`.

## Python in the repository

| File | Lines | Purpose | Replacement | Keep? |
|------|------:|---------|-------------|-------|
| `scripts/verify` | 3518 | macOS UI driver: doctor, builds, window/reading/chrome checks, artifact capture | **Phase 2:** shell + small Rust helpers (xtask) in incremental steps; same behavior as today | **Keep** until migration; do not add new Python |
| `spikes/epub/fixtures/build_books.py` | 186 | Build minimal EPUB fixtures with Literata for epub crate tests | **Keep** (fixture generator; low churn) or **Phase 2:** Rust in `xtask` | **Keep** for Phase 1 |

Policy: new automation must not be Python. Extend `just`, shell, or Rust (`xtask`) instead.

## Phase 2 layout and migration (after open PRs merge)

Planned once branches that touch `spikes/shell` and `scripts/verify` have landed (PRs #8, #10, #13, #14, #15, panel shell, and related):

1. **Directory layout** — Move libraries to `crates/<name>`; move the NSH macOS app to `apps/macos` (no Swift shell today); keep `spikes/` only for time-boxed experiments worth retaining. Root `Cargo.toml` workspace inherits versions, `repository`, and lints.
2. **Documentation per crate** — `README.md` in each crate directory plus a `//!` crate doc on the library root.
3. **CI expansion** — Bring **every** workspace crate into `fmt`, `clippy -D warnings`, `test`, and `doc` after fixing clippy in `spike-shell`, `spike-pdf`, and `spike-epub` without conflicting with spike PRs.
4. **`scripts/verify` migration** — One verifiable slice per PR: doctor and subprocess spawn in shell; parsing of `webkit.json` / summaries in Rust; preserve timing semantics and `docs/verify` output shape.
5. **Fixture build** — Optional: fold `build_books.py` into xtask.

No crate moves in Phase 1 to avoid colliding with in-flight shell and verify work.

## CI scope (Phase 1)

GitHub Actions runs from the repository root:

| Job | Crates / scope |
|-----|----------------|
| `fmt` | Whole workspace (`cargo fmt --all --check`) |
| `clippy` | **`spike-report` only** (`-D warnings`) |
| `test` | **`spike-report`**, **`spike-epub`** |
| `doc` | **`spike-report`**, **`spike-epub`** (`RUSTDOCFLAGS=-D warnings`) |
| `deny` | Full `Cargo.lock` (licenses, advisories, bans, sources) |

**Excluded from clippy (Phase 1)**

| Crate | Why |
|-------|-----|
| `spike-epub` | Open typesetter PRs (#10, #13); known `clippy::unnecessary-to-owned` without fixing here |
| `spike-shell` | Open chrome/shell PRs (#8, #14, #15, panel); GUI + NSH; large surface |
| `spike-pdf` | Needs pdfium dylib for tests and integration |
| `spike-measure` | Binary-only benchmark harness |

**Excluded from lib `test` / `doc`**: same as clippy except `spike-epub` stays in test/doc (clean today).
