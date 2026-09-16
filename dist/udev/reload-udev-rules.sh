#!/bin/sh
# Reloads udev rules and re-triggers hidraw events so 70-hyperpace.rules
# takes effect on an already-plugged-in device without a replug. Installed
# as the post-install and post-remove maintainer script for the deb and rpm
# packages (see ../README.md); the package manager runs it as root during
# install and removal, it is never invoked directly by this build.
if command -v udevadm >/dev/null 2>&1; then
    udevadm control --reload-rules
    udevadm trigger --subsystem-match=hidraw
fi
exit 0
