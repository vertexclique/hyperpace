URL: https://infocenter.nordicsemi.com/topic/sdk_nrf5_v16.0.0/lib_bootloader_dfu_banks.html
Also listed: v15.0.0, v15.2.0, v15.3.0, v12.x, v13.x pages of the same title; v17.1.0 redirects to https://docs.nordicsemi.com/bundle/sdk_nrf5_v17.1.0/page/lib_bootloader_dfu_banks.html
Retrieval date: 2026-09-15
Source type: official vendor documentation (Nordic nRF5 SDK), available here only as search-engine snippets

# nRF5 SDK Secure DFU: dual-bank and single-bank updates

## Acquisition and limits

- The fetch tool got HTTP 302 then HTTP 403 from docs.nordicsemi.com. A curl attempt for the same page was blocked by the session's permission classifier and not retried.
- The text below is the search engine's rendering of the page snippets, not a verbatim page read. Treat wording as approximate; the mechanism matches the SDK source in `dius-2019-nrf5-sdk-15-3-dfu-validation-source.md`.

## Snippet text (search engine, query: nRF5 SDK bootloader "dual-bank" "single-bank" update interrupted existing application preserved)

> During a dual-bank update, the existing application is preserved until the new firmware image is activated. If the firmware update process fails, you can still reboot the device to start the existing application.

> The memory area between the end of the SoftDevice and the beginning of the application data is divided into two banks. Bank 0 holds the existing application, and bank 1 is used to store the received image.

> After all packets of the new application are received, both the old and the new application are present in the memory. This ensures that fallback to the old application is possible if the new application cannot be activated.

> In a single-bank update, the existing application is replaced with the new application during the transfer of the image.

## Snippet text (query: nRF5 SDK Secure DFU bootloader ... USB CDC serial transport documentation)

> The USB Secure DFU Bootloader example uses the CDC ACM USB class, commonly known as Virtual COM port.

> "nrfutil dfu serial -pkg package.zip -p COM_PORT" for UART transport or "nrfutil dfu usb_serial -pkg package.zip -p COM_PORT" for USB transport.
