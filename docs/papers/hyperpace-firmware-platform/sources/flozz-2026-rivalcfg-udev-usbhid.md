URL: https://github.com/flozz/rivalcfg (HEAD f16c521, 2026-08-16) ; docs https://flozz.github.io/rivalcfg/
Retrieved: 2026-09-15
Source type: Source code of a cross-platform SteelSeries mouse configuration CLI (Python, uses the `hid` binding to HIDAPI); documentation via search snippets.
Retrieval method: `git clone --depth 1 https://github.com/flozz/rivalcfg`; files read verbatim. Doc sentences from WebSearch snippets are marked.
Character note: none needed.

# rivalcfg/udev.py

```python
#: Path to the udev rules file
RULES_FILE_PATH = "/etc/udev/rules.d/99-steelseries-rival.rules"
```
```python
    rules += "#     rivalcfg --update-udev\n\n"

    for profile in PROFILES.values():
        rules += "# %s\n" % profile["name"]
        rules += (
            'SUBSYSTEM=="hidraw", ATTRS{idVendor}=="%04x", ATTRS{idProduct}=="%04x", MODE="0666"\n'
            % (profile["vendor_id"], profile["product_id"])
        )
        rules += (
            'SUBSYSTEM=="usb", ATTRS{idVendor}=="%04x", ATTRS{idProduct}=="%04x", MODE="0666"\n\n'
            % (profile["vendor_id"], profile["product_id"])
        )
```

# rivalcfg/usbhid.py

```python
import hid
```
```python
    # Search the device
    for interface in hid.enumerate(vendor_id, product_id):
        if interface["interface_number"] == endpoint:
            path = interface["path"]
            break

    # Open the found device. This can raise an IOError.
    if path:
        device.open_path(path)
        return device
```

# Documentation (WebSearch snippets, not re-read verbatim)

- "The `--update-udev` option updates udev rules (Linux only, requires to be run as root)."
- Changelog: "As of version 3.0.0, rivalcfg now uses the hidapi lib"; version 4.3.0 "added the `--print-udev` option".
- Install docs snippet: "On macOS, you may not need any requirement".
