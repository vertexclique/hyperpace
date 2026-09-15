URL: https://github.com/fwupd/fwupd/issues/5738
Related (titles from search results only): https://github.com/fwupd/fwupd/issues/7759, https://github.com/fwupd/fwupd/issues/7414
Retrieval date: 2026-09-15
Source type: maintainer-level issue report (reporter MarekPieta; Nordic nRF Desktop contributor per sdk-nrf history, not verified here)

# fwupd issue 5738: wrong identity for peripherals behind an nRF Desktop dongle

## Acquisition

Read with the fetch tool (text extraction). Quoted spans are verbatim as returned; the rest is the tool's paraphrase and is marked as such.

## Excerpts

- Title: "plugins: nordic-hid: Invalid VID and PID of devices connected via dongle"
- Opened April 21, 2023, by MarekPieta.
- Paraphrase returned by the tool: the nordic-hid plugin reports the dongle's VID/PID for peripherals connected through an nRF Desktop dongle, so the generated GUIDs do not match the ones in the .cab packages and firmware matching fails.
- Paraphrase: earlier behavior assigned hardcoded VID/PID 0x0000 to peripherals; affected versions are fwupd 1.7.5 and later, after commit c4ca026.
- Paraphrase: proposed fixes are sending the real VID/PID over the configuration channel (unavailable in legacy firmware) or reverting to 0x0000.
- No maintainer reply or linked fix was visible in the extraction.

Search result titles for related issues (not opened):

- "plugins: nordic_hid: Update error - No vendor ID set · Issue #7759 · fwupd/fwupd"
- "Failed to open /dev/hidrawX: Operation not permitted · Issue #7414 · fwupd/fwupd"
