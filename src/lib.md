This crate implements a simple interface for accessing `ccache` stats
without needing an `exec` call.

Experimental testing demonstrates I can produce the same data emitted by
`ccache --print-stats` while **also** using only ¼ the heap, and less
than ½ the stack, while the program itself can call do this multiple
times in-process without adding an `exec()` penalty to everything.
