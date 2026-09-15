URL: https://github.com/signal11/hidapi/issues/218 (original HIDAPI repository)
Retrieved: 2026-09-15
Source type: GitHub issue with a comment from the original HIDAPI author (signal11); CLOSED 2015-04-09.
Retrieval method: `gh issue view 218 -R signal11/hidapi --json ...` (verbatim).
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

`(Windows) Sending packet smaller than caps.OutputReportByteLength | Flafla2 | 2015-04-06T21:05:20Z | CLOSED`

> I am working with a device (the wiimote) that takes commands through the DATA pipe, and only accepts command packets that are EXACTLY as long as the command itself.

> This is a problem on windows, as WriteFile() on windows requires that the byte[] passed to it is at least as long as caps.OutputReportByteLength.  On mac, where this limitation isn't present, my code works correctly.

> Removing the above code, as mentioned in the comments, results in an error from WriteFile().

Flafla2, 2015-04-09:
> Solved.  I used a solution similar to the guys over at Dolphin, a Wii emulator.  Apparently, on the Microsoft bluetooth stack, WriteFile() doesn't work correctly, causing the Wiimote to return with an error.  By using HidD_SetOutputReport() on the MS stack and WriteFile() on the BlueSoleil stack, I was able to successfully connect to the device (at least on my machine).

signal11, 2015-04-14:
> HidD_SetOutputReport() sends a report over the control endpoint. WriteFile() sends it out the OUT endpoint, if one is available. Does your device not support the OUT endpoint even though it has one?
