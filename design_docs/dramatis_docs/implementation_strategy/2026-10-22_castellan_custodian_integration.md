# Castellan custodian integration

Status (2026-10-26): the portable custodian landed on main. `djinn-custodian` is now a separate process behind one contract (`custody-wire`), with launchers for Linux, macOS and Windows. Its negative controls pass on macOS, and the Linux and Windows builds type-check. Djinn routes the two H1 consumers through it when one is installed. Linux, signed-macOS and Windows receiving runs, and every other consumer (H2), are still gated. The frozen native01 fixture's 49/49 qualifies only that synthetic binary. The residue repair is on main and its residue scenarios pass.

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

Implementation: [service core](../../../ports/djinn/custodian/README.md) and [receiving harness](../../../testing/macos-castellan-custodian-h1/README.md). The harness still links the synthetic core (`djinn_custodian::synthetic`) through the `native-host` C ABI. Its receipts were retired with the residue-fix worktree on 2026-10-26; the results that matter are recorded here and in the repair plan.

## Portable cut, 2026-10-26

The macOS slice above was the starting point. The cut on main makes the custodian a process on every desktop OS. It owns vault open, unlock, lock, root and root-derived signing, and reviewed derived-key release:

- **Contract.** `custody-wire` implements §5.6 once. The peer principal comes from the OS connection: `SO_PEERCRED`, `getpeereid`, or the named-pipe client SID, plus a code-signature identity from the macOS audit token. It never comes from the hello. Sessions open with version, a fresh instance, the lock generation and the granted scopes. Operations are typed, with a request id and an expected instance and generation. Consent petitions carry a custodian-derived digest. Decisions count only from the trusted channel and only for that digest, request and generation. Outcomes are typed, and a disconnect is `Unknown`. Lock, unlock and revocation are monotonic facts. Frames keep the Graphshell door's wire and clear their buffers; the door module itself is not linked.
- **Core.** It is transport-free and holds Djinn's real passphrase vault, locked at every start. It reopens only the configured, remembered or sole profile, refuses a changed identity and never mints one. Policy is installed, deny-by-default and per principal. A desktop-uid/SID principal cannot sign or release unattended, or be the consent channel, outside declared development mode.
- **Launchers.**
  - Linux: a socket-activated systemd system service under `djinn-custodian-USER`. It applies `no_new_privs`, non-dumpable, Landlock (ABI 6 in protected mode) and a seccomp allowlist before input. polkit `auth_self` consent runs from a helper forked before confinement.
  - macOS: a launchd agent adopting its launchd socket, with deny-attach and a hardened-runtime check. Both ends identify each other by code signature.
  - Windows: an SCM service under its virtual account, serving a pipe with a protected DACL and client and server SID checks, and process mitigations.
- **Djinn.** Djinn is a client for the two H1 consumers. Graphshell's `mere.graphshell` guest delegations are decoded, checked for audience, issuer and scope key, and signed inside the custodian. Distillery's mesh author receives exactly its two release salts, checked against the custodian's attestations. Distillery's second vault open is gone in every mode, so Djinn no longer keeps the start's passphrase for it. A locked or unreachable custodian leaves identity pending; nothing falls back to Djinn's own vault.

Evidence (macOS, this checkout; Linux and Windows type-checked for `x86_64-unknown-linux-gnu` and `x86_64-pc-windows-gnu`):

| Suite | Result |
| --- | --- |
| `custody-wire` | 8 tests pass. Frames refuse oversize. Decoders refuse forged approval, a principal in the hello and extra fields. The client sends nothing to an unexpected server, and a lost connection is `Unknown`. An unsigned peer never passes as a signed identity. |
| `djinn-custodian` core and socket | 16 tests pass, plus the 13 synthetic-core tests. Refused: wrong profile, adjacent namespace, mislabelled audience and signing oracle. Consent is bound to digest, request and generation; self-approval is refused; per-generation consent resets on lock. A lock during a pending petition invalidates it. A non-durable barrier degrades the lock without unlocking. A stale instance or generation and readmission after revocation are refused. Expiry, withdrawal, budgets and the handshake deadline hold. |
| `djinn-custodian` process | 4 tests pass with the real binary. It starts locked. Its loss is `Unknown`, and a restart is locked under a fresh instance, refusing the old one. Adoption keeps the identity and leaves the source byte-identical. Protected mode refuses a self-bound socket. |
| Djinn `custodian_h1` | 4 tests pass. Distillery's author is released for exactly its salts. A Graphshell delegation is signed by the custodian's key, not Djinn's. A foreign issuer is refused. A locked or absent custodian refuses with no fallback. A user-writable client file is refused. |

Still gated, recorded in the hardening plan's §6 and §8:
- the Linux receiving run on the ThinkPad: seccomp allowlist tuning, Landlock ABI, file, ptrace and socket denial, polkit prompts and restart;
- the signed macOS install and receiving run, including the Keychain envelope;
- the Windows service, ACL and pipe qualification on o-pc;
- every other Djinn vault use (H2): done since, in the next section;
- transactional migration that retires the original vault and its copies.

## H2 consumer migration, 2026-10-26

The custodian now hosts Castellan's resident host (`PersonaeHost`), the wallet, the credential catalog and the SSH agent. Djinn's custody is one trait, `ResidentCustody`. A production build has only its client, `RemoteHost`, and links no castellan. The in-process host survives only under the `in-process-custody` feature, for test suites and receipt hosts, and is never a fallback. The hardening plan's [H2 section](../../archive_docs/2026-10-28/2026-10-21_custody_isolation_and_security_hardening_plan.md#h2--complete-authority-and-consumer-migration) maps each consumer to its §5.6 operation, and records two things:
- Hocket adoption is stopped: it needs an identity-format change.
- The wallet is migrated at the operation level, but its secret files move into custody only after a public/secret file split.

Evidence (macOS, after merging main):
- `custody-wire` passes 8 tests, and `djinn-custodian` 38 unit tests (9 of them H2) and 4 process tests;
- Djinn's `custodian_client` (8), `quarantine_review` (16) and `cxf_import` (1) run against a real custodian;
- the resident end-to-end test `custodian_resident` (run with `--ignored`) passes;
- the rest of Djinn's suite passes, except the replay case that fails on plain main too;
- a default Djinn build links no castellan.

Linux and Windows type-check only.

Still gated: the Linux and signed-macOS receiving runs (custodian README), the wallet split, and Windows consent, vault migration and an agent pipe.
