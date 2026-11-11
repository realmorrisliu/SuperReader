# Verify session architecture

## Grounding

`scripts/verify` today calls `launch()` dozens of times per full run. Each call starts a fresh `spike-shell`, applies a distinct `SPIKE_*` environment, waits for readiness, captures Accessibility and PNG evidence, scores, then kills the process. `run_reading` alone loops trims and books and accounts for most launches. NSH `nsh test` in `shell.test.tsx` covers four headless tree checks only. Pixel and footer checks need the real window per `docs/nsh.md` and the verify-superreader skill.

## Candidates

**A. AppleScript keystrokes into one live process.** Reuse the spike’s digit keys (`1` pdfium, `3` webkit, `8` pride, arrows for page). No Rust change. Fails for drag, reload, trim menu, fullscreen, bold probes, and cold-cache deletes that today depend on env-at-start or separate support dirs.

**B. File-polled `SPIKE_VERIFY_PLAN` executed inside spike-shell.** One or two processes per verify run. Python builds a JSON plan from selected checks. The shell applies each step in-process with the same state machine as startup env (refactored into `apply_preset`). Python scores artifacts after capture. Relaunch-only checks (cold open, warm reopen, packaged alpha reopen) run as a second plan with a fresh `SUPERREADER_SUPPORT_DIR`.

**C. New `verify.suite.tsx` under `nsh test`.** One headless open per suite. Cannot see pdfium bitmaps, WKWebView pixels, or measure raster floors.

## Synthesis decision

Ship **B**. It matches NSH 0.4’s model (Rust root, headless tests separate from window proof) and keeps `summary.json` plus PNG names unchanged. Phase 1 moves **capture** into the shell plan runner and leaves **scoring** in Python. Session count is **one** primary launch plus **one** optional relaunch plan for checks classified as `relaunch` in `feature-map.json`.

## Public surfaces

| Component | Responsibility |
|-----------|----------------|
| `feature-map.json` | `relaunch` check ids, runner membership, smoke set |
| `scripts/verify/plan.py` | Build ordered capture plans from selected check ids |
| `scripts/verify/session.py` | Run primary and relaunch plans; count launches |
| `spikes/shell/src/verify_plan.rs` | Parse plan, `apply_preset`, wait, write step artifacts |
| `scripts/verify` (existing) | Doctor, build, nsh test, measure, alpha app; score functions |

## Invariants

- `launch()` increments a global counter; session mode uses `run_verify_plan()` instead of per-check `launch()` for window capture steps.
- Pending rows and alpha packaged app keep their own process model until migrated.

## Launch count (spike-shell)

| Mode | Before (per full verify) | After (`SR_VERIFY_SESSION=1`) |
|------|---------------------------|-------------------------------|
| spike-shell | ~90 one-shot launches (`docs/verify/reading-slide.md` records the old pattern; `scripts/verify` had on the order of 23 launch call sites, many inside trim or timing loops) | **1** process per verify session |
| Slowpage.app (alpha) | same | unchanged (separate packaged binary per step that still uses `drive_launch` with `binary=`) |
| spike-measure | 5 subprocess runs | unchanged |

`summary.json` carries `launch_count` for the run.
