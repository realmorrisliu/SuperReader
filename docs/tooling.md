# Tooling

SuperReader is moving toward **Rust and shell** for repo automation. **`just`** at the repository root is the supported entry point for common tasks. The Cargo workspace stays under **`spikes/`** in Phase 1 so open shell/verify PRs do not conflict.

## `just` recipes

All `cargo` recipes use `--manifest-path spikes/Cargo.toml`. Build artifacts live under **`spikes/target/`**.

| Recipe | Action |
|--------|--------|
| `just build` | Release `spike-shell` |
| `just test` | Headless lib tests (`spike-report`, `spike-epub`) plus `spike-bold-word-starts` |
| `just fmt` / `just fmt-check` | `cargo fmt` on `spike-report` and `spike-bold-word-starts` |
| `just lint` | Fast-model scan (`scripts/lint-no-fast-models.sh`), then `clippy -D warnings` on `spike-report` and `spike-bold-word-starts` |
| `just doc` | `cargo doc` with `RUSTDOCFLAGS=-D warnings` |
| `just bold-word-starts-ip` | Prefix-length tests plus the name and input scan |
| `just deny` | `cargo deny check` on `spikes/Cargo.lock` |
| `just verify` | Serialized `scripts/verify` (see below; macOS UI harness) |
| `just fixtures-fonts` | Checksummed OFL test fonts into `/tmp/superreader-fixtures` (not committed) |
| `just clean-worktrees` | `scripts/cleanup-worktrees.sh` |

Requires Rust **1.98.1** and, on macOS for full verify, NSH **0.5.0** plus fixtures from `spikes/fetch-fixtures.sh`.

### Verify serialization (morris-mac)

`just verify` runs `scripts/verify-lock.sh`, which wraps `scripts/verify` with a **machine-wide lock** at `/tmp/superreader-verify.lock` via **`mkdir`** (only one holder). The holder then writes **pid**, **git branch**, and **UTC start time** into that directory. Waiters print `waiting for verify lock (held by …)` every 30s. A lock whose pid is no longer running is removed automatically; a lock directory **without** pid metadata is removed only if it is older than 60s (covers the write window). Verify runs as a **background child**; `EXIT` releases the lock and returns verify’s status. `INT`/`TERM` forward to the child, wait for it, then release the lock and exit.

After acquiring the lock, the script waits until the **1-minute load average** is below **8** (override with `SR_VERIFY_MAX_LOAD`), printing `waiting for load < N (now X)` until then, using `sysctl -n vm.loadavg` on macOS or `/proc/loadavg` elsewhere. **`verify starting`** is printed once, when the load gate passes and verify is about to run. That 1-minute figure is `load_start` in `summary.json`. `load_end` is the 1-minute figure when the summary is written.

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
| `fmt` | **`spike-report`**, **`spike-bold-word-starts`** |
| `clippy` | **`spike-report`** (`--lib`), **`spike-bold-word-starts`** (`--all-targets`, `-D warnings`) |
| `test` | **`spike-report`**, **`spike-epub`** (`--lib`), **`spike-bold-word-starts`** (lib and integration tests) |
| `doc` | **`spike-report`**, **`spike-epub`**, **`spike-bold-word-starts`** (`RUSTDOCFLAGS=-D warnings`) |
| `deny` | Full `spikes/Cargo.lock` (licenses, advisories, bans, sources) |
| `no-fast-models` | `scripts/lint-no-fast-models.sh` on tracked `.cursor/`, `AGENTS.md`, and `docs/agents` |
| `bold-word-starts-ip` | `scripts/lint-bold-word-starts.sh` |

**Excluded from clippy / fmt sweep (Phase 1)**

| Crate | Why |
|-------|-----|
| `spike-epub` | Open typesetter PRs (#10, #13); clippy fixes collide |
| `spike-shell` | Open chrome/shell PRs (#8, #14, #15, panel); GUI + NSH |
| `spike-pdf` | pdfium dylib for integration tests |
| `spike-measure` | Binary-only benchmark harness |

**Excluded from lib `test` / `doc`**: same as clippy except `spike-epub` stays in test/doc (passes today).
