# Bold word starts

This draft is a prototype. The chapter is unchanged unless `SPIKE_BOLD_WORD_STARTS` is set. The only user-facing name is `BOLD_WORD_STARTS_LABEL`, "Bold Word Starts" and "词首加粗".

The comparison sheet is `docs/prototypes/bold-word-starts/compare-sheet.png`. It has the opening sentence, the long-word paragraph, and a 200% crop of that paragraph. The 200% files are `*-long-2x.png` in the same directory. On the sheet those files are drawn at twice the type size of the paragraph row.

## Pages

The shots are the spike-shell WKWebView on macOS. Pride and Prejudice, chapter 1, is a white page in Iowan Old Style at 22px. Every column uses font-weight 600.

`scripts/bold-word-starts-shots` takes `/tmp/superreader-verify.lock` before it opens the window, and it waits until the one-minute load average is under 8.

## Reference

The left column is the common preset, comparison only, not shipped. The count function lived in a file outside the repo, passed through `SPIKE_BOLD_WORD_STARTS_FILE`, and was deleted after the shot.

## Rules

Han gets a count of 0. The page script bolds only runs of ASCII letters.

Letter tiers use `tier_bold_count`. Lengths 1 through 12 bold 1, 1, 2, 3, 4, 3, 3, 4, 4, 5, 5, 6 letters. From length 13 the count starts at the ceiling of `len * 45 / 100` and steps to a neighbor when the unit test rejects that count. Lengths 1 through 4 are 1, 1, 2, 3. The unit test checks that sequence against the short-word table named in `.cursor/rules/bold-word-starts-ip.mdc`.

Onset and nucleus counts the opening consonants plus the following vowel. `y` counts as a vowel after the first letter. A word that starts with a vowel has a count of 1.

Hybrid uses onset for a word of 6 letters or fewer, and letter tiers from 7 letters up. `HYBRID_ONSET_MAX` is 6. The page script reads that constant from the Rust table, so the two copies do not drift.

The cutoff is 6 because that is where the two rules trade places on the opening sentence. Through length 6, onset stays closer to the reference than tiers. "good", "must", "want", "wife", and "single" take 2 letters under onset and 3 under tiers. "truth" takes 3 under onset and 4 under tiers. From length 7, a short onset stops being a readable prefix. "fortune" is the first opening word past the cutoff. The words of 9 letters and up are the ones onset marks with a single letter when they start with a vowel: "experience", "insufficient", "understand", "understanding", "information", "uncertain" in the second paragraph, and "universally" in the first. Those words take the tier count, which is 4, 5, or 6 letters.

Lengths 1 through 6 do not have one count. The count is the onset of that word. A probe word of each length from 1 through 15, locked by the unit test, bolds 1, 2, 2, 2, 3, 2, 3, 4, 4, 5, 5, 6, 6, 7, 7 letters. The probes are a, be, man, good, truth, single, fortune, entering, character, experience, universally, acknowledged, understanding, characteristic, representations. From length 7 the count is the tier count for every word of that length. "be" is 2 and "that" is 3, so the short side is not the 1, 1, 1, 2 length table.

## Paragraphs

The opening sentence is the first paragraph of chapter 1.

The long-word paragraph is the last paragraph of the same chapter, the one that begins "Mr. Bennet was so odd a mixture". It has 11 words of 9 letters or more: sarcastic, experience, insufficient, understand, character, difficult, understanding, information, uncertain, discontented, daughters.

## Comparison

Counts below come from `docs/prototypes/bold-word-starts/stats.json`. Average is bold letters per word. Share is the fraction of words whose bold count equals the reference. The long columns count only words of 9 letters or more.

### Opening sentence

| Rule | Words | Average | Median | Share | Long words | Long average | Long median | Long share |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Common preset, comparison only, not shipped | 23 | 40/23, 1.74 | 1 | 23/23 | 3 | 4 | 4 | 3/3 |
| Letter tiers | 23 | 54/23, 2.35 | 2 | 11/23, 48% | 3 | 16/3, 5.33 | 5 | 0/3 |
| Onset and nucleus | 23 | 36/23, 1.57 | 1 | 16/23, 70% | 3 | 4/3, 1.33 | 1 | 0/3 |
| Hybrid | 23 | 49/23, 2.13 | 2 | 15/23, 65% | 3 | 16/3, 5.33 | 5 | 0/3 |

