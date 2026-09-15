URL: https://pkg.go.dev/github.com/go-macos/iokit/hid
Retrieved: 2026-09-15
Source type: Third-party library documentation (Go package docs), module v0.13.0 published 2026-09-08.
Retrieval method: WebFetch (extraction model; sentences were returned in quotation marks).
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

> "A vendor-specific device (usage page 0xFF00 and up) normally opens without any user consent. A device that publishes a keyboard or pointer usage is gated by macOS input monitoring, and Open then fails with an IOError carrying kIOReturnNotPermitted."

> "IOKit delivers them to a CoreFoundation run loop, and a run loop belongs to one OS thread. Stream therefore pins the calling goroutine for its whole duration and calls the handler on that thread."

> "fn is called ON that thread, synchronously, between pumps. It must not block for long -- a slow handler is back-pressure on the device."

> "IOKit keeps the device scheduled on a run loop with a registered callback, releasing it leaves that schedule pointing at freed memory, and macOS kills the program with 'BUG IN CLIENT OF LIBPLATFORM: os_unfair_lock is corrupt' -- SIGKILL, no Go panic, no stack, nothing on the program's own output."

> "Cancelling a stream's context does not stop it -- the pump is inside CFRunLoopRunInMode and looks at the context when it comes out -- so the caller must join the goroutine running Stream before closing: cancel, wait, close."
