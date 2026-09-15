URL: https://raw.githubusercontent.com/torvalds/linux/master/drivers/hid/hid-ids.h
Also: https://ratatoskr.run/lkml/2026/09/17514722/t ("[PATCH v8] HID: pulsar: add driver for Pulsar gaming mice")
Retrieval date: 2026-09-15
Source type: Linux kernel source (primary) and kernel mailing list patch with maintainer action

# Linux kernel: VID 0x3554 entries and the Pulsar HID driver patch

## Acquisition

- hid-ids.h fetched with curl and grepped (not saved).
- Patch thread read with the fetch tool (text extraction; quoted spans verbatim as returned).

## Excerpts

hid-ids.h (verbatim):

```c
#define USB_VENDOR_ID_KYSONA		0x3554
#define USB_DEVICE_ID_KYSONA_M600_DONGLE	0xF57C
#define USB_DEVICE_ID_KYSONA_M600_WIRED	0xF57D
...
#define USB_VENDOR_ID_VXE		0x3554
#define USB_DEVICE_ID_VXE_DRAGONFLY_R1_PRO_DONGLE	0xf58a
#define USB_DEVICE_ID_VXE_DRAGONFLY_R1_PRO_WIRED	0xf58c
```

Patch v8 (fetch tool extraction):

- Title "[PATCH v8] HID: pulsar: add driver for Pulsar gaming mice", author Nikolas Koesling, September 4, 2026.
- Devices: Pulsar X2 V2, X2H, X2A, Xlite V3; Kysona M600; ATK VXE R1 SE+; VXE Dragonfly R1 Pro.
- Adds USB_VENDOR_ID_PULSAR (0x3554) with wired, 1kHz and 4kHz dongles; USB_VENDOR_ID_ATK (0x3554) and USB_VENDOR_ID_ATK_ALT (0x373B).
- Constants quoted by the tool: "USB_PAYLOAD_LEN 17", "CMD_HID_REPORT_ID 0x08", "CHECKSUM_MAGIC 0x55".
- Commands: INFO (0x01), STATUS (0x03), POWER (0x04), EVENT (0x0a, receive-only).
- The driver exposes battery level, voltage and charging status via power_supply.
- Maintainer action: Jiri Kosina queued the patch in hid.git#for-7.4/pulsar on September 11, 2026.

No firmware update command appears in the extracted patch summary.
