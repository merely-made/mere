# Insigne Proofs Plan

**Date**: 2026-09-23
**Status**: phase A landed 2026-09-24; phases B–D open. Mark agreed the split
and ruled how issuing is expressed (§2, option (a)) on 2026-09-23. The Mere
0.4 release baseline waits for phase B (Mark, 2026-09-26).
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
  `covers` and canonical signing bytes (`signing_bytes`, now public).
  `DelegationCertificate::attenuates` names its parent by hash, so it went
  behind `digest` with `id` (§4, 2026-09-24).
- `SignedDelegationCertificate` and `SignedDelegationRevocation` as data: the
  statement, the signer's attestation and the signature bytes, with a
  constructor from parts for personae's issuer.
- `DerivedKeyAttestation` as data, with its canonical message.
- `delegation_signing_salt` and `path_covers`.

The domain strings (`personae/delegation-certificate/v1` and the rest) and the
format versions move unchanged, so every signature issued so far still checks.

**Moves behind insigne's features**, two as built: `digest` (`blake3`) carries
`DelegationCertificate::id`, a blake3 hash of the signing bytes, and
`DelegationCertificate::attenuates`; `verify` (`ed25519-dalek` 3, the version
personae already uses) carries the three `verify` functions and implies
`digest`.

**Stays in personae**: issuing, which needs `IdentityProvider` and the persona's
keys, and `IdentityProvider::attest_derived_key`, which signs with the master.
personae depends on insigne with `verify` on.

## 2. Decision: how issuing is expressed (ruled (a), 2026-09-23)

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
  personae also re-exports the moved types at their old paths, so the 98 files
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

**Ruled 2026-09-23: (a)**, and for the other repos "you do repins or
communicate to 'em". Built as two traits rather than one: `Issue` in
`personae::delegation` provides `issue` for both signed types, and
`AttestationKeys` at personae's root provides `master_public_key` and
`derived_public_key`. insigne's attestation also gained plain accessors
(`master`, `derived`, `signature`) and typed ones (`master_key`,
`derived_key`, as `TypedKey`).

## 3. Phases

- **A — relocation, behaviour identical.** Landed 2026-09-24. Done when:
  - [x] the types in §1 live in insigne's core and their checks behind its
        features; personae keeps issuing and re-exports the types at their
        old paths;
  - [x] `cargo check --workspace` passes, and the tests of personae, notochord,
        servitor, gemot, commons and castellan pass without edits to test
        logic. The portable gate passed at 1,525 packages. Passing: personae
        111, notochord 43, servitor 92, gemot 97, commons-spine 67 and castellan
        59. Beyond the list, stickleback 88, mere-mesh 125, mere-transport 49,
        gaz 54, djinn's chronicle route 2 and graphshell's two integration
        tests also passed. Test edits were import lines, plus one change of
        mechanics: personae's attestation tamper test now builds the tampered
        value with `from_parts`, because the field is private in insigne. What
        it tampers and what it asserts are unchanged;
  - [x] a certificate, a revocation and an attestation serialized and signed
        before the move load and check after it:
        `crates/dramatis/insigne/tests/pre_move_personae.rs`, against a fixture
        personae issued at mere `3943874f`. Each re-serializes to the stored
        JSON and checks, and the certificate keeps its id;
  - [x] insigne's Ed25519 check accepts exactly what personae's
        `Ed25519PublicKey::verify` accepts (the same strict or lax mode),
        proven by a test against a signature one mode accepts and the other
        rejects. The same test file uses the identity point as both key and
        `R` with `s = 0`: ed25519-dalek's lax `verify` accepts it,
        `verify_strict` refuses it, and insigne accepts it.
- **B — conclusions, not booleans.** A passing check returns a local,
  non-`Serialize` conclusion, notochord's `AdmittedPrincipal` rule. Callers
  migrate crate by crate. Done when no caller reads a `bool` from a check and
  `verify() -> bool` is gone. The Mere 0.4 release baseline (crate
  consolidation plan, C5) waits for this phase, so insigne publishes its
  settled API once (Mark, 2026-09-26: "after B").
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
types implement `IdentityProvider`. A plain grep over the same tree finds 99
files: ripgrep skips `crates/probes/`, which `.gitignore` excludes, although one
probe there (`murm-direct-phy`) is force-tracked and names the types. So 98
tracked files. Counts from pattern search, not from a compiler, so the phase A
build is the real census.

