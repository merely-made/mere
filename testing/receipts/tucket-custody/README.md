# Tucket custody qualification, 2026-10-15

This is the Djinn/Signalman custody gate for Murm's R6 Tucket desktop trial.
Personae's device UUID and realm derivation is supplied by Dramatis commit
`5c89001a6ff1aac3f6105ed50b6ae5f09c9f27e2`. These probes do not open a radio.

`initial/qualification.json` passes against the real Keeper and custody route:
absent, Locked and no-wallet refusal; stable public identity on reopen;
another admitted app's release/sign/attest refusal; independent lock-watch and
resident-loss closure. The watch probe waits on `check_live`, without polling
status to cause the closure.

`negative-control/qualification.json` records the deliberately restored
application-local seed fallback. Both absent and Locked probes fail with exit 1
and the expected forbidden-fallback diagnostic. `run-negative-control.ps1`
records the temporary source change and restores the clean source in `finally`.

`restored/qualification.json` is a retained failed restoration check: Copy-Item
preserved the old source timestamp, so Cargo reused the negative executable.
The check detected it before any RF use. The recipe now refreshes that timestamp;
`restored-verified/qualification.json` passes the final clean-binary gate.
`source-restoration.json` identifies the exact original/restored source hashes.
Both policy tests pass in `policy-tests.log`. Strict port/example Clippy passes;
strict Djinn Clippy retains existing sync/door failures in `build-logs/`.

Generated synthetic wallets are kept only in ignored `runtime/` directories.
Their helpers are stopped. Automatic approval review blocked their recursive
deletion with "blocked by policy"; private runtime data must not be staged.
The public reports record executable and source hashes, never private keys.
