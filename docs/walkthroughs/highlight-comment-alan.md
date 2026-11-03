# Highlighting a passage, commenting, and asking alan

Morris is on a single page. He drags across a passage and highlights it. He writes his own note on that highlight, the way a comment sits on a passage in Google Docs, Notion, or Feishu.

He wants alan to look at the same passage. He writes `@alan` and the question in the comment. The comment switches to a thinking state. The reply arrives whole.

SuperReader owns the library, the SQLite annotations, tantivy, the comment thread, and the comment UI. alan is not linked into the reader process, and SuperReader does not speak alan's socket protocol. SuperReader spawns a fresh one-shot `alan` process and pipes stdin. The body is the passage, Morris's note, the earlier comments in the thread, and the new question. The spawn does not pass `--resume`. The annotation id and the thread id stay in SuperReader.

alan's reply arrives as one stdout payload. SuperReader writes it into SQLite as alan's reply comment, and the thinking state becomes that reply.

Another agent, alan included, reads the library and the notes through the `superreader` command, which prints JSON, and through a skill. alan's bash is inside a macOS Seatbelt sandbox that denies writes outside mounted directories, so a write from that command is applied by the running SuperReader app. The command does not open the database. An MCP wrapper comes later, and only for a client that cannot run a terminal.

The exact comment chrome is still open. Two or three prototypes decide it. This walkthrough fixes the handoff, not the chrome.

`@alan` waits until alan has a launch flag that does not rewrite alan's global root-rollout pointer. That flag is Morris's to land in the alan repository. Until it has landed, this walkthrough is the target, not a milestone that can start.
