URL: https://github.com/apple-oss-distributions/IOHIDFamily (main, HEAD 777ccd9, 2026-04-13; latest tag listed IOHIDFamily-2238.120.5) - files IOHIDFamily/IOHIDDevice.cpp and IOHIDLib/IOHIDDeviceClass.m
Retrieved: 2026-09-15
Source type: Apple open-source release of the IOHIDFamily kernel extension and IOHIDLib user-space plug-in (source code).
Retrieval method: `git clone --depth 1 https://github.com/apple-oss-distributions/IOHIDFamily`; grep and sed on the files (verbatim).
Character note: none needed.

# IOHIDFamily/IOHIDDevice.cpp, in IOHIDDevice start (around lines 527-531)

```cpp
    _interfaceNubs = createInterfaces(_interfaceElementArrays);

    // set interface properties
    publishProperties(NULL);
    
    if (conformsTo(kHIDPage_GenericDesktop, kHIDUsage_GD_Keyboard) ||
        conformsTo(kHIDPage_GenericDesktop, kHIDUsage_GD_Mouse) ||
        conformsTo(kHIDPage_Digitizer, kHIDUsage_Dig_TouchPad)) {
        setProperty(kIOHIDRequiresTCCAuthorizationKey, kOSBooleanTrue);
    }
```

# IOHIDFamily/IOHIDDevice.cpp, conformsTo (line 2567)

```cpp
bool IOHIDDevice::conformsTo(UInt32 usagePage, UInt32 usage)
{
    bool result = false;
    OSArray *usagePairs = NULL;
    
    usagePairs = newDeviceUsagePairs();
    __Require(usagePairs && usagePairs->getCount(), exit);
    
    for (unsigned int i = 0; i < usagePairs->getCount(); i++) {
        ...
        up = OSDynamicCast(OSNumber, pairs->getObject(kIOHIDDeviceUsagePageKey));
        ...
        u = OSDynamicCast(OSNumber, pairs->getObject(kIOHIDDeviceUsageKey));
        ...
        if (usagePage == usagePageNum && usage == usageNum) {
            result = true;
            break;
        }
    }
```

# IOHIDLib/IOHIDDeviceClass.m, initConnect (around lines 505-535)

```objc
    if (!_tccRequested) {
        NSNumber *tcc = CFBridgingRelease(IORegistryEntryCreateCFProperty(
                                    _service,
                                    CFSTR(kIOHIDRequiresTCCAuthorizationKey),
                                    kCFAllocatorDefault,
                                    0));
        
        if (tcc && [tcc isEqual:@YES]) {
            _tccGranted = IOHIDRequestAccess(kIOHIDRequestTypeListenEvent);
            [self logProtectedServiceEvent];
        } else {
            _tccGranted = true;
        }
        
        _tccRequested = true;

    }
    
    if (!_tccGranted) {
        HIDLogError("0x%llx: TCC deny IOHIDDeviceOpen", regID);
    }
    __Require_Action(_tccGranted, exit, {
        ret = kIOReturnNotPermitted;
        os_unfair_recursive_lock_unlock(&_deviceLock);
    });
    os_unfair_recursive_lock_unlock(&_deviceLock);
    
    ret = IOServiceOpen(_service,
                        mach_task_self(),
                        kIOHIDLibUserClientConnectManager,
                        &connection);
```

# A separate "protected access" list (IOHIDDevice.cpp, newIsAccessProtected, line ~1703)

```cpp
    static const UsagePair ProtectedAccessUsagePairs[] = {
        {kHIDPage_AppleVendor, 0x004B},
        {kHIDPage_AppleVendor, 0x004D}
    };
    ...
    static const AppleVendorID AppleVendorIDs[] = {
        {kIOHIDTransportUSBValue, 1452},
        {kIOHIDTransportBluetoothValue, 76},
        {kIOHIDTransportBluetoothLowEnergyValue, 76}
    };
```
(This second list only applies to Apple vendor IDs 1452/76 and is not relevant to VID 0x3554.)

Entitlement plists in the same repository list `kTCCServiceListenEvent` (tools/IOHIDReportTest-Entitlements.plist line 9; hidd/hidd-Entitlements.plist line 41).

Caveat: the open-source drop may lag the shipping macOS 26.x build; that the same logic ships in the current OS release is assumed, not verified.