### Long-word paragraph

| Rule | Words | Average | Median | Share | Long words | Long average | Long median | Long share |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Common preset, comparison only, not shipped | 77 | 139/77, 1.81 | 2 | 77/77 | 11 | 40/11, 3.64 | 4 | 11/11 |
| Letter tiers | 77 | 212/77, 2.75 | 3 | 11/77, 14% | 11 | 53/11, 4.82 | 5 | 0/11 |
| Onset and nucleus | 77 | 147/77, 1.91 | 2 | 38/77, 49% | 11 | 17/11, 1.55 | 1 | 1/11 |
| Hybrid | 77 | 195/77, 2.53 | 2 | 29/77, 38% | 11 | 53/11, 4.82 | 5 | 0/11 |

### Both paragraphs

| Rule | Words | Average | Median | Share | Long words | Long average | Long median | Long share |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Common preset, comparison only, not shipped | 100 | 179/100, 1.79 | 2 | 100/100 | 14 | 52/14, 3.71 | 4 | 14/14 |
| Letter tiers | 100 | 266/100, 2.66 | 3 | 22/100, 22% | 14 | 69/14, 4.93 | 5 | 0/14 |
| Onset and nucleus | 100 | 183/100, 1.83 | 2 | 54/100, 54% | 14 | 21/14, 1.50 | 1 | 1/14 |
| Hybrid | 100 | 244/100, 2.44 | 2 | 44/100, 44% | 14 | 69/14, 4.93 | 5 | 0/14 |

Onset reads noticeably worse on the long words. Across both paragraphs it bolds 1.50 letters of a 9-letter-or-longer word, against 3.71 for the reference, and only 1 of those 14 words matches the reference count. "universally", "experience", "insufficient", "understand", "understanding", "information", and "uncertain" each keep a single letter because they start with a vowel.

Letter tiers marks those words. It is heavier on the short words. The opening share is 11/23, and the long paragraph share is 11/77.

Hybrid uses the onset count through 6 letters, so the opening share rises to 15/23, and the tier count from 7 letters up, so the long-word counts match letter tiers. It does not match the reference count on those 14 long words. The tier prefix is one letter longer than the reference on average, 4.93 against 3.71. The words still have a prefix.

## Reflow

The page script paints after the line breaker has committed. Weight-600 spans are wider than the regular advances the breaker measured, so the line count can move while the text stays the same.

Chapter 1 stays at 10 pages. The line count moves from 41 to 39 under letter tiers, and from 41 to 38 under onset, hybrid, and the reference. The text is unchanged.

Proposed typesetter fix, not in this draft. Measure the weight-600 prefix inside the line breaker, and include the mode in the break-cache key and the page stamp.

## Highlight and copy

An earlier capture held the highlight across paint. Off and tiers both reported character offsets 20 and 50, and both shots mark "truth universally acknowledged". `copy.txt` is the plain selection string of the opening paragraph. `copyMarkup` is false. `document.execCommand("copy")` returned false, so that run left the system pasteboard alone.

## Chinese

范愛農 with letter tiers leaves the title and the Chinese paragraphs alone. A Latin name in the body is bold. Pages moved from 55 to 56 in that capture because the spike forces Songti SC at 20px while the flag is set.

No effect on Han. Do not ship this for Chinese body text. A later control should leave Han runs untouched, and it should leave a Chinese book's typeface alone.

## Where the control would go

This draft has no menu item. When it ships, add one View menu item, off by default, titled with `BOLD_WORD_STARTS_LABEL`.

## Overlap

`spikes/shell/src/web.rs` is also in PRs 8, 14, and 15. This draft adds a `bws` message branch, turns the web view backing white only while the flag is set, and appends the page script in `inject`. The default backing color stays the paper color.

## IP

Limits for this work live in `.cursor/rules/bold-word-starts-ip.mdc`. `just bold-word-starts-ip` greps the tree for the names listed there and runs the `bold_word_starts` tests. Hybrid is covered by those tests.

## Follow-ups

This draft is a prototype record. It is not the ship path. After a rule is picked, the work splits into four changes.

1. The rule function, its unit tests, and `just bold-word-starts-ip`.
2. WebKit rendering behind the setting, off by default, with shots.
3. The line breaker measuring the bold prefix, with the mode in the cache key and the page stamp.
4. The owned typesetter.
