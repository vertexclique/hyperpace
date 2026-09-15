# SUMMARY: tauri-2026-code-signing-docs

## Claim
**macOS:**
- Signing avoids the "broken, can not be started" warning for downloaded apps.
- It requires an Apple Developer account ($99 per year; the free plan cannot notarize) and an Apple device.
- Developer ID Application certificates require notarization, using an App Store Connect API key or Apple ID credentials via env vars.
- Ad-hoc signing (`"signingIdentity": "-"`) exists but does not remove Gatekeeper prompts.

**Windows:**
- Signing avoids SmartScreen warnings and is not required to run.
- Since 2024, EV certificates no longer grant instant SmartScreen reputation; EV and OV build reputation the same way.

## Method
The official signing guides.

## Result
CI secrets:
- **macOS:** `APPLE_CERTIFICATE` and `APPLE_CERTIFICATE_PASSWORD`, plus either the API key set (`APPLE_API_ISSUER`, `APPLE_API_KEY`, `APPLE_API_KEY_PATH`) or Apple ID credentials with `APPLE_TEAM_ID`.
- **Windows:** an OV certificate, Azure Key Vault with relic, or a custom sign command.

## Evidence tier
1.

## Performance
Not applicable.

## Correctness
Pricing and policy statements are as of the docs commit (2026-09-13).

## Relevance to hyperpace
- Signing budget and identity are operator decisions.
- Unsigned macOS builds will be hard for users to open.
- Unsigned Windows builds show SmartScreen warnings until reputation builds.
- Code signing also covers embedded firmware (see the blob summaries).
