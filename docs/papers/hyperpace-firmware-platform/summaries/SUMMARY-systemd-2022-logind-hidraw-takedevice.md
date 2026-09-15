# SUMMARY: systemd commit, logind TakeDevice for hidraw

## Claim
logind can hand out and revoke /dev/hidraw file descriptors through TakeDevice(). It uses HIDIOCREVOKE (kernel 6.12) and tags hidraw devices with the "seat" tag.

## Method
Read the commit message and patch.

## Result
- Adds a hidraw device type to logind.
- `sd_hidiocrevoke` warns if the kernel lacks revocation.
- `71-seat.rules` tags hidraw nodes `seat`.
- logind's device allow list gains char-hidraw.

## Evidence tier
1 (source code).

## Performance
Not applicable.

## Correctness
- Proven: the mechanism exists.
- The commit's author date (2022-04-12) predates the kernel 6.12 ioctl it references, so the merge date is later but was not retrieved.
- Who may call TakeDevice is not in this patch. The patch author's blog says only the session leader receives these fds (SUMMARY-hutterer-2024-hidiocrevoke-logind-blog).

## Relevance to hyperpace
- Not usable by hyperpace directly: TakeDevice is for session controllers (compositor or Xorg).
- It explains why systemd refuses blanket hidraw uaccess.
- It is not a packaging route for an AppImage.
