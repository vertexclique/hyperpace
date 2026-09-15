# SUMMARY: Nordic nRF Desktop configuration channel

**Claim.** Nordic's nRF Desktop reference gaming-mouse design carries firmware updates over a HID feature report (the "configuration channel"), and a dongle can forward that channel to a BLE peripheral, acting as a DFU relay. Wire format: [Report ID, Recipient, Event ID (Module ID | Option ID), Status, Data length, Data...], report id 6, report size 30, so at most 25 data bytes per transaction. Recipient 0 = directly connected; non-zero = forward through the dongle. Set = write; fetch = read with a PENDING/SUCCESS poll.

**Method.** Read the rst source and host-script constants from a pinned sdk-nrf checkout (commit 2700156). Cross-checked the report layout against NrfHidDevice.py (REPORT_ID=6, REPORT_SIZE=30, ConfigStatus enum).

**Result.** Mechanism and format confirmed from primary source. This is Nordic's own design, distinct from Compx's.

**Evidence tier.** 1 (official vendor source code and documentation).

**Performance.** 25 payload bytes per HID transaction, one pending request at a time; DFU throughput bounded by feature-report round-trips and device RAM sync buffer.

**Correctness.** Proven: format, report id/size, forwarding semantics. Assumed/unknown: that Lofree/Compx use this. They do NOT: the Compx HYPACE protocol is report id 8, 16 bytes, checksum-to-0x55, a different design. This source is the Nordic baseline the question asked to compare against, not the HYPACE transport.

**Relevance to hyperpace.** Answers "does Compx report 9 / command 13 resemble nRF Desktop?": no. nRF Desktop uses feature report 6 (30 B) with a Module/Option model and a mandatory module-discovery handshake; HYPACE's report 9 is a 48-byte channel the web driver never uses and has no discovery. If Lofree's updater turns out nRF-Desktop-based, this is the format to implement; more likely it is Compx-proprietary USB HID upgrade.
