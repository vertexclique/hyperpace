URL: https://fwupd.github.io/libfwupdplugin/nordic-hid-README.html
Also: https://github.com/nrfconnect/sdk-nrf/blob/main/applications/nrf_desktop/fwupd.rst
Retrieval date: 2026-09-15
Source type: official project documentation (fwupd plugin README) and vendor documentation (Nordic)

# fwupd nordic_hid plugin (nRF Desktop DFU on Linux)

## Acquisition

- fwupd page: read with the fetch tool (its text extraction; spans in quotation marks are verbatim as returned).
- Nordic `fwupd.rst`: local file from the `sdk-nrf` sparse clone, commit `2700156c8b09008e9363261c1cd45d45067f3ef7`.

## Excerpts

### fwupd plugin README (fetch tool extraction)

> "This plugin is able to update the firmware for the hardware supported by nRF Desktop application reference design."

> The cabinet file contains a ZIP archive with either one or two signed image blobs and a `manifest.json` file. "At the moment only nRF Secure Immutable Bootloader ("B0") is supported and tested."

> GUIDs incorporate the target board name, bootloader name, and generation string. Examples include: `HIDRAW\VEN_1915&DEV_52DE&BOARD_nrf52840dk&BL_B0&GEN_default`.

> "The firmware is deployed when the device is in normal runtime mode, and the device will reset when the new firmware has been written."

> The plugin requires ioctl `HIDIOCSFEATURE` and `HIDIOCGFEATURE` access.

> Available since fwupd 1.7.3. Format version 1 support added in fwupd 1.9.25, corresponding to nRF Connect SDK v2.7.0.

### Nordic fwupd.rst (verbatim)

> The fwupd tools can communicate with devices running the nRF Desktop application with the :ref:`nrf_desktop_bootloader_background_dfu` feature enabled.

> The plugin associated with the DFU protocol realized through the :ref:`nrf_desktop_config_channel` is branded as ``nordic_hid``.

```
   [HIDRAW\VEN_1915&DEV_52DE]
   Plugin = nordic_hid
   GType = FuNordicHidCfgChannel
   NordicHidBootloader = B0
```

> If there is no information about the bootloader, both in metadata and from the device, the update procedure fails.
> The possible values are either ``B0``, ``MCUBOOT``, or ``MCUBOOT+XIP``.

> As the ``nordic_hid`` plugin communicates with the device using the Configuration channel, the device update is not allowed through the Serial recovery DFU.

> When the CAB archive has been built, it can be uploaded to the LVFS where it is verified and signed.
