# SUMMARY: OpenMouse README, Linux permissions for VID 0x3554 mice

## Claim
- Linux does not grant user access to hidraw by default.
- For VXE mice under VID 0x3554, OpenMouse recommends narrowly scoped `SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3554", ATTRS{idProduct}=="f58f", TAG+="uaccess"` rules.
- Access to every hidraw node of the product is needed, because Chromium opens the HID device before the app selects the 0xff02:0x0002 configuration collection.

## Method
README of a community WebHID configurator (1,931 stars).

## Result
- Confirms the same vendor ecosystem uses a 0xff02/0x0002 configuration collection.
- One rule per PID, tagging all hidraw nodes, is the practical pattern.

## Evidence tier
3 (community project documentation).

## Performance
Not applicable.

## Correctness
- The "all nodes" requirement is specific to Chromium's WebHID open behavior. A native app could open only the vendor node.
- Matching by VID/PID tags all interfaces anyway, so the rule is the same.

## Relevance to hyperpace
- The udev rule should match `ATTRS{idVendor}=="3554"` with `ATTRS{idProduct}=="fb16"` and `"fb14"`, using TAG+="uaccess", in a file named below 73 (for example 70-hyperpace.rules).
- Also a strong hint (not proof) that HYPACE's report-8 collection is 0xFF02/0x0002.
