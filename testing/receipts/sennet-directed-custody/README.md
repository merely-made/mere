# Scoped Sennet material, 2026-10-16

The Signalman port adds transient `SennetStationMaterial` to the existing
device/realm-scoped release, without extending any broker policy or namespace.
Dramatis `2f27f1a` supplies zeroizing material with redacted Debug. The R7 public
node and sticky custody guard remain available. Murm owns protocol-specific
construction, public peer pinning and the pending directed-text/ACK RF gate.

The updated public-only probe passes all six real-Keeper cases, including
same scoped material on reopen, wrong admitted app release/sign/attest refusal,
absent/Locked/no-wallet refusal and sticky lock/resident-loss closure. The host
is the unchanged qualified synthetic Keeper binary with SHA-256 `25f41c90...`;
its full hash and the new probe hash are recorded in `broker/qualification.json`.
The raw report identifies base HEAD and actual source hashes for the working
supplier patch. Private fixture roots are ignored in Murm and excluded here.

The probe builds. Strict selected port/example Clippy passes with `--no-deps`.
The broader workspace-dependency invocation fails on pre-existing Tabard lints;
`port-clippy.log` retains that failure separately from the selected passing log.
This does not qualify aggregate Djinn/Graphshell Clippy or broader platform D13.
