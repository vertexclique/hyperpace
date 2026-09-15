URL: https://github.com/libusb/hidapi (master HEAD 852cc68, 2026-08-14)
Retrieved: 2026-09-15
Source type: Source code of the C HIDAPI library (git clone), GitHub releases API, repository AGENTS.md.
Retrieval method: `git clone https://github.com/libusb/hidapi`; `gh api repos/libusb/hidapi/releases`; `gh api repos/libusb/hidapi`.
Character note: em-dash characters in AGENTS.md quotes were replaced with ASCII hyphens; nothing else changed.

# Version and activity

- `VERSION` file on master: `0.16.0` (development version on master).
- Releases: `hidapi-0.15.0 2025-05-18T18:58:29Z`, `hidapi-0.14.0 2023-05-22T21:06:14Z`, `hidapi-0.13.1 2023-01-09T11:44:55Z`.
- GitHub API: `open_issues_count=76 pushed_at=2026-09-09T10:50:39Z stars=2274`.
- `hidapi/hidapi.h` on master contains no `hotplug` symbol (grep returned nothing). Branches present: `connection-callback`, `connection-callback-2` (last commit `c7476b2 2026-09-09 ci: rerun virtual-device validation`), `hotplug-integration` (`592bb78 2026-07-15`), whose `hidapi/hidapi.h` declares `hid_hotplug_register_callback()`.

# windows/hid.c

Line 234: `dev->write_timeout_ms = 1000;`
Line 1029: `if (!HidD_SetNumInputBuffers(device_handle, 64)) {`
Lines 1055-1057:
```c
dev->output_report_length = caps.OutputReportByteLength;
dev->input_report_length = caps.InputReportByteLength;
dev->feature_report_length = caps.FeatureReportByteLength;
```
`hid_write` (lines 1096-1121):
```c
/* Make sure the right number of bytes are passed to WriteFile. Windows
   expects the number of bytes which are in the _longest_ report (plus
   one for the report number) bytes even if the data is a report
   which is shorter than that. Windows gives us this value in
   caps.OutputReportByteLength. If a user passes in fewer bytes than this,
   use cached temporary buffer which is the proper size. */
if (length >= dev->output_report_length) {
    /* The user passed the right number of bytes. Use the buffer as-is. */
    buf = (unsigned char *) data;
} else {
    ...
    buf = dev->write_buf;
    memcpy(buf, data, length);
    memset(buf + length, 0, dev->output_report_length - length);
    length = dev->output_report_length;
}

res = WriteFile(dev->device_handle, buf, (DWORD) length, &bytes_written, &dev->write_ol);
```
Lines 1135-1149:
```c
if (overlapped) {
    /* Wait for the transaction to complete. This makes
       hid_write() synchronous. */
    res = WaitForSingleObject(dev->write_ol.hEvent, dev->write_timeout_ms);
    if (res != WAIT_OBJECT_0) {
        /* There was a Timeout. */
        register_winapi_error(dev, L"hid_write/WaitForSingleObject");
        goto end_of_function;
    }
    /* Get the result. */
    res = GetOverlappedResult(dev->device_handle, &dev->write_ol, &bytes_written, FALSE/*wait*/);
    if (res) {
        function_result = bytes_written;
    }
```
`hid_read_timeout` (lines 1202-1233):
```c
res = WaitForSingleObject(ev, milliseconds >= 0 ? (DWORD)milliseconds : INFINITE);
if (res != WAIT_OBJECT_0) {
    /* There was no data this time. Return zero bytes available,
       but leave the Overlapped I/O running. */
    return 0;
}
...
if (dev->read_buf[0] == 0x0) {
    /* If report numbers aren't being used, but Windows sticks a report
       number (0x0) on the beginning of the report anyway. To make this
       work like the other platforms, and to make it work more like the
       HID spec, we'll skip over this byte. */
```
`hid_send_feature_report` comment (lines 1279-1283):
```c
/* Windows expects at least caps.FeatureReportByteLength bytes passed
   to HidD_SetFeature(), even if the report is shorter. Any less sent and
   the function fails with error ERROR_INVALID_PARAMETER set. Any more
   and HidD_SetFeature() silently truncates the data sent in the report
   to caps.FeatureReportByteLength. */
```
`hid_send_output_report` comment (lines 1380-1384), and it pads with `write_buf` sized `output_report_length`:
```c
/* Windows expects at least caps.OutputeportByteLength bytes passed
   to HidD_SetOutputReport(), even if the report is shorter. Any less sent and
   the function fails with error ERROR_INVALID_PARAMETER set. Any more 
   and HidD_SetOutputReport() silently truncates the data sent in the report
   to caps.OutputReportByteLength. */
```
Lines 360-361 (open): `DWORD desired_access = (open_rw)? (GENERIC_WRITE | GENERIC_READ): 0;` / `DWORD share_mode = FILE_SHARE_READ|FILE_SHARE_WRITE;`

