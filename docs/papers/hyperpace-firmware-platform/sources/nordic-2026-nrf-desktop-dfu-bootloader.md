URL: https://github.com/nrfconnect/sdk-nrf/blob/main/applications/nrf_desktop/doc/dfu.rst
Also: https://github.com/nrfconnect/sdk-nrf/blob/main/applications/nrf_desktop/bootloader_dfu.rst
Retrieval date: 2026-09-15
Source type: official vendor documentation (Nordic Semiconductor, nRF Connect SDK)

# Nordic nRF Desktop: DFU module and bootloader choices

## Acquisition

Same sparse clone as `nordic-2026-nrf-desktop-config-channel.md`: `sdk-nrf` commit `2700156c8b09008e9363261c1cd45d45067f3ef7`, scratch path `research-bin/chips/sdk-nrf`. The rendered page https://docs.nordicsemi.com/bundle/ncs-latest/page/nrf/applications/nrf_desktop/bootloader_dfu.html returned HTTP 403 to the fetch tool.

## Excerpts (verbatim)

### dfu.rst

> * Obtain the update image from :ref:`nrf_desktop_config_channel` and store it in the appropriate non-volatile memory partition.
> * Erase the non-volatile memory partition in the background before storing the update image.

> To perform the firmware upgrade, you must enable the bootloader.
> You can use the DFU module with either MCUboot or B0 bootloader.

> The received update image chunks are stored on the dedicated non-volatile memory partition when the current version of the device firmware is running.
> For this reason, make sure that you use configuration with a dedicated update image partition.

> * ``swap`` (:kconfig:option:`CONFIG_MCUBOOT_BOOTLOADER_MODE_SWAP_USING_MOVE`) - ``MCUBOOT``
> * ``direct-xip`` (:kconfig:option:`CONFIG_MCUBOOT_BOOTLOADER_MODE_DIRECT_XIP`) - ``MCUBOOT+XIP``
> * ``RAM load`` (:kconfig:option:`CONFIG_MCUBOOT_BOOTLOADER_MODE_RAM_LOAD`) - ``MCUBOOT``

> * Requests the image upgrade after the whole image is transferred over the :ref:`nrf_desktop_config_channel`.
> * Confirms the running image after the device is rebooted.

> If this bootloader mode is selected, the DFU module does not mark the newly uploaded image as pending and does not confirm it after a successful boot.

Options: `fwinfo`, `devinfo`, `reboot`, `start`, `data`, `sync`, `module_variant`.

> start
>    Perform the set operation on this option to start the DFU.
>    The operation contains the following data:
>
>    * Size of the image being transmitted.
>    * Checksum of the update image.
>    * Offset at which the host tool is going to start the image update.

> If the transmission is interrupted, the current offset position is stored along with the image checksum and size until the device is rebooted.

> For performance reasons, the :ref:`nrf_desktop_config_channel` set operation does not return any information back to the host.
> To check that the update process is correct, the host tool must perform fetch operation on the ``sync`` option at regular intervals.

> The DFU module does not check if the update image contains any valid data.
> It also does not check if it is correctly signed.
> When the update image is received, the host tool requests a reboot.
> Then, the bootloader will check the image for validity and ensure that the signature is correct.
> If the verification of the new images is successful, the new version of the application will boot.

> The host must synchronize the firmware image transfer progress at least on every synchronization buffer byte count.

> To ensure that the non-volatile memory write will not interfere with the device usability, the stored data is split into small chunks and written only if there are no HID reports transmitted and the Bluetooth connection state does not change.

> If a module does not use the :ref:`nrf_desktop_dfu_lock`, simultaneous modification of the data by the module and another application module might result in a broken image.
> This broken image will then be rejected by the bootloader.

### bootloader_dfu.rst

> B0 is a small, simple, and secure bootloader that allows the application to boot directly from one of the application slots, thus increasing the speed of the direct firmware upgrade (DFU) process.

