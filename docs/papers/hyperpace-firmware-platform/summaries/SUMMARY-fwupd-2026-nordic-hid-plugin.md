# SUMMARY: fwupd nordic_hid plugin

**Claim.** fwupd ships a nordic_hid plugin that updates nRF Desktop devices over the config channel on Linux via HIDIOCSFEATURE/HIDIOCGFEATURE. It uses a CAB containing a signed image zip; GUIDs encode board name, bootloader (B0 only tested; MCUboot experimental) and generation. Config-channel DFU cannot be done through serial recovery. It deploys in runtime mode and resets the device.

**Method.** Fetched the fwupd plugin README (text extraction) and Nordic fwupd.rst (local rst).

**Result.** Consistent between the two.

**Evidence tier.** 1 (project docs).

**Performance.** Not discussed; same transport as the config channel.

**Correctness.** Proven: Linux hidraw ioctls suffice, no kernel driver needed. Known limitation from issue 5738: peripherals behind a dongle report the dongle's VID/PID, breaking GUID matching, unresolved. Not applicable to HYPACE unless Lofree uses nRF Desktop.

**Relevance to hyperpace.** Confirms the Linux access model hyperpace plans (hidraw + uaccess udev rule). If HYPACE ever proves nRF-Desktop-based, fwupd/LVFS is a ready path; otherwise hyperpace must implement the Compx upgrade protocol itself.
