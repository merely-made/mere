# Scoped Sennet custody, 2026-10-16

Source `94e1f99edafff380d5a22bee6f9c68da01f003e9`, based on upstream
`85f92f09`, uses Dramatis `19be4831`. The default Djinn/Signalman fixture and
public Sennet probe build. Six [actual Keeper cases](broker/qualification.json)
pass: absent/Locked/no-wallet refusal, stable scoped node number on reopen,
and independent sticky closure on lock or resident loss. The unlocked case
also refuses the wrong admitted app's release, sign and attest operations.

Sennet and Tucket share the same `StationCustodyLease` watch. The original
`TucketCustodyLease` name remains available to existing consumers. Exact
device/realm namespaces are distinct; the Sennet child supplies a public
32-bit node number, not RF authentication or a channel key. The receipt
harness retains its existing `bench-tucket-custody` realm string; the actual
released namespace and tested API are Sennet.

The broker runs with isolated synthetic Keeper wallets. Installed personal
vaults are not opened, and private runtime directories are excluded. This
qualifies the cooperative scoped consumer route, not broader platform D13
revocation. Murm R7 separately owns nonce reservations and native/RF results.

Both selected custody policy unit tests pass, including exact scope and
wrong-app release refusal for Sennet and Tucket. Strict port/example Clippy
passes. `policy-tests.log` and `port-clippy.log` retain those gates. Existing
dependency warnings in the default build do not qualify aggregate
Djinn/Graphshell Clippy. The concurrent upstream integration since `85f92f09`
changed documentation/receipts only, preserving the qualified code and lock.
