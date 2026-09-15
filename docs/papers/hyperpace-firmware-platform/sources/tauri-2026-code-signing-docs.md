URL: https://v2.tauri.app/distribute/sign/macos/
Additional URLs: https://v2.tauri.app/distribute/sign/windows/ ; https://github.com/tauri-apps/tauri-docs/tree/v2/src/content/docs/distribute/Sign (commit a6b59b78)
Retrieved: 2026-09-15
Source type: official documentation (primary)

## Verbatim, macOS signing

> Code signing is required on macOS to allow your application to be listed in the [Apple App Store] and to prevent a warning that your application is broken and can not be started, when downloaded from the browser.

> Code signing on macOS requires an [Apple Developer] account which is either paid (99$ per year) or on the free plan (only for testing and development purposes). You also need an Apple device where you perform the code signing. This is required by the signing process and due to Apple's Terms and Conditions.

> Note when using a free Apple Developer account, you will not be able to notarize your application and it will still show up as not verified when opening the app.

> Choose the appropriate certificate type (`Apple Distribution` to submit apps to the App Store, and `Developer ID Application` to ship apps outside the App Store).

> To use the certificate in CI/CD platforms, you must export the certificate to a base64 string
> and configure the `APPLE_CERTIFICATE` and `APPLE_CERTIFICATE_PASSWORD` environment variables

> To notarize your application, you must provide credentials for Tauri to authenticate with Apple. This can be done via the App Store Connect API, or via your Apple ID.

> 2. Set the `APPLE_API_ISSUER` environment variable to the value presented above the keys table.
> 3. Set the `APPLE_API_KEY` environment variable to the value on the Key ID column on that table.
> 5. Set the `APPLE_API_KEY_PATH` environment variable to the file path of the downloaded private key.

> 3. Set the `APPLE_TEAM_ID` environment variable to your Apple Team ID.

> Notarization is required when using a _Developer ID Application_ certificate.

> If you do not wish to provide an Apple-authenticated identity, but still wish to sign your application, you can configure an _ad-hoc_ signature.

> Ad-hoc code signing does not prevent MacOS from requiring users to

(line truncated in capture)

```
"signingIdentity": "-"
```

## Verbatim, Windows signing

> Code signing is required on Windows to allow your application to be listed in the [Microsoft Store] and to prevent a [SmartScreen] warning that your application is not trusted and can not be started, when downloaded from the browser.
>
> It is not required to execute your application on Windows, as long as your end user is okay with ignoring the [SmartScreen] warning or your user does not download via the browser.
> This guide covers signing via OV (Organization Validated) certificates and Azure Key Vault.

> This guide only applies to OV code signing certificates acquired before June 1st 2023! For code signing with EV certificates and OV certificates received after that date please consult the documentation of your certificate issuer instead.

> Since 2024, an EV Certificate no longer gives your app an immediate reputation with Microsoft SmartScreen. Microsoft removed the special treatment of EV code signing certificates from its Trusted Root Program in 2024, so EV and OV certificates now build SmartScreen reputation the same way, and a newly signed release can show a warning with either.

> An OV Certificate is generally cheaper and available to individuals. With either kind of certificate, Microsoft SmartScreen may show a warning to users when they download the app until the file and the certificate build enough reputation. Signing every release with the same certificate lets that reputation carry across releases.

> You can sign the Windows executables by providing an Azure Key Vault certific

(line truncated in capture)
