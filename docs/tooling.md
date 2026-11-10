# Tooling

SuperReader is moving toward **Rust and shell** for repo automation. **`just`** at the repository root is the supported entry point for common tasks. The Cargo workspace stays under **`spikes/`** in Phase 1 so open shell/verify PRs do not conflict.

## `just` recipes

All `cargo` recipes use `--manifest-path spikes/Cargo.toml`. Build artifacts live under **`spikes/target/`**.

| Recipe | Action |
|--------|--------|
| `just build` | Release `spike-shell` |
| `just test` | Headless lib tests (`spike-report`, `spike-epub`) |
| `just fmt` / `just fmt-check` | `cargo fmt -p spike-report` only (Phase 1) |
| `just lint` | `clippy` on `spike-report` (`-D warnings`) |
| `just doc` | `cargo doc` with `RUSTDOCFLAGS=-D warnings` |
| `just deny` | `cargo deny check` on `spikes/Cargo.lock` |
| `just verify` | Serialized `scripts/verify` (see below; macOS UI harness) |
| `just clean-worktrees` | `scripts/cleanup-worktrees.sh` |

Requires Rust **1.98.1** and, on macOS for full verify, NSH **0.4.0** plus fixtures from `spikes/fetch-fixtures.sh`.

### Verify serialization (morris-mac)

`just verify` wraps `scripts/verify` with a **machine-wide lock** at `/tmp/superreader-verify.lock` (`mkdir`—only one holder). The lock directory records **pid**, **git branch**, and **UTC start time**. Waiters print `waiting for verify lock (held by …)` every 30s. A lock whose pid is no longer running is removed automatically. `EXIT`, `INT`, and `TERM` release the lock.

After acquiring the lock, the recipe waits until the **1-minute load average** is below **8** (override with `SR_VERIFY_MAX_LOAD`), using `sysctl -n vm.loadavg` on macOS or `/proc/loadavg` elsewhere, and prints the load when verify actually starts.

Do not invoke `scripts/verify` directly; always use **`just verify`** so concurrent agents queue instead of stacking UI workloads.

## Python in the repository

| File | Lines | Purpose | Replacement | Keep? |
|------|------:|---------|-------------|-------|
| `scripts/verify` | 3518 | macOS UI driver: doctor, builds, window/reading/chrome checks, artifact capture | **Phase 2:** shell + small Rust helpers (xtask) in incremental steps; same behavior as today | **Keep** until migration; do not add new Python |
| `spikes/epub/fixtures/build_books.py` | 186 | Build minimal EPUB fixtures with Literata for epub crate tests | **Keep** (fixture generator; low churn) or **Phase 2:** Rust in `xtask` | **Keep** for Phase 1 |

Policy: new automation must not be Python. Extend `just`, shell, or Rust (`xtask`) instead.

## Phase 2 layout and migration (after open PRs merge)

Planned once branches that touch `spikes/shell` and `scripts/verify` have landed (PRs #8, #10, #13, #14, #15, panel shell, Books-white, and related):

1. **Workspace root** — Optional hoist to repo-root `Cargo.toml` / `Cargo.lock`, `cargo fmt --all`, and any `scripts/verify` path updates, each with a verify run on morris-mac.
2. **Directory layout** — Move libraries to `crates/<name>`; move the NSH macOS app to `apps/macos`; keep `spikes/` only for time-boxed experiments worth retaining.
3. **Documentation per crate** — `README.md` in each crate directory plus a `//!` crate doc on the library root.
4. **CI expansion** — Every workspace crate in `fmt`, `clippy -D warnings`, `test`, and `doc` after clippy is fixed on `spike-shell`, `spike-pdf`, and `spike-epub`.
5. **`scripts/verify` migration** — One verifiable slice per PR: doctor and subprocess spawn in shell; parsing of `webkit.json` / summaries in Rust.
6. **Fixture build** — Optional: fold `build_books.py` into xtask.

Phase 1 touches **no** files under `spikes/` or `scripts/verify`.

## CI scope (Phase 1)

GitHub Actions uses **`working-directory: spikes`** (deny runs from repo root with `--manifest-path spikes/Cargo.toml`):

| Job | Crates / scope |
|-----|----------------|
| `fmt` | **`spike-report` only** (`cargo fmt -p spike-report --check`) |
| `clippy` | **`spike-report` only** (`-D warnings`) |
| `test` | **`spike-report`**, **`spike-epub`** |
| `doc` | **`spike-report`**, **`spike-epub`** (`RUSTDOCFLAGS=-D warnings`) |
| `deny` | Full `spikes/Cargo.lock` (licenses, advisories, bans, sources) |

**Excluded from clippy / fmt sweep (Phase 1)**

| Crate | Why |
|-------|-----|
| `spike-epub` | Open typesetter PRs (#10, #13); clippy fixes collide |
| `spike-shell` | Open chrome/shell PRs (#8, #14, #15, panel); GUI + NSH |
| `spike-pdf` | pdfium dylib for integration tests |
| `spike-measure` | Binary-only benchmark harness |

**Excluded from lib `test` / `doc`**: same as clippy except `spike-epub` stays in test/doc (passes today).
