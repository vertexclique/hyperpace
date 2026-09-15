URL: https://learn.microsoft.com/en-us/windows-hardware/drivers/install/registering-for-notification-of-device-interface-arrival-and-device-removal
Retrieved: 2026-09-15
Source type: Official how-to documentation (Microsoft Learn, device installation). Page metadata: ms.date 2024-10-24, updated_at 2025-03-25.
Retrieval method: WebFetch (page converted to markdown; text below is the returned page text).
Character note: any em-dash or en-dash in quoted text was replaced with an ASCII hyphen; nothing else changed.

# Excerpts

> Typically, a user-mode component calls **CM_Register_Notification** to find a device interface, and then sends I/O requests to the interface. To do so, the component registers for both **CM_NOTIFY_FILTER_TYPE_DEVICEINTERFACE** and **CM_NOTIFY_FILTER_TYPE_DEVICEHANDLE**, for notification of device interface arrivals and device removals respectively.

> 1. Call **CM_Register_Notification** with **CM_NOTIFY_FILTER_TYPE_DEVICEINTERFACE** to register for device interface arrival notifications. When future interfaces in the specified class arrive, the system notifies your component.
> 2. Because the interface you want to send I/O to might already be present on the system, call **CM_Get_Device_Interface_List** or **SetupDiGetClassDevs** to retrieve a list of existing interfaces. **Note** If an interface arrives between step 1 and step 2, the interface is listed twice, from the registration in step 1 and the list of interfaces in step 2.
> 3. Once you find your desired interface, call **CreateFile** to open a handle for the device.
> 4. After successfully creating a device handle in step 3, call **CM_Register_Notification** a second time. This time, register for notifications of type **CM_NOTIFY_FILTER_TYPE_DEVICEHANDLE**, and provide the new device handle as the handle for which to receive notifications. When the device represented by the interface receives a query remove request, the system notifies your component.

Table rows:
> **CM_NOTIFY_ACTION_DEVICEQUERYREMOVE**: Call **CloseHandle** to close the device handle. If you do not do this, your open handle prevents the query remove of this device from succeeding.

> **CM_NOTIFY_ACTION_DEVICEQUERYREMOVEFAILED**: The query remove failed, so the device and its interface are still valid. To continue sending I/O to the interface, open a new handle to it.

> **CM_NOTIFY_ACTION_DEVICEREMOVEPENDING**: Call **CM_Unregister_Notification** to unregister the notifications for your handle. You must do this from a deferred routine. See the **Remarks** section of **CM_Unregister_Notification** for more information. If you still have an open handle to the device, call **CloseHandle** to close the device handle.

> **CM_NOTIFY_ACTION_DEVICEREMOVECOMPLETE**: Call **CM_Unregister_Notification** to unregister the notifications for your handle. You must do this from a deferred routine. See the **Remarks** section of **CM_Unregister_Notification** for more information. If you still have an open handle to the device, call **CloseHandle** to close the device handle.
