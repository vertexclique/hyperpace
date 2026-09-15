URL: https://github.com/systemd/systemd/commit/305272ab2b62a031a91ed3490cb053bee0594964
Retrieved: 2026-09-15
Source type: Source code commit in systemd (logind).
Retrieval method: `gh api repos/systemd/systemd/commits/305272ab...` (verbatim message and patch excerpts).
Character note: none needed.

# Excerpts

Author date: `2022-04-12T04:48:04Z`
Commit message:
> logind: add support for hidraw devices
>
> Add support for opening /dev/hidraw devices via logind's TakeDevice().
> Same semantics as our support for evdev devices, but it requires the
> HIDIOCREVOKE ioctl in the kernel.

`rules.d/71-seat.rules.in`:
```
+SUBSYSTEM=="hidraw", KERNEL=="hidraw*", TAG+="seat"
```
`src/basic/missing_hidraw.h`:
```c
+/* b31c9d9dc343146b9f4ce67b4eee748c49296e99 (6.12) */
+#ifndef HIDIOCREVOKE
+#define HIDIOCREVOKE _IOW('H', 0x0D, int)
+#endif
```
`src/login/logind-session-device.c`:
```c
+static void sd_hidiocrevoke(int fd) {
+        static bool warned = false;
+
+        assert(fd >= 0);
+
+        if (!warned && ioctl(fd, HIDIOCREVOKE, NULL) < 0) {
+                if (errno == EINVAL) {
+                        log_warning_errno(errno, "Kernel does not support hidraw-revocation, continuing without revoking
```
`units/systemd-logind.service.in`: `+DeviceAllow=char-hidraw rw`

Note: the author date is 2022 while the header references kernel 6.12 (released late 2024); the merge date into systemd was not retrieved.
