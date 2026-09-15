URL: https://github.com/ch32-rs/wchisp
Retrieval date: 2026-09-15
Source type: open-source host tool source code (Rust), reverse-engineered protocol implementation for the WCH factory ISP bootloader

# wchisp: WCH USB/UART ISP protocol as implemented in Rust

## Acquisition

`git clone --depth 1 https://github.com/ch32-rs/wchisp.git` into `research-bin/chips/wchisp` (scratch), commit `cefd8707df345f1fbd7795e15367281f440bbf05`. Files read: `README.md`, `devices/0x17-CH32V30x.yaml`, `src/protocol.rs`, `src/constants.rs`, `src/flashing.rs`, `src/transport/usb.rs`. Nothing was built or run.

## Excerpts (verbatim)

### README.md

> This tool is for **USB** and **UART** ISP, not for use with WCH-Link.

```text
# /etc/udev/rules.d/50-wchisp.rules
SUBSYSTEM=="usb", ATTRS{idVendor}=="4348", ATTRS{idProduct}=="55e0", MODE="0666"
```

```console
> wchisp info
14:51:24 [INFO] Chip: CH32V307VCT6[0x7017] (Code Flash: 256KiB)
14:51:24 [INFO] Chip UID: 30-78-3e-26-3b-38-a9-d6
14:51:24 [INFO] BTVER(bootloader ver): 02.60
14:51:24 [INFO] Code Flash protected: false
RDPR_USER: 0x9F605AA5
  [7:0] RDPR 0b10100101 (0xA5)
    `- Unprotected
```

> Also note that ISP bootloader entry cannot be controlled via external pin state at reset. Instead, user application code must instruct device to enter the bootloader via setting `FLASH_STATR.MODE` flag and performing a software reset (see `PFIC_CFGR`).

(That note is in the "CH32V00x Notes" section and is stated for CH32V00x only.)

Tested list includes "CH32V307 (VCT6, RCT6)" and "CH32V203"; CH32V305 is not listed as tested.

### devices/0x17-CH32V30x.yaml

```yaml
name: CH32V30x Series
mcu_type: 7
device_type: 0x17
support_net: false
support_usb: true
support_serial: true
...
      - bit_range: [7, 0]
        name: RDPR
        description: Read Protection. 0xA5 for unprotected, otherwise read-protected(ignoring WRP)
...
  - offset: 0x08
    name: WRP
    # Each bit represents 4K bytes (16 pages) to store the write protection status
...
variants:
  - name: CH32V305RBT6
    chip_id: 0x50
    flash_size: 128K
```

### src/constants.rs

```rust
pub const MAX_PACKET_SIZE: usize = 64;
pub const SECTOR_SIZE: usize = 1024;
...
pub mod commands {
    pub const IDENTIFY: u8 = 0xa1;
    pub const ISP_END: u8 = 0xa2;
    pub const ISP_KEY: u8 = 0xa3;
    pub const ERASE: u8 = 0xa4;
    pub const PROGRAM: u8 = 0xa5;
    pub const VERIFY: u8 = 0xa6;
    pub const READ_CONFIG: u8 = 0xa7;
    pub const WRITE_CONFIG: u8 = 0xa8;
    pub const DATA_ERASE: u8 = 0xa9;
    pub const DATA_PROGRAM: u8 = 0xaa;
    pub const DATA_READ: u8 = 0xab;
    pub const WRITE_OTP: u8 = 0xc3;
    pub const READ_OTP: u8 = 0xc4;
    pub const SET_BAUD: u8 = 0xc5;
}
```

### src/protocol.rs

```rust
    /// Send ISP key seed to MCU.
    /// Return checksum of the XOR key(1 byte sum).
    ///
    /// The detailedd key algrithm:
    ///
    /// - sum Device UID to a byte, s
    /// - initialize XOR key as [s; 8]
    /// - select 7 bytes(via some rules) from generated random key
    /// - `key[0] ~ key[6] ^= corresponding selected byte`
    /// - `key[7] = key[0] + chip_id`
    ///
    /// In many open source implementations, the key is initialized as [0; N],
    /// which makes it easier to do the calculation
    IspKey { key: Vec<u8> },
    /// Erase the Code Flash.
    ///
    /// Minmum sectors is either 8 or 4 depends on device type.
    Erase { sectors: u32 },
    /// Program the Code Flash.
    ///
    /// `data` is xored with the XOR key.
    /// `padding` is a random byte(Looks like a checksum, but it's not)
```

```rust
                buf.extend_from_slice(b"MCU ISP & WCH.CN");
```

### src/flashing.rs

```rust
    // unprotect -> erase -> flash -> verify -> reset
    /// Program the code flash.
    pub fn flash(&mut self, raw: &[u8]) -> Result<()> {
        let key = self.xor_key();
        let key_checksum = key.iter().fold(0_u8, |acc, &x| acc.overflowing_add(x).0);

        // NOTE: use all-zero key seed for now.
        let isp_key = Command::isp_key(vec![0; 0x1e]);
        ...
        const CHUNK: usize = 56;
        ...
        // NOTE: require a write action of empty data for success flashing
        self.flash_chunk(address, &[], key)?;
```

```rust
    fn xor_key(&self) -> [u8; 8] {
        let checksum = self
            .chip_uid()
            .iter()
            .fold(0_u8, |acc, &x| acc.overflowing_add(x).0);
        let mut key = [checksum; 8];
        key.last_mut()
            .map(|x| *x = x.overflowing_add(self.chip.chip_id).0);
        key
    }
```

```rust
    /// Unprotect code flash.
    pub fn unprotect(&mut self, force: bool) -> Result<()> {
        ...
        config[0] = 0xa5; // code flash unprotected
```

### src/transport/usb.rs

```rust
const ENDPOINT_OUT: u8 = 0x02;
const ENDPOINT_IN: u8 = 0x82;

const USB_TIMEOUT_MS: u64 = 5000;
...
            (desc.vendor_id() == 0x4348 || desc.vendor_id() == 0x1a86)
                && desc.product_id() == 0x55e0
```

## Negative finding

The command set above has no command or field that carries a firmware version. Program takes an address and XOR-obfuscated bytes.
