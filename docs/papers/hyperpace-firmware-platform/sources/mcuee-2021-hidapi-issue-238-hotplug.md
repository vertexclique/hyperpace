URL: https://github.com/libusb/hidapi/issues/238 ; related PR list via `gh pr list -R libusb/hidapi --search hotplug`
Retrieved: 2026-09-15
Source type: GitHub issue (maintainer and contributor comments) and pull request metadata.
Retrieval method: `gh issue view 238 -R libusb/hidapi --json ...` (last 6 comments, verbatim, truncated at 900 chars each) and `gh pr list` (verbatim metadata).
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Issue 238

`Add hotplug support | mcuee | 2021-01-26T02:18:02Z | OPEN`
> One of the main missing feature of hidapi is the hotplug support.

k1-801, 2024-04-06:
> A small concern here: calling any Register or Unregister functions from within a callback may (and probably will) cause a deadlock.

k1-801, 2025-12-19:
> After some additional testing, I found that on Linux (hidraw) in some scenarios, the hotplug event returns empty Usage/Page values. The manufacturer string and serial number also seem to not be filled in.

Youw (maintainer), 2025-12-19:
> That is a good hunch. I guess the presense of UDEV event doesn't guarantee that HIDRAW subsystem done initialising.

k1-801, 2025-12-22:
> @Youw found it was my mistake (further - in the closing comment of #765). Everything works correctly if the library is linked at compilation time. If it is loaded through `dlopen`, hidapi cannot retrieve the report descriptor.

# Hotplug PRs (state at 2026-09-15)

```
826	OPEN	2026-07-14	2026-09-08	tests: hotplug API and virtual-device hotplug scenarios
835	OPEN	2026-09-09	2026-09-10	hotplug docs: align the header contract and the NetBSD stub with the backends
836	OPEN	2026-09-09	2026-09-09	RFC: Hotplug implementation (combined improvements for testing)
830	OPEN	2026-07-15	2026-09-08	[CI/validation] Run hotplug tests against all four backend implementations (do not merge)
824	OPEN	2026-07-13	2026-09-08	mac: deliver hotplug ENUMERATE pass asynchronously on the event context
825	OPEN	2026-07-13	2026-09-09	libusb: deliver hotplug ENUMERATE pass asynchronously on the event context
674	OPEN	2024-04-06	2026-07-15	RFC: Hotplug implementation
822	OPEN	2026-07-13	2026-09-08	linux: deliver hotplug ENUMERATE pass asynchronously on the event context
823	OPEN	2026-07-13	2026-09-08	windows: deliver hotplug ENUMERATE pass asynchronously on the event context
790	MERGED	2026-04-24	2026-07-13	Hotplug docs: clarify thread-safety, safe-to-call list, device->next contract
783	MERGED	2026-03-30	2026-04-24	libusb: Fix high CPU usage in callback thread
```
Related open issues: `794 Hotplug (macOS): self-join deadlock in hid_internal_hotplug_cleanup` (2026-04-24), `795 Hotplug: hid_exit() from inside callback should fail fast, not deadlock` (2026-04-24), `828 Linux hidraw hotplug: optional monitor auto-recovery after a udev-monitor failure` (2026-07-15).

Downstream use: OpenRGB `DetectionManager.cpp` (GitLab CalcProgrammer1/OpenRGB, HEAD 08a9f83, 2026-09-15):
```c
#if(HID_HOTPLUG_ENABLED)
    (hidapi_wrapper_hotplug_register_callback)      hid_hotplug_register_callback,
    (hidapi_wrapper_hotplug_deregister_callback)    hid_hotplug_deregister_callback,
#endif
```
