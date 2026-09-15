URL: https://github.com/apple-oss-distributions/IOKitUser (tag IOKitUser-100231.120.3; files hid.subproj/IOHIDDevice.h, IOHIDManager.h, IOHIDLib.h) ; https://github.com/apple-oss-distributions/IOHIDFamily (main; IOHIDFamily/IOHIDKeys.h)
Retrieved: 2026-09-15
Source type: Apple open-source distribution of the public IOKit HID user-space headers (header doc comments are Apple's API documentation).
Retrieval method: `gh api repos/apple-oss-distributions/IOKitUser/contents/hid.subproj/<file>` and `gh api repos/apple-oss-distributions/IOHIDFamily/contents/IOHIDFamily/IOHIDKeys.h` (base64 decoded, verbatim).
Character note: none needed.

# IOHIDKeys.h (IOHIDFamily)

```c
  @typedef IOHIDOptionsType
  @abstract Options for opening a device via IOHIDLib.
  @constant kIOHIDOptionsTypeNone Default option.
  @constant kIOHIDOptionsTypeSeizeDevice Used to open exclusive
    communication with the device.  This will prevent the system
    and other clients from receiving events from the device.
  @constant kIOHIDOptionsTypeMaskPrivate Mask for reserved internal usage values.
*/
enum {
    kIOHIDOptionsTypeNone     = 0x00,
    kIOHIDOptionsTypeSeizeDevice = 0x01,
    kIOHIDOptionsTypeMaskPrivate = 0xff0000,
};
```

# IOHIDDevice.h (IOKitUser hid.subproj)

```c
/*!
	@function   IOHIDDeviceOpen
	@abstract   Opens a HID device for communication.
    @discussion Before the client can issue commands that change the state of 
                the device, it must have succeeded in opening the device. This 
                establishes a link between the client's task and the actual 
                device.  To establish an exclusive link use the 
                kIOHIDOptionsTypeSeizeDevice option. 
    @param      device Reference to an IOHIDDevice.
    @param      options Option bits to be sent down to the device.
    @result     Returns kIOReturnSuccess if successful.
*/
CF_EXPORT
IOReturn IOHIDDeviceOpen(          
                                IOHIDDeviceRef                  device, 
                                IOOptionBits                    options)
AVAILABLE_MAC_OS_X_VERSION_10_5_AND_LATER;
```

```c
/*! @function   IOHIDDeviceSetDispatchQueue
    @abstract   Sets the dispatch queue to be associated with the IOHIDDevice.
                This is necessary in order to receive asynchronous events from the kernel.
    @discussion An IOHIDDevice should not be associated with both a runloop and
                dispatch queue. A call to IOHIDDeviceSetDispatchQueue should only be made once.
                After a dispatch queue is set, the IOHIDDevice must make a call to activate
                via IOHIDDeviceActivate and cancel via IOHIDDeviceCancel. All calls to "Register"
                functions should be done before activation and not after cancellation.
    @param      device Reference to an IOHIDDevice
    @param      queue The dispatch queue to which the event handler block will be submitted.
*/
CF_EXPORT
void IOHIDDeviceSetDispatchQueue(
                                IOHIDDeviceRef                  device,
                                dispatch_queue_t                queue)
__OSX_AVAILABLE(10.15) __IOS_AVAILABLE(13.0) __TVOS_AVAILABLE(13.0) __WATCHOS_AVAILABLE(6.0);
```

```c
/*! @function   IOHIDDeviceSetReport
    @abstract   Sends a report to the device.
    @discussion This method behaves synchronously and will block until the
                report has been issued to the device. It is only relevent for
                either output or feature type reports.
    @param      device Reference to an IOHIDDevice.
    @param      reportType Type of report being sent.
    @param      reportID ID of the report being sent.  If the device supports
                multiple reports, this should also be set in the first byte of
                the report.
    @param      report The report bytes to be sent to the device.
    @param      reportLength The length of the report to be sent to the device.
    @result     Returns kIOReturnSuccess if successful.
*/
```

Search for `IOHIDRequestAccess` / `IOHIDCheckAccess` in the downloaded `hid.subproj/IOHIDLib.h` and IOHIDFamily headers returned no match; the declaration was not located in these repositories (it may live in another public header not downloaded). Developer-site JSON endpoints tried for IOHIDRequestAccess returned HTTP 404.

Related Apple DTS forum evidence (thread 683940, Quinn "The Eskimo!", Jul 2021, WebFetch extraction): the assertion `"assertion failure: Schedule failed queue: 0x0 runLoop: 0x600003e14300"` comes from `os_assert(!manager->runLoop && !manager->dispatchQueue, "Schedule failed queue: %p runLoop: %p", manager->dispatchQueue, manager->runLoop);` in `IOHIDManagerScheduleWithRunLoop`, i.e. a manager must not be scheduled twice or on both a run loop and a dispatch queue.
