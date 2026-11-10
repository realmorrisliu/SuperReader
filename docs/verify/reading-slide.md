# reading-slide, this head against main

Six `just verify` runs, back to back, under `/tmp/superreader-verify.lock` and the same load gate (1-minute load under 8). Three on `ab94910` (this branch, shell identical to the rebase onto `11765a6`) and three on `11765a6`. There is no single-check mode. `--subset` is only `cache`, which does not include this check.

The check passes when every recorded interval is under 33.4 ms and the video exists. The detail text says `floor 16.7 ms`. The table lists every interval over 16.7 ms. None of the six runs had an interval at or over 33.4 ms, so `reading-slide` passed on both trees. The earlier 36.0 ms frame did not recur. It is not caused by this branch.

`scoreLines` runs only from `scoreChapter`. `scheduleTail` calls `scoreChapter` after layout, and only when `spikeBusy` is false. The slide sets `spikeBusy` before it records frames. `harvestCuts` on a paragraph is the plan reader, called from `readPlans` on that same tail, and that call is already on main. Neither runs inside the animation frame callback.

| Run | Tree | Gate load | Slide load | Result | Max | Intervals over 16.7 ms |
| --- | --- | ---: | --- | --- | ---: | --- |
| head 1 | ab94910 | 7.00 | 8.98 15.21 20.60 | pass | 29.0 | 17.0 26.0 17.0 21.0 17.0 17.0 19.0 23.0 19.0 23.0 24.0 17.0 23.0 17.0 23.0 21.0 20.0 29.0 |
| main 1 | 11765a6 | 7.61 | 9.53 22.55 30.80 | pass | 25.0 | 20.0 19.0 19.0 19.0 19.0 17.0 18.0 17.0 25.0 20.0 25.0 18.0 17.0 18.0 18.0 19.0 21.0 23.0 |
| head 2 | ab94910 | 7.15 | 11.58 11.14 20.19 | pass | 29.0 | 22.0 20.0 17.0 23.0 25.0 19.0 18.0 19.0 17.0 17.0 17.0 17.0 29.0 17.0 20.0 18.0 24.0 22.0 |
| main 2 | 11765a6 | 7.00 | 11.45 12.00 16.27 | pass | 28.0 | 19.0 19.0 18.0 18.0 18.0 17.0 21.0 18.0 18.0 17.0 18.0 20.0 18.0 28.0 17.0 25.0 17.0 17.0 19.0 |
| head 3 | ab94910 | 7.85 | 11.90 14.40 15.28 | pass | 23.0 | 18.0 18.0 19.0 17.0 19.0 17.0 19.0 18.0 18.0 18.0 21.0 22.0 18.0 23.0 17.0 |
| main 3 | 11765a6 | 7.61 | 9.32 12.18 15.60 | pass | 26.0 | 17.0 24.0 18.0 26.0 25.0 26.0 17.0 18.0 17.0 17.0 17.0 18.0 18.0 17.0 17.0 20.0 20.0 |

Each of these runs also failed `reading-cold-open` and `reading-warm-open`. Both failures are marked shared with native. No other check failed.
