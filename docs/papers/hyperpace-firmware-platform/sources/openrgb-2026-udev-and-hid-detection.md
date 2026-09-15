URL: https://openrgb.org/udev.html ; source https://gitlab.com/CalcProgrammer1/OpenRGB (HEAD 08a9f83, 2026-09-15)
Retrieved: 2026-09-15
Source type: Official project documentation page (WebFetch) and source code (shallow git clone).
Retrieval method: WebFetch of openrgb.org/udev.html; `git clone --depth 1 https://gitlab.com/CalcProgrammer1/OpenRGB.git`.
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# openrgb.org/udev.html

> "If you have installed OpenRGB using a distribution package (.deb, .rpm, from your distribution's package manager, or from AUR), the udev rules file should have been automatically installed."

> "If using the AppImage, run the following commands to install the udev rules. Replace the OpenRGB.AppImage name to whatever AppImage version you're using."
```
sudo mkdir -p /etc/udev/rules.d
sudo ./OpenRGB.AppImage --generate-udev-rules /etc/udev/rules.d/60-openrgb.rules
sudo udevadm control --reload-rules
sudo udevadm trigger
```
> "If using OpenRGB from Flatpak, run the following commands to install the udev rules."
```
sudo mkdir -p /etc/udev/rules.d
sudo sh -c 'flatpak run org.openrgb.OpenRGB --print-udev-rules > /etc/udev/rules.d/60-openrgb.rules'
sudo udevadm control --reload-rules
sudo udevadm trigger
```

# DetectionManager.cpp

HID detector filter:
```cpp
bool BasicHIDBlock::compare(hid_device_info* info)
{
    return(((vid        == HID_VID_ANY)
         || (vid == info->vendor_id))
        && ((pid        == HID_PID_ANY)
         || (pid == info->product_id))
        && ((usage_page == HID_USAGE_PAGE_ANY)
         || (usage_page == info->usage_page))
        && ((usage      == HID_USAGE_ANY)
         || (usage      == info->usage))
        && ((interface  == HID_INTERFACE_ANY)
         || (interface  == info->interface_number))
        && ((bus        == HID_BUS_ANY)
         || (bus        == (int)info->bus_type))
        );
}
```
Hotplug wrapper (compiled when the linked hidapi provides it):
```cpp
#if(HID_HOTPLUG_ENABLED)
    (hidapi_wrapper_hotplug_register_callback)      hid_hotplug_register_callback,
    (hidapi_wrapper_hotplug_deregister_callback)    hid_hotplug_deregister_callback,
#endif
```
udev rule generator (line 2389):
```cpp
fprintf(output_file, "TAG+=\"uaccess\", TAG+=\"%s\"\n", device_name_tag.c_str());
```
Example registration from search result (Controllers/AlienwareKeyboardController): `REGISTER_HID_DETECTOR_IPU("Alienware AW510K", DetectAlienwareAW510KControllers, ALIENWARE_VID, ALIENWARE_AW510K_PID, 0x02, 0xFF00, 0x01)` (interface, usage page, usage).
