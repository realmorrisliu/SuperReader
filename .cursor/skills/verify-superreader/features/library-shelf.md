# Library shelf

S1 removed the bare shelf. The checks below are deleted with that screen. Do not retarget them at the card, and do not bring the shelf strings back to make a row pass. `spikes/library` stays. The Full form is scored from `features/shell.md`.

## Sub-features

- `shelf-empty`, `shelf-list`, `shelf-open`, `shelf-open-pdf`, `shelf-switch`, `shelf-streaming`, `shelf-calibre`, and `shelf-denied` are removed.
- `shelf-window-empty`, `shelf-window-epub`, `shelf-window-pdf`, and `shelf-window-switch` are removed.
- `smoke-launch`, `smoke-open`, and `smoke-turn` were the shelf subset's PDF smoke. They are not the Full-form smoke.
- `open-pdf-from-library` stays pending. The reading checks still open a book with `SPIKE_MODE`.

## How to get to it (user POV)

- Launch opens the Full form. There is no **Drop a book to open it** screen and no **Open** button.
- A reader-forcing `SPIKE_*` launch still opens the existing reader. That path is not the shelf.

## Driving it with scripts/verify

Preconditions:

- Do not add the removed ids back into `NSH_CASES`.
- A missing shelf row is the removal. It is not a failure to retarget.

## Gotchas

- `shelf-empty` does not pass the empty card. The empty card is the three peers in the Full form.
- `shell-cover-grid` stays pending until S2.