**2026-09-23: what the checks rest on.** `DelegationCertificate::id` is
`blake3::hash` of the signing bytes, and every check ends in personae's
`Ed25519PublicKey::verify` (`crates/dramatis/personae/src/delegation.rs`,
`crates/dramatis/personae/src/provider.rs`). Hence the verification feature
carries both crates, and phase A's strictness condition.

**2026-09-24: the compiler's census.** In mere, 49 files across 11 crates needed
the trait import (+56 −3 lines). graphshell had 18, gemot 9 and notochord 7.
knot-editor needed 5 files. The imports followed rustc's own suggestions, with
one trap: rustc names the package, `personae::`, even in crates that depend on
it as `identity` (gemot, stickleback, servitor, mere-mesh, mere-transport). That
path does not resolve there, and phase C's import rewrite will meet the same
trap.

**2026-09-24: mere and knot-editor compile each other.** djinn pins knot-editor,
and mere's `[patch]` table serves every mere package Knot names from this tree.
So knot-editor's non-test code (`publish.rs`) compiles against mere's working
personae, and a breaking change to a contract Knot names has to land in both
repos at once. Here, knot-editor `c6d5b9e` added the imports first. Its
standalone build stayed red until it repinned to this move, and mere pinned it.
Mark chose that brief break over pinning a branch on 2026-09-24. Phase C will
meet the same cycle. The repin also removed genet `5ae30cad` from mere's graph.
The old knot pin had pulled `fleece` and `layout-dom-api` in a second time, and
without them the graph went from 1,527 packages to 1,525.

**2026-09-24: `attenuates` hashes.** `DelegationCertificate::attenuates` requires
`parent == Certificate(parent.id())`, so it needs `digest`. §1 had put it in the
plain core. `CapabilityScope::attenuates` stays plain.

**2026-09-24: two test targets the portable gate never compiles.**
`scripts/cargo_mode.py verify` runs `cargo check --workspace` without
`--all-targets`. With it, graphshell's lib tests (`ports/graphshell/src/app.rs:387`,
a match missing `RelationKind::OpenPredicate`) and cambium-nematic's
(`crates/cambium/cambium-nematic/src/views.rs:513` *(historical citation)* <!-- doc-audit: historical-path -->, a `FeedEntry` missing six
fields) fail to compile, both before this move and after it. So graphshell's
own unit tests could not run for phase A. They compile through type checking,
which covers their imports. Resolved since: graphshell's test by `bc7121ae`,
cambium-nematic's by `4b33a963`, and the gate has checked every target since
`67ebef3a` (Mark's ruling).

## 5. Progress

**2026-09-23.** Plan drafted the day Mark agreed the split. Waits on §2.

**2026-09-24.** Phase A landed as `5364dfa0`, pinned to knot-editor `c6d5b9e`
(§4). knot-editor then repinned to it as `a08c1f7`, which made its standalone
build green again. Repin handoff for the other repos that pin mere by rev,
found by pattern search; the compiler is the real census at each repin. The
repos' sessions were offline when this landed, so this note is the handoff.

- turnstone needs `personae::delegation::Issue` in five files:
  `turnstone/src/denizen.rs`, `turnstone/src/place/lanes.rs`,
  `turnstone/src/place/projection_host.rs`, `turnstone/src/place/worker.rs` and
  `turnstone/src/remote_projection.rs`. It has no mere patch table, so its
  knot-document pin (`44f0519`) must move with mere to a knot commit that pins
  the same mere rev (`a08c1f7` for `5364dfa0`). Otherwise the graph holds two
  mere revisions.
- mer3ly needs the same import in `mer3ly/crates/repo-graph/src/lib.rs`.
- hocket, cleromancy and woodshed call neither API.

`AttestationKeys` is needed wherever an attestation's `master_public_key` or
`derived_public_key` is read; none of these repos does today. Next: phase B.

**2026-09-26.** Mark ruled that the Mere 0.4 baseline waits for phase B.
mere's djinn moved its knot pin from `c6d5b9e` to knot's main (`5ad3f67`).
That drops the git-sourced `graphshell-stdio` and a second genet
(`532f1fad`'s `fleece` and `layout-dom-api`) that the old pin kept in the
graph.
