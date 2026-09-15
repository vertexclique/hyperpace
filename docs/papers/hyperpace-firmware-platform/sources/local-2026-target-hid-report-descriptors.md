Source: /sys/class/hidraw/hidraw{8,9,10,11,12,13}/device/report_descriptor on the development machine (the kernel's cached copy of the descriptors read at enumeration)
Retrieved: 2026-09-15
Source type: primary data from the target hardware (operator's mouse and receiver), read from sysfs only. No report was sent to or read from the device.
Decoder: short Python HID item parser (main, global, local items; collections tracked by depth). Output kept at scratchpad/hid_descriptors_decoded.txt.

# USB interfaces

| hidraw | PID | USB interface | Descriptor bytes | Role |
|---|---|---|---|---|
| hidraw8 | 0xFB16 (receiver) | 1.0 | 57 | keyboard (0x0001:0x0006), input 8 B, output 1 B |
| hidraw9 | 0xFB16 (receiver) | 1.1 | 266 | vendor interface, 8 top-level collections |
| hidraw10 | 0xFB16 (receiver) | 1.2 | 87 | mouse (0x0001:0x0002), input 7 B |
| hidraw11 | 0xFB14 (wired) | 1.0 | 57 | keyboard, same descriptor as hidraw8 |
| hidraw12 | 0xFB14 (wired) | 1.1 | 177 | vendor interface, 7 top-level collections |
| hidraw13 | 0xFB14 (wired) | 1.2 | 87 | mouse, same descriptor as hidraw10 |

# Vendor interface top-level collections (decoded)

| TLC | Usage page:usage | Reports | Windows OutputReportByteLength | Windows InputReportByteLength |
|---|---|---|---|---|
| 0 | 0xff05:0x0000 | input 16 = 7 B | 0 | 8 |
| 1 | 0xff03:0x0000 | input 2 = 7 B | 0 | 8 |
| 2 | 0x000c:0x0001 | input 5 = 2 B | 0 | 3 |
| 3 | 0x0001:0x0080 | input 3 = 1 B | 0 | 2 |
| 4 | 0xff02:0x0002 | input 8 = 16 B, output 8 = 16 B | 17 | 17 |
| 5 | 0xff04:0x0002 | feature 6 = 7 B | 0 | 0 |
| 6 | 0xff06:0x0002 | input 9 = 48 B, output 9 = 48 B | 49 | 49 |
| 7 (receiver only) | 0x0001:0x0002 | input 11 = 7 B | 0 | 8 |

Windows byte lengths are derived with the HIDP_CAPS rule (longest report of that kind in the collection plus one report-id byte), see `microsoft-2024-hidp-caps`.

# Raw descriptor, vendor interface, receiver (hidraw9, 266 bytes)

```
0605ff0900a10185100900150026ff00750895078102c00603ff0900a10185020900150026ff00750895078102c0050c0901a10185051500263c0219002a3c02751095018100c005010980a10185031981298315002501950375018102950175058101c00602ff0902a1018508150026ff00750895100902810009029100c00604ff0902a10185060902150026ff0075089507b102c00606ff0902a1018509150026ff00750895300902810009029100c005010902a101850b0901a1000509190129051500250195057501810295017503810105010930093116008026ff7f751095028106c0a100050109381581257f750895018106c0a100050c0a3802950175081581257f8106c0c0
```

# Raw descriptor, vendor interface, wired (hidraw12, 177 bytes)

```
0605ff0900a10185100900150026ff00750895078102c00603ff0900a10185020900150026ff00750895078102c0050c0901a10185051500263c0219002a3c02751095018100c005010980a10185031981298315002501950375018102950175058101c00602ff0902a1018508150026ff00750895100902810009029100c00604ff0902a10185060902150026ff0075089507b102c00606ff0902a1018509150026ff00750895300902810009029100c0
```

# Observations

1. Report 8 is the only report in collection 0xff02:0x0002, so a Windows write of report 8 is 17 bytes (id + 16). Report 9 lives in its own collection 0xff06:0x0002 (49 bytes on Windows). The collection layout matches the Lamzu Atlantis Mini 4K capture in `openmouse-2026-lamzu-atlantis-testing` (same ODM, same VID 0x3554), including the config channel being Col05 0xff02/0x0002.
2. The receiver's vendor interface additionally declares a Mouse collection (0x0001:0x0002, report 11). The wired vendor interface does not; its only page-0x01 collection is System Control (0x0001:0x0080).
3. Implication, unverified: macOS builds one IOHIDDevice per USB interface with all collection usages in its usage-pair list. If Input Monitoring is gated on a mouse usage anywhere in that list (`apple-2026-iohidfamily-tcc-gating-source`), opening the receiver's vendor interface on macOS may require Input Monitoring while the wired interface would not. Needs a macOS run to confirm; it cannot be settled from Linux.
4. Both receiver and wired vendor interfaces carry identical collections 0 to 6, so one code path can serve both links.
