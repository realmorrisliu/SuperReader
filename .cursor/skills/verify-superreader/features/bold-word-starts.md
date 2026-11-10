# Bold Word Starts

Opt-in leading-letter emphasis. The spike builds it only when `SPIKE_BOLD_WORD_STARTS` is set. The default launch leaves the chapter DOM alone.

## Sub-features

- `bold-word-starts-ip` fails if a name listed in the IP rule appears outside that rule file.
- `bold-word-starts-count` runs the `bold_word_starts` unit tests. Lengths 1 through 4 are not the claimed short-word table, and lengths 5 through 30 are not a two-fifths or three-fifths ratio.

## How to get to it (user POV)

- The shipped control is not in this draft. `docs/design/bold-word-starts.md` names a View menu item, off by default. The comparison sheet is `docs/prototypes/bold-word-starts/compare-sheet.png`.
- A prototype launch sets `SPIKE_BOLD_WORD_STARTS` to `off`, `tiers`, `onset`, or `hybrid`.

## Driving it with scripts/verify

Preconditions:

- `just bold-word-starts-ip` is the check this draft adds. It does not launch the window.
- `scripts/verify --doctor` still passes with the variable unset.

- **Lint.** From the repo root, run `just bold-word-starts-ip`. The shell script exits 0, and `cargo test -p spike-report --lib bold_word_starts` exits 0.
- **Default path.** Launch Pride with `SPIKE_MODE=webkit` and without `SPIKE_BOLD_WORD_STARTS`. The chapter text is unchanged and the page count matches a build from before this draft.

## Gotchas

- The length rule lives in `tier_bold_count`. A second copy of the 1-to-12 table in JavaScript will drift. The page script embeds the Rust table.
- `SPIKE_BOLD_WORD_STARTS=preset` reads a count function from `SPIKE_BOLD_WORD_STARTS_FILE`. That file is a local comparison aid. Do not commit it.
- Screenshot runs share `/tmp/superreader-verify.lock` with `just verify`.
