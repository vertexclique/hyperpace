URL: https://learn.microsoft.com/en-us/windows-hardware/drivers/hid/top-level-collections
Retrieved: 2026-09-15
Source type: Official conceptual documentation (Microsoft Learn, HID drivers). Page metadata: ms.date 2024-06-27, updated_at 2025-07-18.
Retrieval method: WebFetch (page converted to markdown; text below is the returned page text).
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

> A *top-level collection* is a grouping of functionality that targets a particular software consumer (or type of consumer) of the functionality. For example, a top-level collection can be described as keyboard, mouse, consumer control, sensor, display, and so on. In the HID spec, these top-level collections are also referred to as *application collections*. The HID device describes the purpose of each top-level collection, allowing the consumers of HID functionality to identify top-level collections in which they're interested. In Windows, the HID device setup class (HIDClass) generates a unique physical device object (PDO) for each top-level collection described by the report descriptor. Microsoft defines a *top-level collection* as a HID collection that isn't nested within another collection. An unnested collection is always a top-level collection, regardless of its HID type. In particular, a top-level collection doesn't have to be an application collection, as defined by the USB HID Standard.

> A report descriptor can include more than one top-level collection. The HID class driver enumerates the top-level collections of an input device and creates a physical device object (*PDO*) for each top-level collection. User-mode applications or kernel-mode drivers can access a top-level collection by opening its PDO and using the HIDClass support routines and the HID class driver IOCTLs.

> - A **HIDP_CAPS** structure summarizes a top-level collection's capability.
