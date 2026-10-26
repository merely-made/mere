# Castellan custodian integration on macOS

Status: the frozen native01 fixture passed 49/49; this source cut adds the residue repair and has not been requalified as a new native binary; real consumers remain gated.

## Decision and constraints

Use the independently launched macOS custodian as the owner of the actual Castellan sealed-profile vault. The new `djinn-custodian` component links into that signed native process through a private C ABI. The authenticated XPC protocol is a separate, narrower interface. Only signatures, public keys, public status and bounded opaque grants cross it. No IPC failure falls back to an in-process key provider.

This implements the next vertical slice after native11's isolated-runtime-account experiment. Preserve that fixture and its 38/38 receipt as historical evidence; a changed binary requires a new receiving run. Preserve the native10 signer-oracle failure too. The runtime account must not hold or use the signing key. The current production Djinn still links Castellan: this slice does not enable real consumer migration or complete the sole-owner transition.

Retain App Sandbox XPC workers, protected root-owned installation, launchd/SMAppService activation, launch-parent constraints, Hardened Runtime, library validation, signed peer authentication and the exact custodian-only data-protection Keychain group. A compiled expected path and strict bundle validation additionally reject a genuine signed custodian copied to an attacker-writable location before key retrieval. Neither a crate boundary nor six unconfined processes would establish these controls.

## C04: signing consent

Source repair: the general `PersonaeHost::apply_intent` path refuses all three projected signing decisions. Djinn's trusted native composition obtains an authoritative pending request, presents a native confirmation, and completes only if request/profile/expiry/generation are still current. Missing or cancelled UI cannot release the SSH adapter. Lock/unlock and profile changes invalidate confirmation contexts. Direct trusted native authority methods are not IPC routes.

The isolated service applies the same rule to both a root signature and a derived continuous-session grant. Rust creates the pending facts and one-use ticket; the custodian's own dialog supplies the decision. The wire decoder has no approval operation and rejects extra approval booleans. An instance, principal, profile, lock generation, fixed namespace, expiry and use limit constrain grants. The synthetic namespaces are not adopted product purpose/audience policy.

## C07: durable lock outcome

Source repair: the resident serializes memory lock, all registered holders, marker persistence and reported state. Marker errors propagate while memory remains locked. Repeated lock retries the barrier. A clear error during unlock relocks the vault and holders, restores a marker where possible and returns failure. The marker is created exclusively in the same directory, synced, atomically renamed and its Unix directory synced; clearing also syncs the Unix directory. Malformed/unreadable markers count as locked. These calls are not a power-loss receipt or a Windows durability claim.

The new service drops its entire owning vault/storage on lock, clears all approvals/grants and reports persistence failure. Every existing vault starts locked before decryption, independently of marker presence. A user-writable advisory marker cannot authorize auto-unlock: an unrelated same-user app can remove it. Reopen therefore requires OS-confirmed native unlock with fresh Keychain retrieval even after an ordinary restart. Missing/corrupt records never initialize a replacement root under an existing OS key.

## Evidence and release gate

| Boundary | Executed evidence | Still needed |
| --- | --- | --- |
| C04 resident repair | Real SSH adapter tests reject projected decisions; cancelled/unavailable UI cannot decide; stale native contexts refuse | Real receiving dialogs and hostile IPC against this exact binary |
| C07 resident repair | Marker-write failure, retry and clear failure preserve memory/holder lock and report errors | Physical lifecycle and power-loss qualification |
| Castellan integration | Actual encrypted profile/root and slot preservation; independently verified signatures; scope, revocation, expiry, use/capacity limits, corrupt/missing records, marker deletion and native ABI tests | Signed receiving run in the account without signing authority |
| Preserved macOS controls | Native11 historical receipt, new exact signature/identity preflight | All 38 old controls rerun with the new library and host, plus 11 integration controls |
| Migration | No real vault or consumer opened/migrated; synthetic-only configuration and status gate | Inventory every root-opening path and usable key copy, transaction-safe format/identity-preserving migration and rollback |

The new receiving fixture requires 49/49 checks. It runs actual root and derived signatures, forged approval requests, adjacent scope requests, real lock-I/O failure and retry, attacker removal of the advisory marker, cold restart locked, native OS unlock, old-grant refusal, revocation, use exhaustion and valid signed-copy denial. The root and derived signatures are independently verified by CryptoKit. Evidence evaluator tests reject absent or incomplete native evidence; they cannot substitute for it.

The additional heap-residue instrument found two failing passphrase-storage scenarios in this source cut: an uncleared master-seed canary in a freed 88-byte allocation after profile switching, and one in a live 176-byte allocation after vault unlock/relock. Sealed-profile storage, sealed-vault lock, authoritative-store lock and agent-listing scenarios passed, as did the separate resident-lock residue instrument. Baseline comparison and root cause are separate evidence, not inferred from unchanged source. This finding blocks real consumers even if all 49 native integration checks pass. The instrument excludes stack copies and OS allocations; a passing scenario is limited to its measured heap canaries.

Even a passing synthetic receiving run does not enable real consumers automatically. Production release additionally requires complete consumer routing, no alternate root-opening fallback, private-data/blob-owner cuts, actual purpose/audience policy, bounded production framing/resource behavior, import isolation, all custody-copy invalidation, lock/screen-lock/suspend/logout/reboot handling, residue qualification and signed update/rollback behavior. Linux and Windows retain separate qualification gates.

The historical attacker decoder understands the old CryptoKit envelope. Any obtained wrapping key or available signing authority fails the new gate, but that decoder is not a positive control for Castellan's storage codec. User cancellation, missing files, arbitrary signing failures, timeouts, unavailable providers and ENOENT are not passing denials.

Implementation: [service core](../../../ports/djinn/custodian/README.md) and [receiving harness](../../../testing/macos-castellan-custodian-h1/README.md). Machine-readable receipts pin the source, artifact, tests, failures and pending receiving status; they must keep `real_consumers_allowed` false until the applicable gates are explicitly satisfied.
