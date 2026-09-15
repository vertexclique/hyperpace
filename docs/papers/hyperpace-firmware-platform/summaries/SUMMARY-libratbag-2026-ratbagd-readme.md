# SUMMARY: libratbag / ratbagd architecture

## Claim
libratbag avoids per-user hidraw permissions entirely by running ratbagd as a root, D-Bus-activated systemd service. GUIs such as Piper talk to it over D-Bus.

## Method
Official README.

## Result
- Requires system-level installation: D-Bus policy, service file and systemd unit.
- No udev rule needed for users; all device access is centralized in a privileged daemon.

## Evidence tier
1 (project documentation).

## Performance
Not applicable. D-Bus round trips add latency, not quantified.

## Correctness
- Proven for libratbag's design.
- A root daemon enlarges the attack surface and cannot be installed by an AppImage without root either.

## Relevance to hyperpace
- An alternative architecture: a tiny root helper that only proxies HYPACE report-8 I/O.
- It needs the same root installation step as a udev rule, with more code and more privilege, so it is unlikely to beat a udev rule. Listed for the operator's option set.
