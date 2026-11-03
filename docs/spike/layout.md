# Spike layout

Base sketch is the one that splits the window into a PDF strip, a single-page EPUB, and a TextView chapter. The other sketch lost because it kept both PDF engines hot inside one object and made the shell match on a five-way surface for every frame.

Grafted from the other sketch:

- Zoom is absolute. The same value a second time keeps the cache.
- The only measured scales are Retina 1x, 2 pixels per point shown at point size, and zoom 2x, 4 pixels per point shown at twice the point size.
- An EPUB anchor stores the spine href and a character range in the chapter's plain text. It does not store a page number.
- The measure binary does not link AppKit. A stale bitmap is never scaled up to fake a zoom.

A spread has no type. Comment threads and alan are not in this workspace. The anchor is the value a later thread would store.

Claude could not sit as the cross-family judge in this session, so the pick above is the parent's reading of the two sketches.
