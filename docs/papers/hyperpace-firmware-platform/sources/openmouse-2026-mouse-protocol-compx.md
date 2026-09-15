URL: https://github.com/OpenMouse-Project/mouse-protocol
Also: https://github.com/OpenMouse-Project/openmouse (commit `0a3531ca6e3eb56dcc8f98ad847441fd2eb5a954`), and PR pages seen in search: https://github.com/OpenMouse-Project/mouse-protocol/pull/44, https://github.com/OpenMouse-Project/openmouse/pull/235
Retrieval date: 2026-09-15
Source type: community open-source driver code plus hardware test notes (WebHID drivers for Compx-ODM mice)

# OpenMouse mouse-protocol: Compx (0x3554) command table, firmware package format, bootloader PIDs

## Acquisition

`git clone --depth 1 https://github.com/OpenMouse-Project/mouse-protocol.git`, commit `a915bb3811e51124a2d9428245f7d9baafcc7705` (commit date 2026-09-15), into `research-bin/chips/mouse-protocol` (scratch). Files read: `src/atk/index.ts`, `docs/atk-testing.md`, `src/lamzu/index.ts`, `docs/lamzu-inca-testing.md`, `src/compx/codec.ts`, `src/drivers/vendors.ts`. Nothing was built or run. Em-dash characters inside quoted text are normalized to " - " below (house style); no other change.

## Excerpts (verbatim)

### src/atk/index.ts: Compx command ids

```ts
/** Command ids used by the COMPX configuration transport. */
export const ATK_COMPX_COMMAND = {
  getWirelessMouseOnline: 0x03,
  setWirelessDonglePair: 0x05,
  getWirelessDonglePairResult: 0x06,
  restoreFactory: 0x09,
  enterUsbUpgradeMode: 0x0d,
  getCurrentConfig: 0x0e,
  setCurrentConfig: 0x0f,
  dongleExitPair: 0x13,
  getDongleVersion: 0x1d,
  reportMouseUpgradeError: 0x5a,
  reportMouseUpgradeStatus: 0x5b,
} as const;
```

```ts
  const sum = 0x08 + payload.subarray(0, 15).reduce((total, byte) => total + byte, 0);
  payload[15] = (CHECKSUM_TOTAL - (sum & 0xff)) & 0xff;
```

### src/atk/index.ts: Compx firmware package parser

```ts
export const ATK_COMPX_FIRMWARE_PAYLOAD_OFFSET = 8192;
export const ATK_COMPX_FIRMWARE_HEADER_LENGTH = 720;
const ATK_FIRMWARE_FIELD_LENGTH = 64;
const ATK_FIRMWARE_FIELD_BASE = 23;
```

```ts
/** CRC-32 register state used by COMPX packages: init 0xffffffff, no final xor. */
```

```ts
  const headerLength = view.getUint32(4, true);
  const firmwareLength = view.getUint32(8, true);
  ...
  const prepareCommand = commandField(data, 7);
  const payloadCrc = new DataView(prepareCommand.buffer, prepareCommand.byteOffset, prepareCommand.byteLength).getUint32(19, false);
  ...
    headerCrc: view.getUint32(0, true),
    headerLength,
    firmwareLength,
    nextFileAddress: view.getUint32(12, true),
    version: view.getUint32(16, true).toString(16),
    deviceType: view.getUint8(20),
    cid: view.getUint8(21),
    mid: view.getUint8(22),
    fileId: asciiField(data, 0),
    icName: asciiField(data, 1),
    bootInput: firmwareEndpoint(asciiField(data, 2)),
    bootOutput: firmwareEndpoint(asciiField(data, 3)),
    normalInput: firmwareEndpoint(asciiField(data, 4)),
    normalOutput: firmwareEndpoint(asciiField(data, 5)),
    resetCommand: commandField(data, 6),
    prepareCommand,
    downloadCommand: commandField(data, 8),
    sensorName: asciiField(data, 9),
    productName: asciiField(data, 10),
```

### docs/atk-testing.md

> Use the vendor configuration interface (`usagePage 0xff02`, `usage 0x02`). It
> uses report ID `0x08` with a 16-byte payload.

> - USB: VID/PID `0x3554:0xf58f`, product `VXE R1SE+`, firmware/bcdDevice 3.15.

> The Nordic receiver was identified over USB as `0x3554:0xf58e`, product `VXE
> Mouse 1K Dongle`, firmware/bcdDevice 1.10.

> Dongle-version command `0x1d` returned status `1`,
> declared zero payload bytes

> Pairing was validated with the mouse cable unplugged and the mouse switched to
> 2.4 GHz mode. After command `0x05`, holding left click, wheel click, and right
> click until the indicator flashed completed the exchange.

> A prior
> attempt without the physical button gesture also reached status 2 but remained
> offline, so status 2 alone is not success.

> The official R1 SE+ 3.15 COMPX package is 261,382 bytes. Its 720-byte logical
> header sits in an 8,192-byte header area followed by a 253,190-byte payload.
> The header identifies normal endpoint `0x3554:0xf58f`, boot endpoint
> `0x3554:0xf406`, IC `CX52850P`, sensor `3395se`, and version `0x315`.

> The prepare-command descriptor stores the raw CRC-32 register state (initial
> value `0xffffffff`, polynomial `0xedb88320`, no final XOR) in big-endian order.
> For version 3.15 it is `0x4026708e`; for version 3.14 it is `0xab88f525`.
> OpenMouse's package parser checks this payload CRC and package bounds. The
> vendor upgrader reads the header's `headCRC` field but does not appear to
> validate it, so OpenMouse does not claim that field as an integrity check.

> Firmware parsing is read-only. Entering boot mode, erase preparation, chunk
> transfers, factory reset, and macro clearing remain intentionally unavailable.

### src/lamzu/index.ts

```ts
 * Deliberately absent: 0x000a and 0x0002, which Aurora lists as
 * `DeviceBLPID` and `Receiver4K8KBLPID` - the DFU bootloader identities the
 * mouse and dongle take while flashing firmware. They never speak this
 * protocol and must not be offered in the picker.
```

### docs/lamzu-inca-testing.md

> | `0x000a` | Mouse DFU bootloader - **not a mouse** | Vendor table only |
> | `0x0002` | Receiver DFU bootloader - **not a mouse** | Vendor table only |

> The two bootloader ids are the identities the mouse and dongle take while
> firmware is being flashed. They do not speak this protocol and are excluded
> from the catalog on purpose.

### src/compx/codec.ts (a different, 64-byte Compx framing used by WLMouse and Lamzu receivers)

```ts
/** Shared page-command framing used by WLMouse and Lamzu receivers. */
export const COMPX_REPORT_ID = 0;
export const COMPX_PACKET_LENGTH = 64;
export const COMPX_HEADER_LENGTH = 6;
export const COMPX_STATUS = { request: 0x00, pending: 0xa0, ok: 0xa1, unsupported: 0xa2, busy: 0xa3 } as const;
```

### src/drivers/vendors.ts

```ts
  // Lamzu's Atlantis generation lands here too - 0x3554 is CompX's ODM id, and
  // this filter already surfaces it, so it needs no entry of its own.
```

```ts
  // 0x373e is the shared CompX ODM vendor id behind Lamzu, CRDRAKO, and
  // Attack Shark.
```

## Local negative search

`grep -rniI -E "EnterUsbUpdate|MTKMode|ReadCIDMID|SetDeviceVidPid|DescriptorString"` over both OpenMouse repos: no hits for `EnterMTKMode`, `SetDeviceVidPid` or `SetDeviceDescriptorString`. CID/MID read is command `0x10` (`docs/atk-testing.md`).
