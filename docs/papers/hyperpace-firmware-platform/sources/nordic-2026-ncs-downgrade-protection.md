URL: https://github.com/nrfconnect/sdk-nrf/blob/main/doc/nrf/app_dev/bootloaders_dfu/mcuboot_nsib/bootloader_downgrade_protection.rst
Also: https://github.com/nrfconnect/sdk-nrf/blob/main/doc/nrf/app_dev/bootloaders_dfu/mcuboot_nsib/bootloader_dfu_image_versions.rst, https://github.com/nrfconnect/sdk-nrf/blob/main/samples/bootloader/README.rst, https://github.com/nrfconnect/sdk-nrf/blob/main/samples/bootloader/src/main.c
Retrieval date: 2026-09-15
Source type: official vendor documentation and bootloader source code (Nordic nRF Connect SDK: NSIB, also called B0, and MCUboot integration)

# nRF Connect SDK: downgrade protection, image versions, B0 slot selection

## Acquisition

Sparse clone `sdk-nrf` commit `2700156c8b09008e9363261c1cd45d45067f3ef7` (scratch `research-bin/chips/sdk-nrf`), paths added: `doc/nrf/app_dev/bootloaders_dfu`, `samples/bootloader`, `subsys/bootloader`. Nothing was built.

## Excerpts (verbatim)

### bootloader_downgrade_protection.rst

> The downgrade protection feature blocks downgrading the firmware version on the device.

> As part of this process, the bootloader compares the new firmware image version with the version of the currently booted firmware.
> The bootloader rejects the update if the new image version is lower than the version of the currently booted firmware.

> * Software-based downgrade protection is supported only by MCUboot.
>   In this kind of protection, the current firmware version is encoded into the currently active image.
> * Hardware-based downgrade protection is supported by both MCUboot and |NSIB|.
>   In this kind of protection, the current version is encoded into a non-volatile storage partition outside of any image.

> To enable this feature, set the configuration option :kconfig:option:`CONFIG_MCUBOOT_DOWNGRADE_PREVENTION` for the MCUboot image and :kconfig:option:`SB_CONFIG_MCUBOOT_MODE_OVERWRITE_ONLY` for sysbuild.

> Enabling :kconfig:option:`SB_CONFIG_MCUBOOT_MODE_OVERWRITE_ONLY` prevents the fallback recovery of application images.

> In such cases, the application could roll back to a previous working version, but the network core would remain unchanged, leading to inconsistencies.

> If this image has, in order of precedence, a *major*, *minor*, or *revision* value that is lower than the primary application image, it is considered invalid and the existing primary application boots instead.

> By default, the optional label or build number specified after the ``+`` character is ignored when evaluating the version.

> Counter updates are written to slots in the *Provision* area, with each new counter update occupying a new slot.
> For this reason, the number of counter updates, and therefore firmware version updates, is limited.

> Using a counter is optional and can be configured for the application using configuration options.

> To enable anti-rollback protection with monotonic counter for |NSIB|, set the following sysbuild configuration options: :kconfig:option:`SB_CONFIG_SECURE_BOOT_MONOTONIC_COUNTER` and :kconfig:option:`SB_CONFIG_SECURE_BOOT_NUM_VER_COUNTER_SLOTS`

> * :kconfig:option:`SB_CONFIG_MCUBOOT_HARDWARE_DOWNGRADE_PREVENTION`

> This design means that hardware downgrade protection can be used by the |NSIB| or MCUboot, but not both simultaneously.

### bootloader_dfu_image_versions.rst

> You can set the image version for your application using the :kconfig:option:`CONFIG_FW_INFO_FIRMWARE_VERSION` Kconfig option.

### samples/bootloader/README.rst

> For the signature verification, to save space, NSIB only stores the hashes of the provisioned keys and compares only the hashes of these keys.
> The next image has metadata containing the full public key that corresponds to the private key used to sign the firmware.
> This public key is checked against the provisioned hashes of public keys to determine if the image is valid.
> All public key hashes at lower indices than the matching hash are permanently invalidated at this point.

> When the image for the next stage in the boot chain is upgraded, the new image is written to the slot with the oldest image version.

> The ECDSA-P256 key type is supported for validating the next image in the boot chain.

### samples/bootloader/src/main.c (lines 150 to 262)

```c
	if (!bl_validate_firmware_local(fw_info->address,
					fw_info)) {
		printk("Failed to validate, permanently invalidating!\r\n");
		fw_info_invalidate(fw_info);
		return;
	}

	printk("Firmware version %d\r\n", fw_info->version);

#ifdef CONFIG_SB_MONOTONIC_COUNTER_ROLLBACK_PROTECTION
	counter_t stored_version;

	int err = get_monotonic_version(&stored_version);

	if (err) {
		printk("Failed to read the monotonic counter!\r\n");
		return;
	}

	if (fw_info->version > stored_version) {
		int err = set_monotonic_version(fw_info->version, slot);

		if (err) {
			return;
		}
	}
#endif
```

```c
	if (s0_info && s1_info) {
		if (s0_info->version >= s1_info->version) {
			validate_and_boot(s0_info, BOOT_SLOT_0);
			validate_and_boot(s1_info, BOOT_SLOT_1);
		} else {
			validate_and_boot(s1_info, BOOT_SLOT_1);
			validate_and_boot(s0_info, BOOT_SLOT_0);
		}
	} else if (s0_info) {
		validate_and_boot(s0_info, BOOT_SLOT_0);
	} else if (s1_info) {
		validate_and_boot(s1_info, BOOT_SLOT_1);
	}

	printk("No bootable image found. Aborting boot.\r\n");
```

## Not read

`bl_validate_firmware_local` (in `subsys/bootloader/bl_validation/bl_validation.c`) was not read in full, so whether it compares an image version against the monotonic counter is not quoted here.
