# Bold word starts

This draft is a prototype. The chapter is unchanged unless `SPIKE_BOLD_WORD_STARTS` is set. The only user-facing name is `BOLD_WORD_STARTS_LABEL`, "Bold Word Starts" and "词首加粗".

The comparison sheet is `docs/prototypes/bold-word-starts/compare-sheet.png`. Full pages and 200% crops of the same opening paragraph are in that directory.

## Pages

The shots are the spike-shell WKWebView on macOS. Pride and Prejudice, chapter 1, is a white page in Iowan Old Style at 22px, in the paperback trim. 范愛農 is Songti SC at 20px. Every rule uses font-weight 600, so the sheet compares the rule.

`scripts/bold-word-starts-shots` takes `/tmp/superreader-verify.lock` before it opens the window, and it waits until the one-minute load average is under 8.

## Reference

The left column is the common preset, comparison only, not shipped. The count function lived in a file outside the repo, passed through `SPIKE_BOLD_WORD_STARTS_FILE`, and was deleted after the shot. The commit keeps the PNG, the 200% crop, and the measured counts in `reference-common-preset.json`.

On the opening sentence, 23 words, the reference bolds 40/23 letters per word, 1.74. The median is 1.

## Rules in this draft

Han gets a count of 0 from every rule. The page script bolds only runs of ASCII letters.

Letter tiers use `tier_bold_count`. Lengths 1 through 12 bold 1, 1, 2, 3, 4, 3, 3, 4, 4, 5, 5, 6 letters. From length 13 the count starts at ceil of `len * 45 / 100`. When that count is two fifths or three fifths of the length, it moves to a neighbor that is not. Lengths 1 through 4 are 1, 1, 2, 3. The unit test checks that sequence against the short-word table named in `.cursor/rules/bold-word-starts-ip.mdc`, and it checks lengths 5 through 30 against a fifth ratio. The count takes a length and a script. It does not take a word frequency.

First syllable uses `window.spikeSyllablePoints`, which returns the line breaker's Liang points. The bold count is the first point. A word of 9 letters or more uses the second point when one exists. A word with no point is bold through the last letter. Words shorter than 5 letters have no legal point, because the breaker keeps at least 2 letters before a point and 3 after.

Onset and nucleus counts the opening consonants plus the following vowel. `y` counts as a vowel after the first letter. A word that starts with a vowel has a count of 1.

## Comparison

Opening sentence of chapter 1, 23 words.

| Rule | Average bold letters | Median | Share equal to the reference |
| --- | --- | --- | --- |
| Common preset, comparison only, not shipped | 40/23, 1.74 | 1 | 23/23 |
| Letter tiers | 54/23, 2.35 | 2 | 11/23, 48% |
| First syllable | 71/23, 3.09 | 3 | 4/23, 17% |
| Onset and nucleus | 36/23, 1.57 | 1 | 16/23, 70% |

First syllable reads worse. "truth", "that", "good", "must", "want", and "wife" are bold for their whole length, because they have no hyphen point. Those words have no lighter tail, and the average is 71/23 against the reference 40/23.

Onset and nucleus reads worse on the long words. "universally" and "acknowledged" start with a vowel, so each bolds one letter. The 70% share comes from short words that happen to match. The long words, which carry the cue, do not.

Letter tiers still marks every Latin word with a prefix, including the long ones. It is heavier than the reference. Length 4 bolds 3 letters and length 5 bolds 4, so "that" and "truth" look dark. Those counts stay off the short-word table and off a fifth ratio.

Keep letter tiers as the candidate to tune. Leave syllable and onset as comparison shots. A later pass can lighten lengths 5 and up, still off a fifth ratio, if the opening should sit closer to the reference.

## Reflow

Pride chapter 1 stays at 10 pages. The line count moves. The text matches before and after.

| Mode | Pages | Lines before | Lines after |
| --- | --- | --- | --- |
| off | 10 | 41 | 41 |
| tiers | 10 | 41 | 39 |
| syllable | 10 | 41 | 40 |
| onset | 10 | 41 | 38 |

The reference shot stays at 10 pages and moves from 41 lines to 38. That shot is not a shipped mode.

The page script waits until the line breaker has committed, then replaces text nodes with weight-600 spans. The line count changes because those spans are wider than the regular advances the breaker measured. On this chapter the page count stays put. The opening paragraph shows the move. The reference keeps "acknowledged" on one line. Tiers and syllable break it after "ac". Onset breaks it after "acknow".

Proposed typesetter fix, not in this draft. Measure the weight-600 prefix inside the line breaker, and include the mode in the break-cache key and the page stamp. A toggle then cannot reuse regular-weight breaks.

## Highlight and copy

The spike holds the highlight, clears it, paints, and paints it again. Off and tiers both report character offsets 20 and 50. `highlight-off.png` and `highlight-tiers.png` mark the same words, "truth universally acknowledged".

`copy.txt` is the selection string of the opening paragraph after the tiers paint:

`It is a truth universally acknowledged, that a single man in possession of a good fortune, must be in want of a wife.`

`copyMarkup` is false. `document.execCommand("copy")` returned false, so this run left the system pasteboard alone. The selection string has no tags.

## Chinese

范愛農 with letter tiers leaves the title and the Chinese paragraphs alone. The first long paragraph reports no Latin words. The Latin name in the body is bold, because the rule matches ASCII letters. Pages move from 55 to 56, and lines from 143 to 144. The text is unchanged. The spike forces Songti SC at 20px and a white page whenever the flag is set, including on a Chinese book. That force, plus the one Latin name, accounts for the extra line.

No effect on Han. Do not ship this for Chinese body text. A later control should leave Han runs untouched, and it should leave a Chinese book's typeface alone.

## Where the control would go

This draft has no menu item. When it ships, add one View menu item, off by default, titled with `BOLD_WORD_STARTS_LABEL`.

## Overlap

`spikes/shell/src/web.rs` is also in PRs 8, 14, and 15. This draft adds a `bws` message branch, turns the web view backing white only while the flag is set, and appends the page script in `inject`. The default backing color stays the paper color.

`spikes/justify/justify.js` is also in PR 13. This draft adds `window.spikeSyllablePoints`, which returns the existing `liangPoints` for a word.

## IP

Limits for this work live in `.cursor/rules/bold-word-starts-ip.mdc`. `just bold-word-starts-ip` greps the tree for the names listed there and runs the `bold_word_starts` tests.