> This bootloader can be used only for the :ref:`background DFU <nrf_desktop_bootloader_background_dfu>` through the :ref:`nrf_desktop_config_channel` and :ref:`nrf_desktop_dfu`.

> In this scenario, the MCUboot either swaps the application images located on the secondary and primary slots before booting a new image (``swap mode``) or boots a new application image directly from the secondary image slot (``direct-xip mode``).

> In this mode, unlike in the background DFU mode, the DFU image transfer is handled by the bootloader.
> The application is not running and it can be overwritten.
> Because of that, only one application slot may be used.

> If the swap mode is used, the application must request a firmware upgrade and confirm the running image.

> The nRF Desktop application configurations do not use the direct-xip mode with the revert mechanism (:kconfig:option:`SB_CONFIG_MCUBOOT_MODE_DIRECT_XIP_WITH_REVERT`).

> When the direct-xip mode is enabled, the application modules that control the DFU transport do not request firmware upgrades or confirm the running image.
> In that scenario, the MCUboot bootloader simply boots the image with the higher image version.

> The bootloader overwrites the existing application located on the primary slot with the new application image.
> If the transfer is interrupted, the device cannot boot the incomplete application, and the image upload must be performed again.

> Make sure that you use your own private key for the release version of the devices.
> Do not use the debug key for production.

## Downgrade note (search-engine snippet of the same doc set)

Query "Nordic nRF Desktop CONFIG_DESKTOP_CONFIG_CHANNEL_DFU ... downgrade" returned, attributed to the nRF Connect SDK DFU module page:

> For each image included in the DFU-generated package, you should use a higher version number than your currently active firmware. Otherwise, the DFU target may reject the FOTA process due to a downgrade prevention mechanism.

And (attributed to the broader Nordic SDK guidance in the same result set):

> You can disable the version check by setting NRF_DFU_APP_DOWNGRADE_PREVENTION to 0 in the bootloader's sdk_config.h file.

These two lines are search-engine renderings, not verbatim page reads (the docs host returns 403 to the fetch tool). The mechanism is confirmed at source level in `dius-2019-nrf5-sdk-15-3-dfu-validation-source.md` (nRF5 SDK) and `mcu-tools-2026-mcuboot-design.md` (MCUboot). Whether downgrade prevention is compiled into a given Compx/Lofree image is a build-time choice not visible from outside; see the DECISION note in the summary.

## B0 (NSIB) slot selection and rollback, from NCS source

Verbatim from `sdk-nrf` `samples/bootloader/src/main.c` and `subsys/bootloader` (same clone, commit `2700156...`):

```c
	if (s0_info && s1_info) {
		if (s0_info->version >= s1_info->version) {
			validate_and_boot(s0_info, BOOT_SLOT_0);
			validate_and_boot(s1_info, BOOT_SLOT_1);
		} else {
			validate_and_boot(s1_info, BOOT_SLOT_1);
			validate_and_boot(s0_info, BOOT_SLOT_0);
		}
	}
```

```c
#ifdef CONFIG_SB_MONOTONIC_COUNTER_ROLLBACK_PROTECTION
	counter_t stored_version;
	int err = get_monotonic_version(&stored_version);
	...
	if (fw_info->version > stored_version) {
		int err = set_monotonic_version(fw_info->version, slot);
```

So B0 boots the valid slot with the highest `version`. Rollback (monotonic-counter) protection is compiled in only under `CONFIG_SB_MONOTONIC_COUNTER_ROLLBACK_PROTECTION`; without it, B0 will boot an older-version image if it is the highest valid one present. `subsys/bootloader/bl_validation/bl_validation.c` line 44: `__ASSERT(version <= 0x7FFF, "version too large.\r\n")` (15-bit monotonic version). Signature model (from `samples/bootloader/README.rst`): NSIB stores hashes of provisioned public keys; the image carries the full public key, checked against a provisioned hash; ECDSA-P256; lower-index key hashes are permanently invalidated when a higher one matches (key revocation).
