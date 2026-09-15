URL: https://learn.microsoft.com/en-us/windows/win32/api/cfgmgr32/nf-cfgmgr32-cm_register_notification
Retrieved: 2026-09-15
Source type: Official API reference (Microsoft Learn, Win32 API). Page metadata: ms.date 2018-12-05, updated_at 2025-07-01.
Retrieval method: WebFetch (page converted to markdown; text below is the returned page text).
Character note: the typographic apostrophe in "routine's" was kept; any em-dash or en-dash was replaced with an ASCII hyphen.

# Excerpts

> The **CM_Register_Notification** function registers an application callback routine to be called when a PnP event of the specified type occurs.
>
> Use RegisterDeviceNotification instead of **CM_Register_Notification** if your code targets Windows 7 or earlier versions of Windows. Kernel mode callers should use IoRegisterPlugPlayNotification instead.

Filter types named on the page: `CM_NOTIFY_FILTER_TYPE_DEVICEINTERFACE`, `CM_NOTIFY_FILTER_TYPE_DEVICEHANDLE`, `CM_NOTIFY_FILTER_TYPE_DEVICEINSTANCE`.

Remarks:
> Be sure to handle Plug and Play device events as quickly as possible. If your event handler performs any operation that may block execution (such as I/O), it is best to start another thread to perform the operation asynchronously.

> The **CM_Register_Notification** function does not provide notification of existing device interfaces. To retrieve existing interfaces, first call **CM_Register_Notification**, and then call CM_Get_Device_Interface_List. If the interface is enabled after your driver calls **CM_Register_Notification**, but before your driver calls **CM_Get_Device_Interface_List**, the driver receives a notification for the interface arrival, and the interface also appears in the list of device interface instances returned by **CM_Get_Device_Interface_List**.

> HCMNOTIFICATION handles returned by **CM_Register_Notification** must be closed by calling the CM_Unregister_Notification function when they are no longer needed.

> If responding to a **CM_NOTIFY_ACTION_DEVICEQUERYREMOVE** notification, the PCM_NOTIFY_CALLBACK callback should return either ERROR_SUCCESS or ERROR_CANCELLED, as appropriate. Otherwise, the callback should return ERROR_SUCCESS.

Requirements:
> Minimum supported client: Available in Microsoft Windows 8 and later versions of Windows.
