# SUMMARY: Nordic HID configurator DFU host script

**Claim.** The reference host tool (hid_configurator, dfu.py, configurator_cli.py) implements DFU over the config channel: reads device fwinfo, matches board name, streams the image in <=25-byte chunks with periodic sync fetches, then reboots. Image is a dfu_application.zip with manifest.json and one or two signed .bin blobs; B0 blobs named signed_by_b0_s{slot}_image.bin, MCUboot app_update.bin. Host CRC is zlib CRC-32 seeded at 1 over 512-byte chunks. Crucially, perform_dfu prints device version and file version and asks the user y/n; it does NOT itself refuse an older image. Any downgrade block lives in the device bootloader.

**Method.** Read the four files from the pinned sdk-nrf checkout.

**Result.** Confirmed from source.

**Evidence tier.** 1.

**Performance.** Chunk size = REPORT_SIZE-5 = 25 bytes; sync cadence tied to device sync-buffer size; store retries use bounded exponential backoff capped at DFU_SYNC_INTERVAL. No fixed sleeps in the hot loop.

**Correctness.** Proven: no host-side version gate. Assumed: MCUboot validity via imgtool verify. Not applicable to HYPACE unless Lofree ships an nRF-Desktop stack.

**Relevance to hyperpace.** A working reference for a native Rust reimplementation of Nordic DFU if needed, and a precise statement that a downgrade UI must rely on the bootloader's policy, not the transfer tool. Confirms the zip/manifest packaging hyperpace would parse.
