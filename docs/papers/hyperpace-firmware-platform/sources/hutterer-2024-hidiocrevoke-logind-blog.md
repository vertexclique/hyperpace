URL: http://who-t.blogspot.com/2024/10/hiocrevoke-merged-for-kernel-612.html
Retrieved: 2026-09-15
Source type: Maintainer blog post (Peter Hutterer, author of the HIDIOCREVOKE kernel patch and the logind hidraw support), dated Friday, October 4, 2024.
Retrieval method: WebFetch (extraction model; sentences in quotation marks were returned as quotes).
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

Title: "HIOCREVOKE merged for kernel 6.12"

> "if you know what EVIOCREVOKE does, the same now works for hidraw devices via HIDIOCREVOKE."

Tool summary: after revocation the file descriptor becomes invalid and further operations fail with ENODEV.

> "The corresponding logind support has also been merged."

> "Logind will only hand out those fds to the session leader (read: compositor or Xorg)"

> "we may have something like the inputfd Wayland protocol (or maybe a portal but right now it seems a Wayland protocol is more likely)."

On the current state for applications (as returned): hidraw access workarounds involve either "run as root or shipping udev rules tagging the device with uaccess."

Related: xdg-desktop-portal issue 611 comment by whot (2024-09-04): "the `HIDIOCREVOKE` ioctl is now scheduled for kernel 6.12"; the logind PR referenced there is systemd/systemd#33970 (superseding #23140).
