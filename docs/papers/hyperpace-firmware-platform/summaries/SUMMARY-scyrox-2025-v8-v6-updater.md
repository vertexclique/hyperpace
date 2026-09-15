# SUMMARY: Scyrox V6/V8 updaters confirm the ODM lineage

- Claim: Scyrox V6/V8 (cid 62) run the same .NET UpgradeTool and carry the "欣春旗" (Xinchunqi) ODM watermark, tying Lofree/Pulsar/Scyrox to one Compx ODM.
- Method: Downloaded both zips, extracted, parsed embedded NRF52840 images, read cfg.ini/en.xml/manual/log.
- Result: cid 62 matches the sample cid baked into Lofree's ODM /home/ template; identical protocol and packaging; AlwaysUpgrade=1.
- Evidence tier: 1 (vendor packages).
- Performance: same protocol.
- Correctness: Self-consistent; Scyrox MCU is NRF52840 (not HYPACE NRF52833), so not flashable to HYPACE.
- Relevance: Confirms the format is ODM-wide, strengthening confidence that the HYPACE uses it identically.
