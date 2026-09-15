URL: https://github.com/libusb/hidapi/wiki/Multi%E2%80%90threading-Notes
Retrieved: 2026-09-15
Source type: Project wiki page maintained by the HIDAPI maintainers (referenced as "authoritative" by the repository AGENTS.md).
Retrieval method: WebFetch (extraction model output; the quoted sentences below were returned in quotation marks, and the same rules appear verbatim in AGENTS.md, see libusb-2026-hidapi-c-source.md).
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

> HIDAPI v0.x.x is **not thread-safe**.

> Functions including `hid_init`, `hid_exit`, `hid_enumerate`, `hid_open`, `hid_open_path`, `hid_close`, and `hid_error(NULL)` "shall not be called concurently from different threads."

> **macOS Requirement:** "on macOS there is even stricter requirement: `hid_init` and `hid_exit` has to be called from the _same_ thread." Additionally, the initialization thread must remain active until all devices close and `hid_exit` completes.

> Device-specific functions are "thread-unsafe, but may be used from different threads on different devices."

> Since v0.15.0, dedicated `hid_read`/`hid_read_timeout` threads are safe when only read operations occur there. The documentation states: "such scenario/usage is perfectly safe for as long as only `hid_read`/`hid_read_timeout` and `hid_read_error` are the only functions to be used from a dedicated thread."
