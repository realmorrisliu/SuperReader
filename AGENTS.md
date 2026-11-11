# Agent notes (SuperReader)

## Mac verify policy

Per pull request, run **`just verify-only <checks…>`** plus **`just smoke`** on Morris's Mac, not the full suite. Pick checks from the diff with [feature-map.json](.cursor/skills/verify-superreader/feature-map.json) and `scripts/verify-path-checks.sh`.

- **`just verify`** runs the full suite. Use it only before a **release tag**, not on every PR.
- **`just smoke`** runs about one minute of UI checks (launch, open an EPUB, turn a page, open a PDF).
- A **rebase with no code changes** may merge when CI is green. Do not re-run Mac verify for that PR.

Cloud agents must not take `/tmp/superreader-verify.lock` and must not target Morris's Mac for verify. Parse and plan with `python3 scripts/verify --dry-run …` on Linux instead.

Mac verify uses **one `spike-shell` process** per session (`SR_VERIFY_SESSION=1`, default). Presets advance in-process via `SPIKE_VERIFY_SESSION` and `/tmp/superreader-spike/verify-cmd.json`. `summary.json` includes `launch_count` (target **1** for spike-shell checks plus packaged-app launches for alpha).

Entry points always go through the lock wrapper (`just verify`, `just verify-only`, `just smoke`), never bare `scripts/verify` except `--doctor`, `--self-test`, and `--dry-run`.
