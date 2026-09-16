# Device art pack

These images are the manufacturer's own product renders and control faces for the mouse and its
receiver, taken from the vendor's web configurator. The operator authorized their use in this app
on 2026-09-16.

What is here:

- `mouse-top.png` (710x380) top-down render of the mouse.
- `receiver.png` (169x95) render of the USB receiver.
- `key/Keys/<n>.png` and `<n>-click.png` numbered button callouts, idle and pressed.
- `sensor/*.png` dial faces and their pointer sprites. Face diameters: `sensor_mode` 170,
  `lod` 186, `sleep_time` 262. Pointers are radius-length sprites rotated about their bottom edge
  at the dial centre.
- `key/brightness.png` (151x84) and `key/speed.png` (160x89) half-ring faces with their pointers.
- `key/battery.png` (132) battery face, `key/charging.png` charging face.
- `colors/color<n>.png` the 14 preset lighting swatches.
- `macro/`, `setting/`, and the check and uncheck states for the toggles.

Not used, and why:

- `sensor/performance_on.png` and `performance_off.png` are composites with the segment labels and
  the vendor's orange baked into the pixels. Hyperpace draws those segments itself so the live arc
  carries this app's accent. The neutral faces are used instead.
- The vendor's light page backgrounds (`bg.png`, `home_bg.png`, `bg_setting.png`) were dropped:
  this app is dark.

The vendor wordmark is printed on the physical device, so it is visible in `mouse-top.png` and
`receiver.png`. No vendor name, logo or model string appears anywhere in the app's own chrome,
copy, binary or package metadata.
