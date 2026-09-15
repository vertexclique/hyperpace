# Search log: stream "chips" (firmware update mechanisms per chip)

Retrieval date 2026-09-15. Tools: WebSearch, WebFetch, curl, git clone, pdftotext, strings, python (byte reading only). No device node touched; no downloaded binary executed; the vendor .exe installer was NOT downloaded.

## Chip identity: CX52650N / CX52850 / CompX

- "CX52650N" -> ring jewelry + one hit "CX52650P used in MONKA M3". Weak.
- "CX52650" / "CX52650P" mouse receiver chip -> AJAZZ AJ139, MONKA M3 "Shanghai Botong (Jiangmeng) CX52650P, Nordic 52840", XM2we CX52850+CX52650 pairing. Useful.
- "CompX CX52850 semiconductor company" -> Medium article "Compx Tech enables wireless mouse" (fabless Chinese, 2.4G, no public datasheet); Endgame Gear XM2we/OP1we listings. Useful.
- CX52850 芯片 康芯/匠盟/博通 (several Chinese queries) -> 匠盟 = Compx; 上海博通 = Beken; TAIDU TSG808 "CX52850 PRO+", GravaStar "BK CX52850". Established 匠盟(Compx) and 博通(Beken) relationship.
- "BK3633" "CX52850" -> TechPowerUp Fantech Aria II: "CompX CX52850 ... rebranded Beken BK3633". KEY: CompX = Beken silicon relabeled.
- CX52850/CX52650 core ARM/RISC-V datasheet/teardown -> nothing; Beken keeps datasheets confidential.
- "CX52650" rebranded Beken / BK2535 / BK5863 -> BK2535 is 8051+2.4G (different part); no CX52650 register spec. Dead end for a datasheet.
- 博通集成 匠盟 Compx -> compx.com.cn (Shenzhen 匠盟, founded 2015) and Beken (博通集成, 603068). Company identities fixed.
- CX52850 nRF52840 rebranded -> "CX52850 is rebranded Beken BK3633" (again), plus Fantech Aria II teardown. Consistent.
- reddit/X CompX CX52850 nordic -> haus/NVNT tweets: CX52850 idle-delay reputation; CompX-firmware mice cross-pair 4K dongles (unverified).
- compx.com.cn site (curl -k, expired cert): home/product/news pages; news page lists the demo driver + Compx-4K-Dongle .bin. No chip part numbers or datasheets on the site.
NOTHING-NEW after ~5 differently-angled CX52650N queries: no public datasheet or open flashing tool for the CX52650N exists; it is a Beken 2.4G SoC updated only via Compx's proprietary USB-HID path.

## Compx HID protocol / "EnterUsbUpdateMode" / "EnterMTKMode" / open projects

- "EnterUsbUpdateMode" "EnterMTKMode" -> nothing (fwupd, MTK phones, unrelated).
- "MTK mode" keyboard dongle / EnterMTK / 进入MTK模式 -> only MediaTek phone tooling. No match for the Compx keyboard command. Dead end.
- "EncryptionData" "DongleEnterPair" / "PCDriverStatus" -> nothing (honey encryption, dongles). The literal Compx string set is not indexed.
- github "0x3554" mouse vendor id hid -> Linux hid-ids.h (KYSONA/VXE 0x3554), OpenMouse PRs, Pulsar LKML driver. KEY entry point.
- OpenVXE / VXE R1 open source -> OpenMouse-Project (openmouse, mouse-protocol). Cloned both. Found Compx report-8 protocol, command 0x0d enterUsbUpgradeMode, 0x5a/0x5b upgrade status, full firmware-package parser, Lamzu DFU boot PIDs. BEST protocol source.
- "ComUsbUpgradeFile" / "pid_f401" 3554 -> nothing indexed; also surfaced an NVNT tweet about CompX cross-compat.
- "UsbUpgrade" Compx pid f406/f401/FB16 -> generic HID bootloader docs, nothing Compx-specific.
- Pulsar LKML v8 patch (ratatoskr.run) -> USB_PAYLOAD_LEN 17, CMD_HID_REPORT_ID 0x08, CHECKSUM_MAGIC 0x55; battery only, no firmware update. Confirms protocol family + 0x55 checksum from kernel source.
- linux-hardware.org 3554:fc00 -> "Compx 2.4G Receiver"; also 25a7:fa03. Vendor string confirmation.
NOTHING-NEW: no open-source project implements the CompX firmware-FLASH path; all (OpenMouse, hid-pulsar, libratbag-style) do config/telemetry only. EnterMTKMode / SetDeviceVidPid / SetDeviceDescriptorString have zero public references.

## Compx 4K dongle upgrade file (primary artifact)

- Downloaded Compx-4K-Dongle-1101-E8AF2ED4-V1.27-20230824.bin from the compx.com.cn news page (nd.jsp?id=11). Parsed statically: two "ComUsbUpgradeFile" images, IC "CH32V305" (boot pid_f401) and IC "NRF52810" (boot pid_f510 col07), version 0x127, checksum-to-0x55 reset frames = command 0x0d. Payload entropy ~7.99 bits/byte (encrypted/compressed). CRC location per OpenMouse is zero here; prepare-tail bytes matched no standard CRC/sum tested. KEY: 4K receiver is dual-MCU (WCH + Nordic); update is USB-HID via Compx boot PIDs, not the WCH factory ISP.
- A later curl and one WebSearch were blocked by the session's permission classifier after this download; no further vendor binaries fetched.

## Nordic nRF52833 update mechanisms

