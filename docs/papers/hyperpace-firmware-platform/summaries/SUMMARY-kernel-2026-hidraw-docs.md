# SUMMARY: Linux kernel hidraw documentation

## Claim
- hidraw `read()` returns one queued input report, with the report number in byte 0 for numbered reports. It blocks unless the node was opened O_NONBLOCK.
- `write()` takes the report number in byte 0. On USB the report goes out the interrupt OUT endpoint if one exists, otherwise via SET_REPORT on the control endpoint.
- Ioctls expose the report descriptor and send output and feature reports over the control endpoint.

## Method
Official kernel documentation (the rendered page reported 7.3.0-rc3).

## Result
- The report-ID-first-byte convention is the same across hidraw, hidapi and Windows.
- The descriptor is retrievable with HIDIOCGRDESC on an open node. The kernel ABI also exposes it read-only in sysfs (search snippet, not re-read verbatim).

## Evidence tier
1 (official documentation). The USB endpoint sentence is from a search snippet of the same document.

## Performance
Blocking or poll-based reads; one syscall per report.

## Correctness
- Proven: the byte layout and blocking semantics.
- The docs do not say what happens on disconnect. The source does (SUMMARY-torvalds-2026-hidraw-c-source).

## Relevance to hyperpace
- The Linux write for report 8 is `[0x08, 16 bytes]`. No padding is needed, but this is not proof the device accepts short writes (see the hidraw.c summary).
- A descriptor dump via HIDIOCGRDESC, done by the operator with authorization, answers the TLC questions for all three OSes.
