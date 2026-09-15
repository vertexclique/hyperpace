# SUMMARY: Linux kernel VID 0x3554 and the Pulsar HID driver

**Claim.** VID 0x3554 is shared across CompX-ODM gaming mice (kernel names it KYSONA and VXE; the Pulsar v8 patch adds PULSAR and ATK aliases, all 0x3554). The mainline "hid-pulsar" driver (queued for 7.4) decodes the same report-8, checksum-0x55 protocol family (USB_PAYLOAD_LEN 17, CMD_HID_REPORT_ID 0x08, CHECKSUM_MAGIC 0x55; commands INFO 0x01, STATUS 0x03, POWER 0x04, EVENT 0x0a) but only for battery/telemetry: it has no firmware-update command.

**Method.** Grepped torvalds/linux hid-ids.h; read the LKML v8 patch thread via fetch.

**Result.** Confirms 0x3554 is CompX's shared ODM VID and that the report-8/0x55 framing is the same family as HYPACE.

**Evidence tier.** 1 (kernel source) + 2 (LKML patch with maintainer queue action).

**Performance.** N/A (input/telemetry driver).

**Correctness.** Proven: shared VID, matching frame format, checksum magic 0x55, command overlap (0x03 online, 0x04 battery match HYPACE). The kernel driver deliberately does no firmware update. HYPACE-specific PIDs (FB16/FB14) are not in hid-ids.h; the kernel entries are for other CompX brands.

**Relevance to hyperpace.** Independent confirmation of the HYPACE protocol family and the 0x55 checksum from a primary (kernel) source, and evidence that no open-source project implements the CompX firmware-update path, only config/telemetry. hyperpace would be first to implement flashing for this family and must reverse the official updater.
