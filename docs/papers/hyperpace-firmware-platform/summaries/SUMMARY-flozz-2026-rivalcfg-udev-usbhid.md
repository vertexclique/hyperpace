# SUMMARY: rivalcfg, udev rules and interface selection

## Claim
rivalcfg, a cross-platform SteelSeries mouse CLI built on the Python HIDAPI binding:
- selects the device interface by `interface_number` from `hid.enumerate(vid, pid)` and opens it by path;
- on Linux, `--update-udev` (as root) writes `/etc/udev/rules.d/99-steelseries-rival.rules` with `MODE="0666"` for both the hidraw and usb subsystems.

Its docs say macOS may need no setup.

## Method
Source code (clone at f16c521) plus documentation snippets from search.

## Result
- Interface-number selection works for SteelSeries, where the vendor interface is a separate USB interface.
- MODE=0666 gives every local and remote user access. The file name 99- sorts after 73, so a uaccess tag would not work there; 0666 sidesteps that.

## Evidence tier
1 (its own source), 3 for the documentation snippets.

## Performance
Not applicable.

## Correctness
- Proven for rivalcfg.
- 0666 is a security trade-off: any user, including remote sessions, can reconfigure the mouse.
- The macOS "no requirement" note predates the current TCC gating, and SteelSeries vendor interfaces may not include mouse usages.

## Relevance to hyperpace
- Prefer TAG+="uaccess" in a 70- file over MODE="0666" in a 99- file.
- Interface number alone is not enough on Windows, where several TLCs share MI_01. Combine it with usage page and usage.
