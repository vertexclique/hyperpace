URL: https://github.com/ch32-rs/wchisp (README; repository pushed 2026-04-27)
Retrieved: 2026-09-15
Source type: README of a Rust command-line tool for the WCH USB ISP bootloader.
Retrieval method: `gh api repos/ch32-rs/wchisp/readme` (verbatim excerpts via grep).
Character note: none needed.

# Excerpts

> ### Note for Windows
>
> If you are using Windows, you may need to install the CH375DLL64.dll if you do not have it in your system. If you encounter a "CH375DLL64.dll not found" error, please download it from the WCH official website and put the dll next to this executable. ... When you use the official WCH driver you installed with IDE or WCHISPTOOL, CH375DLL64.dll allows you to program the target chips without changing the driver.
>
> It is also OK to use the WinUSB driver with Zadig.

> ### Note for Linux
>
> If you are using Linux, you need to set the udev rules for your device.
>
> ```text
> # /etc/udev/rules.d/50-wchisp.rules
> SUBSYSTEM=="usb", ATTRS{idVendor}=="4348", ATTRS{idProduct}=="55e0", MODE="0666"
> # or replace MODE="0666" with GROUP="plugdev" or something else
> ```

Install section:
> # install libusb for your platform
