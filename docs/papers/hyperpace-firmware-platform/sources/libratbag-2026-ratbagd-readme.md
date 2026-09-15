URL: https://github.com/libratbag/libratbag (README; repository pushed 2026-08-18)
Retrieved: 2026-09-15
Source type: Official project README.
Retrieval method: `gh api repos/libratbag/libratbag/readme` (verbatim excerpts via grep).
Character note: none needed.

# Excerpts

> libratbag provides **ratbagd**, a DBus daemon to configure input devices,
> mainly gaming mice. The daemon provides a generic way to access the various
> features exposed by these mice and abstracts away hardware-specific and
> kernel-specific quirks.

> To run ratbagd, simply run it as root `sudo ratbagd`. However,
> ratbagd is intended to run as dbus-activated systemd service and installs
> the following files:
>
>     /usr/share/dbus-1/system.d/org.freedesktop.ratbag1.conf
>     /usr/share/dbus-1/system-services/org.freedesktop.ratbag1.conf
>     /usr/share/systemd/system/ratbagd.service

> libratbag has two main components, libratbag and ratbagd. Applications like
> Piper talk over DBus to ratbagd. ratbagd uses libratbag to access the actual
> devices.
