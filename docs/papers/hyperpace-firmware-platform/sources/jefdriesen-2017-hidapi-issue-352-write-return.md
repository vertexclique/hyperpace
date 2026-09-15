URL: https://github.com/signal11/hidapi/issues/352 (original HIDAPI repository, predecessor of libusb/hidapi)
Retrieved: 2026-09-15
Source type: GitHub issue with a comment from the original HIDAPI author (signal11, Alan Ott); OPEN at retrieval (repository unmaintained).
Retrieval method: `gh issue view 352 -R signal11/hidapi --json ...` (verbatim).
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

`The hid_write return value is too large on windows | jefdriesen | 2017-08-18T12:12:11Z | OPEN`

> The Windows HID api always expects to receive a fixed size buffer (corresponding to the largest report supported by the device). Therefore the hidapi library internally pads the buffer with zeros to the expected size, but apparently it also returns the size of the padded buffer! Thus, for small writes it returns a value that is larger than the buffer that was passed. And that leads to very unexpected results when you actually use the return value.
>
> For example, if I write 2 bytes to a device with an output report size of 32 bytes, then hid_write will return 32 instead of 2!

signal11, 2018-02-01:
> It's how Windows works, unfortunately. You have to give it the length of the longest report, and it gives you the length of the longest report. It's only an issue for devices with multiple (numbered) reports, so it doesn't come up very often.

jefdriesen, 2018-02-01:
> For example it's common practice to compare the return value with the passed length to check whether the entire write operation was successful (and not just a partial success):
> ```
> nbytes = hid_write(device, data, size);
> if (nbytes < 0 || nbytes != size) {
>     /* Error or partial write */
> }
> ```
> Suddenly an error is reported on Windows (but not on Linux or Mac), even if the operation was actually successful!

Current code check (libusb/hidapi master, windows/hid.c): `length = dev->output_report_length;` then `function_result = bytes_written;`, so the padded length is still returned in 2026.
