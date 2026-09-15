URL: https://github.com/ruabmbua/hidapi-rs/issues/127
Retrieved: 2026-09-15
Source type: GitHub issue with maintainer comment (state OPEN at retrieval; last update 2025-10-03).
Retrieval method: `gh issue view 127 -R ruabmbua/hidapi-rs --json title,author,createdAt,state,body,comments` (verbatim text; long bodies truncated at 2500 chars by the query).
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

Title / author / date / state: `Assertion / Illegal instruction on macOS when calling HidApi::new in different threads | Tiwalun | 2023-07-14T15:10:10Z | OPEN`

Body:
> I've run into an issue on macOS where calling `HidApi::new` from different threads leads to an abort of the process, with an exception for an illegal instruction.
>
> Looking at the stacktrace, it seems to be cause by a `CFAssertMismatchedTypeID` assertion somewhere in Core Foundation.

Stack (excerpt):
```
  * frame #0: 0x00007ff80cf83f3e CoreFoundation`_CFAssertMismatchedTypeID + 110
    frame #1: 0x00007ff80ce254bd CoreFoundation`CFRunLoopAddSource + 973
    frame #2: 0x00007ff80f76e305 IOKit`IOHIDDeviceScheduleWithRunLoop + 74
    frame #3: 0x00007ff80f772651 IOKit`__IOHIDManagerDeviceApplier + 527
    frame #4: 0x00007ff80f735537 IOKit`__IOHIDManagerDeviceAdded + 766
    frame #5: 0x00007ff80f73513b IOKit`__IOHIDManagerSetDeviceMatching + 347
    frame #6: 0x0000000100010724 hidapi-repro`hid_enumerate(vendor_id=0, product_id=0) at hid.c:654:2
```

Maintainer ruabmbua, 2023-07-14:
> Because of the not very well defined nature of the underlying C library (regarding multithreading), we used to prevent creating a second instance of a Hidapi object, by acquiring a lock.
> However people started requesting that I lift this restriction, and testing on the backends that we could test showed, that as long as you open a device not from multiple threads, everything works out fine.
>
> Unfortunately we may have not tested on macos (due to nobody owning a mac).

Tiwalun, 2023-07-19:
> The workaround I have for now is to spawn a dedicated thread for `hidapi`, run `HidApi::new()` in there, and keep it alive for the whole duration of the process. With that, it seems possible to use `hidapi` from other threads without issues afterwards :man_shrugging:.
>
> My current assumption is that this line https://github.com/libusb/hidapi/blob/d0856c05cecbb1522c24fd2f1ed1e144b001f349/mac/hid.c#L443 associates the `IOHidManager` with the run loop of the thread calling `hid_init`, and because of this the thread has to stay alive as long as `hidapi` is used, otherwise the run loop doesn't exist anymore, and then an assertion is triggered.

micolous, 2023-09-12:
> That's correct, `IOHIDManager` needs to be alive and associated with an active `CFRunLoop` while any `IOHIDDevice` instance it created is still open or in use.
>
> When `IOHIDManager` is unscheduled, it also unschedules all its `IOHIDDevice`s, when closed it closes all devices, and when released, it releases all devices. Unlike most Core Foundation objects, there's no reference counting - `IOHIDManager` just tears everything apart once it's done, and there's no way to control that.

White-Rabbit-Scientific, 2024-11-17:
> Just wanted to note that I have reached the same endpoint with hidapi on MacOS Sequia 15.1.
>
> I cannot run this line inside egui without it crashing:
> `let api = hidapi::HidApi::new().unwrap();`

davehorner, 2025-07-31:
> I released a crate using hidapi-rs.  I am seeing the same crashes when using macos.
> There doesnt seem to be a solution, I see above someone wrapped in a thread.
