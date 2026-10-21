# Live Mere owner actions

The native **Open in browser** control uses the existing `OpenAddressV1`,
Hest/Chirograph offer and exact target observation. The resident validates and
journals the request; the receiving native host attempts the system browser
handoff only after the typed acceptance of that exact invocation. A reading,
refusal, stale response or lost reply cannot trigger a launch. Refresh and
reconnect never retry the effect. Local handoff failure is shown separately
from owner acceptance.

Add `"open_web_addresses": true` to the existing profile's `reservoir` owner
settings to enable the offer, then restart the resident. The default is false;
only HTTP/HTTPS addresses are offered. No executable paths or shell command
strings are stored in the graph or settings. This packet enabled it only in
an isolated test profile.

`qualification.json` records 93 passing focused tests and their source hashes.
`accepted/qualification.json` records three real native processes and three
resident starts on macOS 15.8.1: one native activation, one actual Firefox visit
to a unique localhost page, one saved access-history entry, cold reopen without
redispatch, then a disabled owner setting with an empty current offer and
unchanged history. `visual-review.json` records inspection of four presented
native captures. Software commands and compressed logs are in `software/`.
The initial feedback regression is retained as a diagnostic, separate from the
passing runs. The earlier complete cohort remains in the task's external
`output/live-owner-actions-20261020/acceptance-01` directory.

Build the native host separately from the resident to preserve its independent
feature selection:

```sh
cargo +1.99.1 build --locked --offline -j4 -p djinn --features passphrase-fd --bin djinn
cargo +1.99.1 build --locked --offline -j4 -p djinn --example coop_existing_graph
cargo +1.99.1 build --locked --offline -j4 -p graphshell-mere --bin graphshell_mere
python3 support/receipts/domain-actions-live-20261020/run.py --repo "$PWD" --artifacts target/debug --output /tmp/mere-live-evidence
```

Use a fresh output directory and an unlocked real macOS desktop. The controller
reuses the existing isolated-resident and Mesquite native harnesses; the
preparation helper seeds and reads ordinary admitted routes. It creates no
invitation or co-op session. Temporary profiles, vaults and sockets are removed
when the controller finishes. The system browser may retain the local proof tab.

This is qualification of working source, identified by baseline commit and
edited-source hashes. It does not qualify Linux/Windows runtime, physical
pointer/keyboard input, screen-reader activation, a remote owner, or general
page-load/application completion. The successful handoff status is deliberately
not a terminal completion claim.
