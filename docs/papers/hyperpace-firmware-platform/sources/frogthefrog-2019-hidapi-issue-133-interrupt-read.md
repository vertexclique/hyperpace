URL: https://github.com/libusb/hidapi/issues/133
Retrieved: 2026-09-15
Source type: GitHub issue with maintainer (Youw) and node-hid maintainer (todbot) comments; OPEN at retrieval.
Retrieval method: `gh issue view 133 -R libusb/hidapi --json ...` (verbatim, comments truncated at 800 chars).
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

`Windows: unable to interrupt blocking hid_read call | FrogTheFrog | 2019-12-17T20:50:29Z | OPEN`

> Currently hidapi is not able to close open device handle in a thread-safe manner. This is due to `CancelIo` function being used instead of `CancelIoEx` for closing a win32 device handle.

Youw (maintainer), 2019-12-17:
> I don't think it is safe to assume `hid_close` as a thread-safe function at all.
>
> Any client code has to stop any read/write attempts before calling `hid_close`.

Youw, 2019-12-18 (recommended pattern):
> thread1:
> ```cpp
> while(!interrupted) {
>   result = hid_read_timeout(dev, buffer, buffer_size, 1000);
>   if (result == 0) // no data available
>     continue;
>   if (result > 0) {
>     // handle data here
>   }
>   else {
>     // handle/report error, retry, etc.
>   }
> }
> ```
> thread2:
> ```cpp
> interrupted = true;
> thread1.join();
> // now we're sure no other threads are trying to access the device,
> // assuming there is no threads, except thread1
> hid_close(dev);
> ```

todbot (node-hid), 2019-12-18:
> Suffice to say, `node-hid` was doing a blocking `hid_read()` in a separate thread and thus could lead to a race condition. I've now changed it to use `hid_read_timeout()` like you describe above.

Youw, 2019-12-18:
> I've checked both hidraw and macOS implementations: there is always a _race condition_ when `hid_close` is called during blocking `hid_read` call.
>
> The fact that you don't hit any issues is just you being _lucky_. It doesn't mean it is a well defined behavior.

Current-code note (2026-09-15): hidapi-rs `windows_native` uses `CancelIoEx` (src/windows_native/mod.rs lines 184 and 362); libusb/hidapi master `windows/hid.c` still calls `CancelIo(dev->device_handle)` on a failed ReadFile (line 1191).
