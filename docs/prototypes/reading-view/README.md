# Reading view prototypes

Morris picks one of these single-page views. The switch is `SPIKE_READING=a`, `b`, or `c`, or the `0` key. It is not a setting.

A quiet justified page in Georgia and Songti, with a first-line indent and an instant cut.

An airy ragged page in New York and PingFang, with space between paragraphs and a short slide.

A narrow page in Iowan Old Style and PingFang, with a paper curl.

| View | Latin | CJK | Size | Line height | Measure | Paragraph | Align | Hyphens | Paper | Margins | Title | Footer | Turn |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| A | Georgia | Songti SC | 20px | 1.70 | 64ch | indent 2em | justify | auto | #f3eee4 | 72px 11vw 64px | fade 200ms 500ms | stay 0.42 | cut |
| B | New York | PingFang SC | 18px | 1.85 | 52ch | space 0.85em | left | off | #f7f5ef | 56px 8vw 48px | fade 200ms 400ms | hide after 4000ms over 400ms | slide 160ms |
| C | Iowan Old Style | PingFang SC | 22px | 1.48 | 48ch | indent 2em | justify | auto | #fbf9f3 | 88px 14vw 72px | fade 300ms 500ms | stay 0.50 | curl 280ms |

The footer reads `590/1146页` and `本章还剩34页`. Page turns stay on one column. There is no second page beside it.

A turn's frame gaps are the time between animation frames inside the page. They cluster around 17 ms, one refresh. The check fails a gap of 33 ms, which is a missed refresh. `latency.json` holds the measured gaps.
