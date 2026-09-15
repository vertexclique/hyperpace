URL: https://github.com/nrfconnect/sdk-nrf/blob/main/applications/nrf_desktop/doc/config_channel.rst
Also: https://github.com/nrfconnect/sdk-nrf/blob/main/scripts/hid_configurator/NrfHidDevice.py and https://github.com/nrfconnect/sdk-nrf/blob/main/applications/nrf_desktop/doc/hid_forward.rst
Retrieval date: 2026-09-15
Source type: official vendor documentation and host source code (Nordic Semiconductor, nRF Connect SDK)

# Nordic nRF Desktop: configuration channel (the HID transport that carries DFU)

## Acquisition

- Partial, blob-less git sparse clone of `https://github.com/nrfconnect/sdk-nrf.git`, commit `2700156c8b09008e9363261c1cd45d45067f3ef7`, into `research-bin/chips/sdk-nrf` (scratch). Only the files quoted here were materialized.
- The rendered docs site (nrfconnectdocs.nordicsemi.com, docs.nordicsemi.com) returned HTTP 403 to the fetch tool, so the rst sources were read instead.

## Excerpts (verbatim)

From `applications/nrf_desktop/doc/config_channel.rst`:

> The configuration channel allows a dongle type device to act as a proxy for Bluetooth® LE Peripheral devices.

> Among the types of data that you can send through the configuration channel are the following:
>
> * Device configuration parameters, for example mouse sensor CPI.
> * Firmware updates.
> * LED effect display data, after it has been generated on the computer.

> The configuration channel activity is performed using a dedicated HID feature report.

> * A request from the host to the device is a single HID feature report set operation.
> * The response is a single HID feature report get operation.

> There can be only one pending configuration channel request.
> The host can send the following request only after it has received a response for the previous request.

Data format table (reproduced as plain text): byte 0 `Report ID`, byte 1 `Recipient`, byte 2 `Event ID` (bits 0..3 `Option ID`, bits 4..7 `Module ID`), byte 3 `Status`, byte 4 `Data length`, bytes 5.. `Data`.

> * ``0`` - The transaction is intended for a directly connected device.
> * Other values - The transaction should be forwarded by the :ref:`nrf_desktop_hid_forward` to the peripheral connected over Bluetooth® LE.

> The values of Module ID and Option ID are assigned during the application build time.

> Bluetooth® LE HID Service removes the leading report ID byte.

> * ``CONFIG_STATUS_PENDING`` - The operation has not completed.
> The response is not ready and the data field should not be interpreted.

> * ``CONFIG_STATUS_DISCONNECTED`` - The operation failed because a module or a device addressed by the request did not respond.
> This can happen when Bluetooth® LE Peripheral disconnects.

From `scripts/hid_configurator/NrfHidDevice.py`:

```
REPORT_ID = 6
REPORT_SIZE = 30
EVENT_DATA_LEN_MAX = REPORT_SIZE - 5
...
POLL_INTERVAL_DEFAULT = 0.02
POLL_RETRY_COUNT = 200
...
class ConfigStatus(IntEnum):
    PENDING            = 0
    GET_MAX_MOD_ID     = 1
    GET_HWID           = 2
    GET_BOARD_NAME     = 3
    INDEX_PEERS        = 4
    GET_PEER           = 5
    SET                = 6
    FETCH              = 7
    SUCCESS            = 8
    TIMEOUT            = 9
    REJECT             = 10
    WRITE_FAIL         = 11
    DISCONNECTED       = 12
    GET_PEERS_CACHE    = 13
    FAULT              = 99
```

From `applications/nrf_desktop/doc/hid_forward.rst`:

> * Receive the HID input reports from the peripherals connected over Bluetooth®.
> * Forward the HID input reports in report or boot protocol.
> * Forward the :ref:`nrf_desktop_config_channel` data between the peripherals connected over Bluetooth and the host.

## Local check

`grep -rn -i -E "\besb\b|gazell|shockburst" applications/nrf_desktop/` over the checked-out nRF Desktop docs: 0 matches. Only `applications/nrf_desktop/*.rst` and `applications/nrf_desktop/doc/` were materialized, so this covers the docs, not the firmware sources.
