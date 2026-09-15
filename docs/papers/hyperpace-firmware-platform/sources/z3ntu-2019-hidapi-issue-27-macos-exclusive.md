URL: https://github.com/libusb/hidapi/issues/27
Retrieved: 2026-09-15
Source type: GitHub issue with maintainer (Youw) comments; CLOSED 2022-04-23.
Retrieval method: `gh issue view 27 -R libusb/hidapi --json ...` (verbatim, comments truncated at 1000 chars).
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

`mac: Option to open device in non-exclusive mode | z3ntu | 2019-06-18T18:15:50Z | CLOSED closed=2022-04-23T11:16:42Z`

> Example: If you open a mouse currently (on macOS), the mouse input doesn't reach the OS, so you can't move the cursor with the mouse anymore while the device is opened in hidapi. That can be solved by not opening the device in exclusive mode

z3ntu, 2019-06-19:
> On Windows it seems it's not possible to open some devices in exclusive mode. On Mac both is possible, so no fallback is possible on the hidapi side.

Youw, 2020-02-24:
> 1) On Windows `hid_open/_path` is non-exclusive in any case, so #148 will make the behavior consistent, which is great;
> ...
> It will be very convenient for many users who find exclusive mode on macOS inconvenient/unusable;

Resolution evidence in current source (mac/hid.c on master): `hid_darwin_set_open_exclusive(1); /* Backward compatibility */` in `hid_init`, and the public setter `hid_darwin_set_open_exclusive(int open_exclusive)`. hidapi-rs exposes it as `HidApi::set_open_exclusive` and the `macos-shared-device` feature.

Related: issue 212 (smartavionics, 2020-11-19, closed 2021-06-10) "MacOS Big Sur IOError on open". Maintainer Youw, 2021-01-26:
> Two applications (regardless - a user applications, or a kernel driver) cannot access same HID device on macOS at the same time if at least one of them tries to open the device in exclusive mode.

and after a non-exclusive test build still failed:
> That only means, that 3dconnection driver openes the device in exclusive mode too, and until it is active - no other application will able to access it.
