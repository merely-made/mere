# Castellan custody service core

Status: the frozen native01 fixture passed 49/49; this source cut adds the residue repair and has not been requalified as a new native binary; real consumers remain gated.

`djinn-custodian` owns `SealedProfileStorage` and `IdentityVault` in the independently launched native process. It uses Castellan's existing sealed profile records and derivation implementation. The native host supplies the 32-byte OS storage key from its own Keychain group. Only that host links the `native-host` C ABI; it is not the IPC interface. Workers and clients receive signatures and public keys, never a storage key, root seed or vault handle.

The initial API requires the `h1-synthetic` profile and a matching run marker. Every status reports `realConsumersAllowed: false`. This is a production component under synthetic qualification, not an enabled replacement for Djinn's current in-process consumers.

## Authority

The native listener authenticates its signed peer before selecting the principal. The Rust decoder rejects unknown fields and operations, bounds requests to 16 KiB and challenges to 4096 bytes, and checks instance, profile and operation namespace. Each root signature and scoped-session grant requires a custodian-owned native confirmation using authoritative pending facts. Client approval booleans and decision operations are invalid requests.

Scoped signing uses a Castellan-derived key, with an opaque grant bound to principal, profile, instance and lock generation. Grants last at most 60 seconds and eight uses; pending confirmations and sessions each have an eight-entry limit. Lock, restart, expiry and explicit revocation invalidate grants. A native confirmation is one use, expires after 120 seconds and cannot survive lock.

Every existing vault starts locked before profile decryption, even if a same-user app removes the advisory marker. Unlock requires native OS user verification and a fresh key retrieval. Only a newly created OS key may initialize a synthetic profile; missing or corrupt existing records never regenerate an identity. Failed lock persistence drops the whole owning vault, revokes grants, returns an error and retries on the next lock. Failed marker clear drops the reopened vault and refuses unlock.

## Qualification

Rust tests use actual encrypted records and cryptographically verify root and derived signatures. They cover forged approval, stale confirmation, principal/instance mismatch, scope, exhaustion, revocation, expiry, bounded entry points, corrupt/missing records, wrong keys, marker deletion and persistence errors. A relocation test preserves the root and a direct slot payload in the existing format; it does not implement transactional migration.

The [macOS harness](../../../testing/macos-castellan-custodian-h1/README.md) adds signed installation, confinement, independent launch, same-user attacks, native dialogs, cold restart, lock failure and real Castellan consumption. Passing Rust or evidence-evaluator tests does not qualify those native checks.

Real migration additionally needs an inventory of every usable key copy and root-opening path, transactional format-preserving migration and rollback, product purpose/audience policy, bounded production framing, all lock/lifecycle hooks and adversarial receiving evidence. Djinn's existing authority remains a transitional owner; this service has no client fallback to it.