# mac/hid.c

Lines 446-449 (`init_hid_manager`):
```c
hid_mgr = IOHIDManagerCreate(kCFAllocatorDefault, kIOHIDOptionsTypeNone);
if (hid_mgr) {
    IOHIDManagerSetDeviceMatching(hid_mgr, NULL);
    IOHIDManagerScheduleWithRunLoop(hid_mgr, CFRunLoopGetCurrent(), kCFRunLoopDefaultMode);
```
Line 476 (`hid_init`): `hid_darwin_set_open_exclusive(1); /* Backward compatibility */`
Lines 1538-1540:
```c
void HID_API_EXPORT_CALL hid_darwin_set_open_exclusive(int open_exclusive)
{
	device_open_options = (open_exclusive == 0) ? kIOHIDOptionsTypeNone : kIOHIDOptionsTypeSeizeDevice;
```
`hid_enumerate` (lines ~720-726): `/* give the IOHIDManager a chance to update itself */` / `process_pending_events();` where `process_pending_events` runs `CFRunLoopRunInMode(kCFRunLoopDefaultMode, 0.001, FALSE)` on the calling thread.
`hid_open_path` (lines 1062-1088):
```c
ret = IOHIDDeviceOpen(dev->device_handle, dev->open_options);
if (ret != kIOReturnSuccess) {
    register_global_error_format("hid_open_path: failed to open IOHIDDevice from mach entry: (0x%08X) %s", ret, mach_error_string(ret));
...
IOHIDDeviceRegisterInputReportCallback(
    dev->device_handle, dev->input_report_buf, dev->max_input_report_len,
    &hid_report_callback, dev);
IOHIDDeviceRegisterRemovalCallback(dev->device_handle, hid_device_removal_callback, dev);

/* Start the read thread */
pthread_create(&dev->thread, NULL, read_thread, dev);
```
`set_report` (lines 1118-1136):
```c
report_id = data[0];

if (report_id == 0x0) {
    /* Not using numbered Reports.
       Don't send the report number. */
    data_to_send = data+1;
    length_to_send = length-1;
}

/* Avoid crash if the device has been unplugged. */
if (dev->disconnected) {
    register_device_error(dev, "Device is disconnected");
    return -1;
}

res = IOHIDDeviceSetReport(dev->device_handle,
                           type,
                           report_id,
                           data_to_send, length_to_send);
```
Enumeration uses `kIOHIDDeviceUsagePairsKey` (line 356) to emit per-usage entries.

# linux/hid.c

Lines 814-847:
```c
/*
 * Parse the first usage and usage page
 * out of the report descriptor.
 */
if (!get_next_hid_usage(report_desc.value, report_desc.size, &usage_iterator, &page, &usage)) {
    cur_dev->usage_page = page;
    cur_dev->usage = usage;
}

/*
 * Parse any additional usage and usage pages
 * out of the report descriptor.
 */
while (!get_next_hid_usage(report_desc.value, report_desc.size, &usage_iterator, &page, &usage)) {
    /* Create new record for additional usage pairs */
    ...
    cur_dev->path = dev_path? strdup(dev_path): NULL;
```

# udev/69-hid.rules

```
# HIDAPI/hidraw
KERNEL=="hidraw*", ATTRS{idVendor}=="04d8", ATTRS{idProduct}=="003f", TAG+="uaccess"
```
```
# NOTE: these rules must have priority before 73-seat-late.rules.
```
```
# TAG+="uaccess" only gives permission to physically present users, which
# is appropriate in most scenarios. If you require access to the device
# from a remote session (e.g. over SSH), add
# GROUP="plugdev", MODE="660"
```

# AGENTS.md "HIDAPI API usage contract" (added in 852cc68, 2026-08-14)

> - HIDAPI v0.x.x is **not thread-safe**. The following must not be called concurrently from different threads: `hid_init`, `hid_exit`, `hid_enumerate`, `hid_open`, `hid_open_path`, `hid_close`, `hid_error(NULL)` - the global error string is one reason.
> - Functions taking a `hid_device *` are not individually thread-safe, but different devices may be used from different threads - i.e. a dedicated thread per device.
> - `hid_close()` must be serialized against initialization and enumeration calls.
> - Since v0.15.0, a dedicated read thread using only `hid_read`, `hid_read_timeout` and `hid_read_error` is safe alongside other operations on the same device from another thread. Do not use `hid_error` from such a thread; use `hid_read_error`.
> - macOS: `hid_init()` and `hid_exit()` must be called from the same thread, and that thread must stay alive until all devices are closed and `hid_exit()` has completed.
