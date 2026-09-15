URL: https://raw.githubusercontent.com/DiUS/nRF5-SDK-15.3.0-reduced/master/components/libraries/bootloader/dfu/nrf_dfu_ver_validation.c
Also: https://raw.githubusercontent.com/DiUS/nRF5-SDK-15.3.0-reduced/master/components/libraries/bootloader/dfu/nrf_dfu_validation.c
Doc pages (search results, not opened): https://infocenter.nordicsemi.com/topic/sdk_nrf5_v17.0.2/lib_bootloader_dfu_validation.html, https://infocenter.nordicsemi.com/topic/com.nordic.infocenter.sdk5.v15.3.0/group__nrf__dfu__validation__config.html
Retrieval date: 2026-09-15
Source type: vendor source code (Nordic nRF5 SDK 15.3.0 bootloader), via a third-party GitHub mirror (DiUS)

# nRF5 SDK Secure DFU bootloader: version, signature and bank checks

## Acquisition and limits

- Both files read with the fetch tool on raw.githubusercontent.com. The tool returned code blocks it labeled as verbatim; they were not diffed against Nordic's own SDK zip. The mirror is third party (DiUS), header reads "Copyright (c) 2017 - 2019, Nordic Semiconductor ASA".
- Only the functions below were returned; the full call sites (for example where `is_debug` gates the version check itself) were not shown, so that part is unverified.

## Excerpts (as returned)

### nrf_dfu_ver_validation.c

```c
static bool fw_version_ok(dfu_init_command_t const * p_init)
{
    ASSERT(p_init != NULL);
    ASSERT(p_init->has_fw_version);

    if ((p_init->type == DFU_FW_TYPE_APPLICATION) ||
        (p_init->type == DFU_FW_TYPE_SOFTDEVICE))
    {
        if (!NRF_DFU_APP_DOWNGRADE_PREVENTION)
        {
            return true;
        }
        else if ((p_init->fw_version > s_dfu_settings.app_version))
        {
            return true;
        }
        else if ((p_init->fw_version == s_dfu_settings.app_version))
        {
            return NRF_DFU_APP_ACCEPT_SAME_VERSION;
        }
        else
        {
            return false;
        }
    }
```

```c
else if (p_init->hw_version != NRF_DFU_HW_VERSION)
{
    NRF_LOG_WARNING("Faulty HW version.");
    ret_val = EXT_ERR(NRF_DFU_EXT_ERROR_HW_VERSION_FAILURE);
}
else if (!sd_req_ok(p_init))
{
    NRF_LOG_WARNING("SD req not met.");
    ret_val = EXT_ERR(NRF_DFU_EXT_ERROR_SD_VERSION_FAILURE);
}
else if (p_init->has_fw_version)
{
    if (!fw_version_ok(p_init))
    {
        NRF_LOG_WARNING("FW version too low.");
        ret_val = EXT_ERR(NRF_DFU_EXT_ERROR_FW_VERSION_FAILURE);
    }
}
```

### nrf_dfu_validation.c

```c
if (!NRF_DFU_DEBUG ||
    (NRF_DFU_DEBUG && (p_init->has_is_debug == false || p_init->is_debug == false)))
{
    s_dfu_settings.app_version = p_init->fw_version;
}
```

```c
if (signature_type != DFU_SIGNATURE_TYPE_ECDSA_P256_SHA256)
{
    NRF_LOG_INFO("Invalid signature type");
    return EXT_ERR(NRF_DFU_EXT_ERROR_WRONG_SIGNATURE_TYPE);
}
```

```c
err_code = nrf_crypto_ecc_public_key_from_raw(&g_nrf_crypto_ecc_secp256r1_curve_info,
                                              &m_public_key, pk_copy, sizeof(pk));
```

```c
if (p_signature == NULL)
{
    NRF_LOG_WARNING("No signature found.");
    return EXT_ERR(NRF_DFU_EXT_ERROR_SIGNATURE_MISSING);
}
```

```c
static bool use_single_bank(dfu_fw_type_t new_fw_type)
{
    bool result = false;
    if (((new_fw_type == DFU_FW_TYPE_APPLICATION) ||
         (new_fw_type == DFU_FW_TYPE_SOFTDEVICE)) &&
        NRF_DFU_SINGLE_BANK_APP_UPDATES)
    {
        result = true;
    }
    return result;
}
```

```c
ret_code_t err_code = nrf_dfu_cache_prepare(fw_size,
                                            use_single_bank(p_init->type),
                                            NRF_DFU_FORCE_DUAL_BANK_APP_UPDATES,
                                            keep_softdevice(p_init));
```

Log string: `"Can't find room for update"`.

## Search snippet (vendor doc page, not opened)

> Version checking can be enabled/disabled with the NRF_DFU_APP_DOWNGRADE_PREVENTION config.
