URL: https://github.com/nrfconnect/sdk-nrf/tree/main/scripts/hid_configurator
Retrieval date: 2026-09-15
Source type: official vendor host-tool source code and README (Nordic Semiconductor)

# Nordic HID configurator: the host side of DFU over HID

## Acquisition

Sparse clone of `sdk-nrf` commit `2700156c8b09008e9363261c1cd45d45067f3ef7` (scratch `research-bin/chips/sdk-nrf`). Files read: `README.rst`, `configurator_cli.py`, `NrfHidDevice.py`, `modules/dfu.py`.

## Excerpts (verbatim)

### README.rst

> The script looks for nRF Desktop devices connected to the host through USB, Bluetooth®, or nRF Desktop dongle.
> The devices are identified based on Vendor ID.

> The nRF Desktop application supports background DFU (Device Firmware Upgrade).
> The image is passed to the device while the device is in normal operation.
> The new image is stored on a dedicated update partition of the non-volatile memory.
> When the whole image is transmitted, the update process is completed during the next reboot of the device.

> If the DFU process is interrupted, it can be resumed using the same image, unless the device restarts.
> After the device reboots, the process always starts from the beginning.

> The ``dfu`` command reads the version of the firmware and the bootloader variant that are running on the device and compares them with the firmware version and the bootloader variant in the update image at the provided path.

> The script uses the :file:`dfu_application.zip` file.

### modules/dfu.py

```
DFU_SYNC_INTERVAL = 1

class DFUInfo:
    _DFU_STATE_INACTIVE = 0x00
    _DFU_STATE_ACTIVE   = 0x01
    _DFU_STATE_STORING  = 0x02
    _DFU_STATE_CLEANING = 0x03
    ...
    def __init__(self, fetched_data):
        fmt = '<BIIIH'
```

FwInfo uses `fmt = '<BIBBHI'`; devinfo uses `fmt = '<HH'`.

```
def b0_get_dfu_image_name(dfu_slot_id):
    return f'signed_by_b0_s{dfu_slot_id}_image.bin'
...
def mcuboot_get_dfu_image_name(dfu_slot_id):
    return 'app_update.bin'
...
def mcuboot_xip_get_dfu_image_name(dfu_slot_id):
    assert dfu_slot_id in (0, 1)

    if dfu_slot_id == 0:
        return 'app_update.bin'
    else:
        return 'mcuboot_secondary_app_update.bin'
```

```
        dfu_slot_id = 1 - flash_area_id
```

```
        if zip_board_name != dev_board_name:
            print(f"Update file is for other board: {zip_board_name}")
```

```
def dfu_start(dev, img_length, img_csum, offset):
    # Start DFU operation at selected offset.
    # It can happen that device will reject this request - this will be
    # verified by dfu sync at data exchange.
    event_data = struct.pack('<III', img_length, img_csum, offset)
```

```
def file_crc(dfu_image):
    crc32 = 1
    ...
        chunk_data = img_file.read(512)
        ...
        crc32 = zlib.crc32(chunk_data, crc32)
```

```
            if (not dfu_info.is_busy()) and (dfu_info.get_offset() != img_length):
                print('DFU interrupted by device')
                return False
```

```
        # Read data from the file
        chunk_len = EVENT_DATA_LEN_MAX
```

(`EVENT_DATA_LEN_MAX = REPORT_SIZE - 5` with `REPORT_SIZE = 30`, see `nordic-2026-nrf-desktop-config-channel.md`.)

MCUboot images are checked with `imgtool.image.Image.verify(dfu_bin, None)`.

### configurator_cli.py, `perform_dfu` (version handling)

```
    img_ver_file = img_file.get_dfu_image_version()
    if img_ver_file is None:
        print('Cannot read image version from file')
        return

    if bootloader_variant and bootloader_variant != img_file.get_dfu_image_bootloader_var():
        print('Bootloader types does not match')
        return

    print('Current FW version from device: ' +
          '.'.join([str(i) for i in img_ver_dev]))
    print('Current FW version from file: ' +
          '.'.join([str(i) for i in img_ver_file]))
    print('Perform update? [y/n]')
```

No other comparison of `img_ver_dev` against `img_ver_file` appears in `perform_dfu` (lines 52 to 110 read). The host tool shows both versions and asks the user; it does not refuse an older image.
