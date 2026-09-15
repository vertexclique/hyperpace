# SUMMARY: Peter Hutterer, "HIOCREVOKE merged for kernel 6.12"

## Claim
- Kernel 6.12 adds HIDIOCREVOKE for hidraw, mirroring EVIOCREVOKE, and logind support has merged.
- logind only hands hidraw fds to the session leader (compositor or Xorg).
- Applications would need a future Wayland protocol or portal to get them.
- Until then the options are running as root or shipping uaccess udev rules.

## Method
Blog post by the author of the kernel patch and the logind change.

## Result
- There is no user-app route to hidraw through logind today.
- The uaccess udev rule remains the only unprivileged mechanism for a normal desktop app.

## Evidence tier
2 (maintainer/author post).

## Performance
Not applicable.

## Correctness
- Authoritative for the design intent as of October 2024.
- xdg-desktop-portal issue 611 (hidraw portal) was still open on 2026-09-15, so no portal route has appeared since.

## Relevance to hyperpace
- Rules out logind TakeDevice as a way around udev rules.
- The Linux permission design must center on a device-specific uaccess rule.
