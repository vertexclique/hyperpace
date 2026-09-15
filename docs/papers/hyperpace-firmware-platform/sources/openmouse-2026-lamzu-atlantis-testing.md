URL: https://github.com/OpenMouse-Project/mouse-protocol/blob/main/docs/lamzu-atlantis-testing.md ; driver code https://github.com/OpenMouse-Project/mouse-protocol/tree/main/src/drivers
Retrieved: 2026-09-15
Source type: Community hardware capture notes (Windows 11, real device) and TypeScript WebHID driver source in the OpenMouse protocol library (repo created 2026-08-09, pushed 2026-09-14, 16 stars, no license declared in GitHub metadata).
Retrieval method: `git clone --depth 1 https://github.com/OpenMouse-Project/mouse-protocol` (HEAD a915bb3, 2026-09-15); files read verbatim.
Character note: em-dash characters in quoted text were replaced with ASCII hyphens; nothing else changed.

# docs/lamzu-atlantis-testing.md

> Hardware report from a **Lamzu Atlantis Mini 4K**, firmware `1.24`, on its
> cable, Windows 11.

> Vendor id `0x3554` is CompX's shared ODM id - the same one the Pulsar 4K
> receiver, VGN and Teevolution units already in this repository use, and the
> one `ATK_COMPX_PRODUCT_IDS` covers for VXE.

| Interface | Collection | Usage page | Usage | Report 8 | Role |
| --- | --- | --- | --- | --- | --- |
| MI_00 | - | 0x0001 | 0x0006 | no | Keyboard |
| MI_01 | Col01 | 0xff05 | 0x0000 | no | Vendor - rejects writes |
| MI_01 | Col02 | 0xff03 | 0x0000 | no | Vendor - rejects writes |
| MI_01 | Col03 | 0x000c | 0x0001 | no | Consumer control |
| MI_01 | Col04 | 0x0001 | 0x0080 | no | System control |
| **MI_01** | **Col05** | **0xff02** | **0x0002** | **yes** | **Config channel** |
| MI_01 | Col06 | 0xff04 | 0x0002 | no | Vendor - feature report 6 only |
| MI_02 | - | 0x0001 | 0x0002 | no | Mouse |

> Chrome's own view of the same mouse, from `navigator.hid.getDevices()`, is
> what the driver's `isSupported` actually gates on. It differs from the
> platform view above - Windows exposes no report ids for several collections
> that Chrome does - and it confirms report 8 is declared in both directions on
> the config collection:

| Usage page | Usage | Input | Output | Feature |
| --- | --- | --- | --- | --- |
| 0xff05 | 0x00 | 16 | - | - |
| 0xff03 | 0x00 | 2 | - | - |
| 0x000c | 0x01 | 5 | - | - |
| 0x0001 | 0x80 | - | - | - |
| **0xff02** | **0x02** | **8** | **8** | - |
| 0xff04 | 0x02 | - | - | 6 |

> The config channel is the `0xff02`/`0x0002` collection, which matches the
> `Interfaceid=1` in the shipped `Config.ini` of Lamzu's Windows app. Every
> other vendor collection rejects `WriteFile` with `Incorrect function`.

> The `0xff04` collection is a red herring worth recording: it answers
> `HidD_GetFeature` on report id 6 with a 32-byte snapshot and accepts
> `HidD_SetFeature`, but it does not speak this protocol.

> Report id 8, 17 bytes on the wire (16 to WebHID, which supplies the id).

> | `0x04` | Battery | `64 01 10 82` - 100%, charging, 4,226 mV |

> - **The battery reply lies about its length.** Byte 4 says `0x02` while four
>   bytes follow: percent, charging flag, then the millivolts. Decoding by the
>   declared length silently drops the voltage.

# src/drivers/teevolution/hid.ts (collection gate)

```ts
      && device.collections.some((collection) =>
        collection.inputReports.length === 1
        && collection.outputReports.length === 1
        && collection.inputReports[0]?.reportId === TEEVOLUTION_REPORT_ID
        && collection.outputReports[0]?.reportId === TEEVOLUTION_REPORT_ID);
```
```ts
    if (event.reportId === TEEVOLUTION_REPORT_ID && bytes[0] === this.responseWaiter?.command) {
```
```ts
        reject(new Error(`The Teevolution mouse did not answer command 0x${command.toString(16).padStart(2, "0")}.`));
```

# src/drivers/pulsar/pulsar-hid.ts

```ts
// The Pulsar 4K Wireless Receiver is sold as a Pulsar product but enumerates
// under the shared Teevolution/VGN vendor id (0x3554) and speaks the same
// report-8 16-byte protocol as Pulsar receivers.
```

# docs/lamzu-inca-testing.md (a second CompX-adjacent Windows capture)

| Interface | Collection | Usage page | Usage | Report 8 | Role |
| --- | --- | --- | --- | --- | --- |
| MI_01 | Col04 | 0xffa0 | 0x0001 | none | Vendor - rejects `HidD_SetFeature` |
| MI_01 | Col05 | 0xffff | 0x0001 | none | Vendor - rejects `HidD_SetFeature` |
