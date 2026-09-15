# SUMMARY: wchisp README, WCH USB ISP bootloader access

## Claim
The WCH USB ISP bootloader (4348:55e0):
- Windows: needs WCH's own CH375 driver and DLL, or the WinUSB driver installed with Zadig.
- Linux: needs a `SUBSYSTEM=="usb"` udev rule (MODE 0666 or a group).
- Build: wchisp uses libusb.

## Method
Tool README.

## Result
- A WCH ISP bootloader is not HID. It needs a vendor driver or WinUSB on Windows and a usb-subsystem rule on Linux.

## Evidence tier
1 for wchisp's own requirements, 3 for general WCH bootloader facts.

## Performance
Not applicable.

## Correctness
- Proven for this bootloader ID.
- Whether HYPACE's update mode uses a WCH ISP bootloader at all is unknown. PROTOCOL.md does not say, and the Lofree web flasher uses WebHID, which suggests the update path may stay HID-class. That is unverified.

## Relevance to hyperpace
- If the update mode turns out to be a WCH ISP bootloader, Windows users would need a driver install. That is a major UX cost; nusb plus WinUSB is the Rust path.
- A hardware experiment (enter update mode, observe the re-enumerated VID/PID and interface class) must come before designing this.
- Linux rules would need a usb-subsystem line in addition to the hidraw lines.
