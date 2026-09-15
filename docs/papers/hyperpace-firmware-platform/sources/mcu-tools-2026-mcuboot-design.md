URL: https://github.com/mcu-tools/mcuboot/blob/main/docs/design.md
Retrieval date: 2026-09-15
Source type: open-source bootloader design documentation (MCUboot project, used by nRF Connect SDK)

# MCUboot design: swap and revert, direct-xip selection, downgrade prevention

## Acquisition

`git clone --depth 1 https://github.com/mcu-tools/mcuboot.git` into `research-bin/chips/mcuboot` (scratch), commit `184e436f737730c5fbf990f09af2fe6ee22c416d` (commit date 2026-09-14). File read: `docs/design.md`.

## Excerpts (verbatim)

### Swap with test and revert (lines 460 to 500)

> Upgrading an old image with a new one by swapping can be a two-step process. In
> this process, MCUboot performs a "test" swap of image data in flash and boots
> the new image or it will be executed during operation. The new image can then
> update the contents of flash at runtime to mark itself "OK", and MCUboot will
> then still choose to run it during the next boot. When this happens, the swap is
> made "permanent". If this doesn't happen, MCUboot will perform a "revert" swap
> during the next boot by swapping the image(s) back into its original location(s)
> , and attempting to boot the old image(s).

> Test swaps are supported to provide a rollback mechanism to prevent devices
> from becoming "bricked" by bad firmware.  If the device crashes immediately
> upon booting a new (bad) image, MCUboot will revert to the old (working) image
> at the next device reset, rather than booting the bad image again.

> - `BOOT_SWAP_TYPE_FAIL`: Swap failed because image to be run is not valid.

### Direct-xip slot selection (lines 384 to 390)

> built to be executed from the corresponding slot. At boot time the bootloader
> first looks for images in the slots and then inspects the version numbers in the
> image headers. It selects the newest image (with the highest version number) and
> then checks its validity (integrity check, signature verification etc.). If the
> image is invalid MCUboot erases its memory slot and starts to validate the other
> image. After a successful validation of the selected image the bootloader
> chain-loads it.

### Downgrade prevention (lines 1400 to 1432)

> Downgrade prevention is a feature which enforces that the new image must have a
> higher version/security counter number than the image it is replacing, thus
> preventing the malicious downgrading of the device to an older and possibly
> vulnerable version of its firmware.

> During the software based downgrade prevention the image version numbers are
> compared. This feature is enabled with the `MCUBOOT_DOWNGRADE_PREVENTION`
> option. In this case downgrade prevention is only available when the
> overwrite-based image update strategy is used (i.e. `MCUBOOT_OVERWRITE_ONLY`
> is set).

> Each signed image can contain a security counter in its protected TLV area, which
> can be added to the image using the `-s` option of the [imgtool](imgtool.md) script.
> During the hardware based downgrade prevention (alias rollback protection) the
> new image's security counter will be compared with the currently active security
> counter value which must be stored in a non-volatile and trusted component of
> the device. It is beneficial to handle this counter independently from image
> version number:
>
>   * It does not need to increase with each software release,
>   * It makes it possible to do software downgrade to some extent: if the
>     security counter has the same value in the older image then it is accepted.

> It is an optional step of the image validation process and can be enabled with
> the `MCUBOOT_HW_ROLLBACK_PROT` config option.

TLV id: `#define IMAGE_TLV_SEC_CNT           0x50    /* security counter */`