- nRF Desktop config channel + hid_configurator -> cloned sdk-nrf (sparse), read config_channel.rst, dfu.rst, bootloader_dfu.rst, hid_forward.rst, fwupd.rst, the hid_configurator scripts, samples/bootloader, subsys/bootloader. Full DFU-over-HID wire format (feature report 6, 30 B), dongle forwarding, B0 vs MCUboot, single/dual bank, B0 highest-version slot selection, monotonic-counter rollback opt-in.
- fwupd nordic_hid plugin (README + fwupd.rst) + issue 5738 (dongle VID/PID GUID mismatch). Linux path via hidraw ioctls.
- nRF5 SDK Secure DFU (serial/USB CDC/BLE) doc snippets + DiUS mirror source (nrf_dfu_ver_validation.c, nrf_dfu_validation.c): fw_version_ok, NRF_DFU_APP_DOWNGRADE_PREVENTION, ECDSA-P256 signature mandatory, is_debug skip, use_single_bank. KEY for downgrade + signing.
- Gazell/ESB DFU relay -> DevZone threads: Gazell+bootloader OTA exists (nRF51/nRF24LU1), a dev got ~822 Hz with Gazell on a mouse; no turnkey ESB/Gazell DFU-relay reference. nRF Desktop uses BLE (not ESB/Gazell) for its config-channel forwarding; grep of the checked-out nRF Desktop docs for esb/gazell = 0 hits.
- Nordic docs host (infocenter/docs.nordicsemi.com) returns 403 to the fetch tool throughout; used rst sources and search snippets.

## WCH CH32V305 (4K receiver MCU)

- wchisp CH32V305 4348:55e0 -> cloned ch32-rs/wchisp (commit cefd870). Full ISP protocol (a1..c5), XOR key from chip UID, no version field, unprotect=0xa5, 56-byte chunks, final empty write. CH32V305 defined (chip_id 0x50) not "tested".
- isp55e0 protocol.txt (curl) + README -> capture-based confirmation; XOR key last byte = first+chip_type; write-protect handling.
- CH32V305 software jump to bootloader without BOOT0 -> only STM32/NXP analogs; wchisp's software-jump note is CH32V00x-only.
- CH32V30x reference manual (chipdip mirror PDF) + datasheet (github mirror PDF) -> 28KB factory bootloader, USB+USART1 reprogram, boot select via BOOT0/BOOT1 pins ONLY; manufacturer config factory-locked. No documented software bootloader entry for V30x.
- CH32V305 RDPR/WRP brick/recover -> STM32/NXP analogs + basilhussain ch32v003-bootloader-docs (V003 only). CH32V305-specific unbrick not found; read-protect triggers erase, WRP can lock.
- wchisp issue 26 (verify mismatch, CH582) -> generic flash/power issue, not version-related.
NOTHING-NEW: WCH ISP has no version check (downgrade unrestricted at that layer); sealed-dongle ISP entry method is the open question; the Compx 4K file uses pid_f401 not 4348:55e0, so entry is Compx-software-driven.

## Downgrade / revert (scope extension)

- MCUboot design.md -> downgrade prevention (SW: MCUBOOT_DOWNGRADE_PREVENTION + overwrite-only; HW: security counter, same-counter allowed), swap/revert anti-brick, direct-xip highest-version. KEY.
- NCS bootloader_downgrade_protection.rst + image_versions.rst + B0 main.c -> SW downgrade prevention (semantic version, overwrite-only, caution: removes fallback), HW monotonic counter (opt-in, limited slots), B0 boots highest-version slot, counter rollback under CONFIG_SB_MONOTONIC_COUNTER_ROLLBACK_PROTECTION.
- nRF5 SDK NRF_DFU_APP_DOWNGRADE_PREVENTION -> DevZone "remove firmware version checking", NCS DFU page snippet "may reject due to downgrade prevention", "disable by setting NRF_DFU_APP_DOWNGRADE_PREVENTION to 0".
- Mouse/dongle firmware version coupling -> Pulsar (dongle first, else re-pair), Lamzu (re-pair after 4K update), ATK (update receiver too; step1/step2 error codes; multi-MCU), MCHOSE (both must match, some need dual-cable simultaneous flash), VGN 4K dongle manual (must update mouse to latest to pair). Convergent: mouse+receiver firmware are a coupled pair; mismatch breaks 2.4G / needs re-pair.
- Vendor downgrade/rollback UIs -> ATK HUB / VXE HUB / Lamzu / VGN / Lofree all push latest only; none expose a downgrade or host per-version firmware files ("connect the HUB to get the latest"; discontinued-product firmware only via customer service). No vendor documents an official downgrade path.
- Endgame Gear "Is my mouse bricked?" -> button-held (LMB+RMB or LMB+MB+RMB) bootloader entry + reflash. Real mouse brick-recovery gesture.
- sameid/samkey84 -> Lofree's own firmware update bricked a Flow 2 keyboard, unrecoverable by support. Caution about Lofree updater quality.
NOTHING-NEW after 5 differently-angled downgrade queries: vendors do not offer or document downgrade; per-version firmware files are not publicly hosted; the bootloader-level ability to downgrade is (nRF) build-flag-dependent and (WCH ISP) unrestricted, but no vendor exposes it.

## Dead ends / not found

- No public CX52650N or CX52850 datasheet, register map, or bootloader spec (Beken confidential).
- No public reference to EnterMTKMode, SetDeviceVidPid (11), SetDeviceDescriptorString (12), ReadCIDMID as command 16 by that name (OpenMouse uses 0x10 for CID/MID read); these remain undocumented Compx factory commands.
- The Compx 4K .bin payload cipher and exact chunk-transfer handshake are not derivable statically (high entropy); require capturing the official updater.
- No confirmation of which bootloader build (bank mode, downgrade flag) the actual HYPACE nRF52833 or its receivers ship with; only a live capture or sacrificial-unit test can settle it.
- Nordic docs site and several vendor support sites 403 the fetch tool; worked around with curl and search snippets where possible.
