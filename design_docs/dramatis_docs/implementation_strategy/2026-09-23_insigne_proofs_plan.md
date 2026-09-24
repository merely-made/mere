# Insigne Proofs Plan

**Date**: 2026-09-23
**Status**: plan. Mark agreed the split on 2026-09-23; how issuing is expressed
(§2) is his call before phase A starts. Nothing has moved yet.
**Scope**: move personae's delegation and attestation data types into insigne's
plain-data core and their checks behind an insigne feature, with issuing kept in
personae; then let gaz keep the proofs it receives.

**Related**:

- [crate consolidation plan](../../mere_docs/implementation_strategy/2026-09-23_crate_consolidation_plan.md),
  insigne row: the move this plan executes.
- [gaz founding plan](2026-08-08_gaz_founding_plan.md), M2: gaz keeps the
  proofs themselves (Mark, 2026-09-23), which waits on this plan.
- [device-grant delegation reconciliation](../../mere_docs/technical_architecture/2026-08-11_device_grant_delegation_reconciliation.md):
  it put the delegation grammar in personae. This plan changes where the
  grammar lives, not what it says.
- insigne's crate docs (`crates/dramatis/insigne/src/lib.rs`): the core is
  plain serializable data; a passing check yields a local conclusion that is
  never serialized.

---

## 1. What moves, and what stays

**Moves to insigne's core**, plain serializable data with no hashing and no
signatures, so a holder like gaz takes on no cryptography:

- `DelegationId`, `DelegationParent`, `CapabilityScope` with its
  `attenuates` and well-formedness checks.
- `DelegationCertificate` and `DelegationRevocation`, with their constructors,
  `attenuates`, `covers` and canonical signing bytes.
- `SignedDelegationCertificate` and `SignedDelegationRevocation` as data: the
  statement, the signer's attestation and the signature bytes, with a
  constructor from parts for personae's issuer.
- `DerivedKeyAttestation` as data, with its canonical message.
- `delegation_signing_salt` and `path_covers`.

The domain strings (`personae/delegation-certificate/v1` and the rest) and the
format versions move unchanged, so every signature issued so far still checks.

**Moves behind insigne's verification feature** (`ed25519-dalek` 3, the version
personae already uses, and `blake3`): the three `verify` functions and
`DelegationCertificate::id`, which is a blake3 hash of the signing bytes.

**Stays in personae**: issuing, which needs `IdentityProvider` and the persona's
keys, and `IdentityProvider::attest_derived_key`, which signs with the master.
personae depends on insigne with the verification feature on.

## 2. Decision needed: how issuing is expressed

Once the types live in insigne, personae cannot add inherent methods to them
(the orphan rule), and insigne cannot depend on personae (a cycle). Today
`SignedDelegationCertificate::issue` and `SignedDelegationRevocation::issue` are
inherent methods, and `DerivedKeyAttestation::master_public_key` and
`derived_public_key` return personae's key type. Those are the call sites the
move breaks: **120 in 60 files** (§4). They span mere (graphshell, gemot,
commons, notochord, servitor, castellan, djinn, mesh, murm, personae itself)
and turnstone, knot-editor and mer3ly, which pin mere by rev and so break only
when they repin.

- **(a) An extension trait in personae (recommended).** A trait such as
  `personae::delegation::Issue` provides `issue` and the personae-typed key
  accessors. Call syntax is unchanged; each calling file adds one import.
  personae also re-exports the moved types at their old paths, so the 97 files
  that only name the types change nothing. Issuing stays in personae, as ruled.
- **(b) Free functions in personae**, such as
  `personae::delegation::issue_certificate(&provider, certificate)`. Clearer
  at the call site, but every call site changes shape.
- **(c) A signer trait in insigne that personae's providers implement.**
  Issuing logic would move into insigne, against the ruling, and each of the
  11 `IdentityProvider` implementations (mere, turnstone, hocket, woodshed)
  would need a second implementation. Not recommended.

Also Mark's call: whether this session makes the other repos' import edits
when each repins mere, or leaves them to those repos' own sessions.

## 3. Phases

- **A — relocation, behaviour identical.** Done when:
  - [ ] the types in §1 live in insigne's core and their checks behind its
        verification feature; personae keeps issuing and re-exports the types;
  - [ ] `cargo check --workspace` passes, and the tests of personae, notochord,
        servitor, gemot, commons and castellan pass without edits to test
        logic;
  - [ ] a certificate, a revocation and an attestation serialized and signed
        before the move load and check after it (signing bytes unchanged);
  - [ ] insigne's Ed25519 check accepts exactly what personae's
        `Ed25519PublicKey::verify` accepts (the same strict or lax mode),
        proven by a test against a signature one mode accepts and the other
        rejects.
- **B — conclusions, not booleans.** A passing check returns a local,
  non-`Serialize` conclusion, notochord's `AdmittedPrincipal` rule. Callers
  migrate crate by crate. Done when no caller reads a `bool` from a check and
  `verify() -> bool` is gone.
- **C — re-exports removed.** Consumers import from insigne and personae's
  re-exports go (DOC_POLICY §3), timed to the other repos' repins. Done when no
  crate names the types through personae.
- **D — gaz keeps the proofs** (gaz founding plan, M2). `RootKey` and
  `AttestedKey` carry the artifact that proved them. Done when a gaz record
  round-trips a stored attestation and it checks again after reload.

## 4. Findings

**2026-09-23: the blast radius, counted.** Searched with ripgrep over
`Code/repos`, build output excluded. 787 references in 97 files name the moved
types; 120 call sites in 60 files call `issue` or read an attestation's keys; 11
types implement `IdentityProvider`. Counts from pattern search, not from a
compiler, so the phase A build is the real census.

**2026-09-23: what the checks rest on.** `DelegationCertificate::id` is
`blake3::hash` of the signing bytes, and every check ends in personae's
`Ed25519PublicKey::verify` (`crates/dramatis/personae/src/delegation.rs`,
`crates/dramatis/personae/src/provider.rs`). Hence the verification feature
carries both crates, and phase A's strictness condition.

## 5. Progress

**2026-09-23.** Plan drafted the day Mark agreed the split. Waits on §2.
