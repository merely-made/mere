# Insigne Proofs Plan

**Date**: 2026-09-23
**Status (2026-09-29)**: phase A landed 2026-09-24 and phase B on 2026-09-26
(§3); C landed in Mere and Knot on 2026-09-29; sibling repins remain open.
D landed in Gaz on 2026-09-29: stored artifacts reload and check again.
Mark agreed the split
and ruled how issuing is expressed (§2, option (a)) on 2026-09-23. The Mere
0.4 release baseline's Insigne prerequisite was phase B, met on 2026-09-26.
**Scope**: move personae's delegation and attestation data types into insigne's
plain-data core and their checks behind an insigne feature, with issuing kept in
personae; then let gaz keep the proofs it receives.

**Related**:

- [crate consolidation plan](../../mere_docs/implementation_strategy/2026-09-23_crate_consolidation_plan.md),
  insigne row: the move this plan executes.
- [gaz founding plan](2026-08-08_gaz_founding_plan.md), M2: gaz keeps the
  proofs themselves (Mark, 2026-09-23), implemented by phase D of this plan.
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
personae already uses) carries the three statement `check` methods and their
conclusions, and implies `digest`.

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
  crate names the types through personae. The Mere/Knot graph meets this
  condition at `7926d3a8`/`855cb75`; the sibling pins listed in §5 still await
  their own adoption gates.
- **D — gaz keeps the proofs** (gaz founding plan, M2). `RootKey` and
  `AttestedKey` carry the artifact that proved them. Done when a gaz record
  round-trips a stored attestation and it checks again after reload. Met
  2026-09-29 by JSON and postcard book reloads followed by real signature checks.

### B, as ruled 2026-09-26

Mark chose both options below from named alternatives.

**The conclusion is borrowed and unforgeable.** Each statement gets a
`check`, behind `verify`. It returns `Result<Checked…<'_>, CheckFault>` and
leaves the old `verify` in place, deprecated, until no caller needs it:

- `DerivedKeyAttestation::check(salt)` returns `CheckedAttestation`;
- `SignedDelegationCertificate::check()` returns `CheckedCertificate`;
- `SignedDelegationRevocation::check()` returns `CheckedRevocation`.

A conclusion borrows the statement it was drawn from and has private
fields, so only a passing check makes one. It is not serializable. It
exposes the statement and, for the signed kinds, the checked signer.
`CheckFault` says which step failed: `Malformed`, `BadAttestation`,
`WrongIssuer` or `BadSignature`. The steps run in the order the old
`verify` ran them, so exactly the same statements pass. "Check" is
insigne's own word: its crate docs say a check that passes yields a local
conclusion. The alternatives were owned copies of the facts, keeping the
name `verify` in one breaking step, or public fields as `AdmittedPrincipal`
has, which anyone could forge.

**The conclusion travels to the first decision, and through notochord.**
Every `if !x.verify()` becomes a `check` whose fault the caller keeps.
notochord's `RevocationLedger::fold` takes a `CheckedRevocation`, so a ledger
cannot fold an unchecked statement. `validate_chain` returns the checked
leaf, so admission asks `covers` of a checked certificate. The alternatives
were call sites only, or demanding conclusions in every API that receives a
statement (pandect's wallet grants, gemot's store, graphshell's carriers).

**The census, by compiler.** The three `verify` functions were marked
deprecated in a scratch tree, and the workspace was built with every target.
That found 60 call sites in 29 files across 11 crates: personae 19, insigne
itself 10, pandect 8, gemot 7, graphshell, commons, servitor and notochord 3
each, stickleback 2, mere-mesh and castellan 1 each. signalman adds 1: it is
its own workspace, which the workspace build never reaches (§4, 2026-09-26).

Outside mere, by pattern:

- turnstone: 1 `verify` and 2 `fold` calls;
- hocket: 1 `verify`, with the same code vendored in woodshed;
- knot-editor: 2 test assertions and 1 `fold` in an example.

mere compiles only knot-editor's library, which calls neither, so this
phase needs no knot lockstep. Each of those repos migrates at its next repin.

**Order.** Each step keeps the workspace green:

1. insigne gains `check` and the conclusions, and deprecates `verify`.
2. notochord changes `fold` and `validate_chain`, updating their callers in
   the same commit.
3. The remaining call sites move crate by crate.
4. `verify` goes.

Done when (landed 2026-09-26):

- [x] insigne's three `check` functions and their conclusions exist
      (`bd6b8c4b`), and `verify() -> bool` is gone (`538226a3`);
- [x] `fold` takes a `CheckedRevocation`, and `validate_chain` returns the
      checked leaf, which notochord's admission asks `covers` of
      (`e4471b93`);
- [x] no caller in mere, signalman included, reads a `bool` from a check
      (`87aae7ab`), with one exception, which Mark confirmed on 2026-09-26. castellan's
      `DeviceGrantView` reports "verified" or "invalid" in a serialized
      view: it can carry the outcome, but never the conclusion. Functions
      whose own contract is a `bool` or an `Option` keep it, built on the
      conclusion: notochord's `verify_proof`, graphshell's
      `verify_derived` and `verify_session_key`, mesh's `attests`, and
      personae's SSH ledger `fold`;
- [x] the portable gate passes (1,521 packages), and the workspace checks
      with every target and every feature. signalman checks and tests
      with its own command. The moved crates' tests pass, the pre-move
      fixture tests included: insigne 16 + 7, ten crates with all
      features 1,086, signalman 22, and graphshell's library 190. One
      graphshell golden test fails on Windows checkouts regardless (§4);
- [x] the handoff for turnstone, hocket, woodshed and knot-editor is
      recorded here (§5, 2026-09-26).

## 4. Findings

**2026-09-29: phase C starts from current consumers.** Knot now pins Mere
`8fce5365` and has adopted phase B. Insigne already exposes the replacement
imports at that pin, so Knot can move its imports before Mere removes the
re-exports without deliberately breaking its standalone build. Mere must
also patch Knot's new `insigne` dependency to the local workspace package,
as it does for the other shared contracts. The primary Mere checkout has an
active Genet repin in its manifest and lock, so this phase uses the isolated
`mere-insigne-phase-c` worktree; that repin remains outside this change.

The tracked `crates/probes/murm-direct-phy` probe still names obsolete
Personae and sibling Retinue paths. Updating its proof import does not make
its existing standalone build reproducible. Native `--all-features` also
does not reach Graphshell's `cfg(wasm32)` imports; the browser check is a
separate gate.

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

**2026-09-26: phase A missed a nested workspace.** `ports/signalman` is its
own workspace, excluded from mere's, but it path-depends on personae. So
neither the portable gate nor phase A's workspace check compiled it. It
reads an attestation's `master_public_key` and `derived_public_key`, and it
failed with E0599 from `5364dfa0` until `aacf39c7` imported
`AttestationKeys`. One other nested workspace depends on the proof crates,
`ports/distillery/probe/remote-fixture`, and it calls none of the moved
APIs. From here on, each nested workspace that depends on these crates is
checked with its own command.

**2026-09-26: a census needs every feature.** Phase B's first compiler
census built default features. `--all-features` found two more sites:
castellan's `reticulum` grant check called `verify`, and murm's
`session-lane` test had lacked phase A's `Issue` import since `5364dfa0`
(fixed in `b2677e15`). Like a gate, a census proves nothing about code
behind a feature it did not enable.

**2026-09-26: the nested workspaces, checked.** signalman checks and tests
with its own command. `ports/distillery/probe/remote-fixture` does not
resolve at all, locked or not: mere-mesh requires
`mere-p2panda-net = "=0.7.4"`, which only mere's root patch table
supplies. That breakage predates both phases. Its tracked lock is also
stale: it has had no `insigne` entry since phase A.

**2026-09-26: two unrelated failures met on the way.**

- graphshell's `distillery_w1::…_mount_resumes_by_diff…` test compares
  its receipt with a golden JSON, and a Windows checkout gives that file
  CRLF endings. The values are identical. Git Bash's grep strips the
  `\r` before matching, so only a byte-level read (Python) shows it.
  tabard's goldens had the same fault, fixed with `eol=lf` (`b7ed0bdc`).
- `mere-linked-data`'s tests name `Node.properties`, which the graph
  kernel no longer has. Only an all-features build reaches them.

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

**2026-09-26.** Phase B landed (§3): `0f9eaa52` (plan), `bd6b8c4b`,
`e4471b93`, `87aae7ab`, `538226a3`, plus two forward fixes for phase A,
`aacf39c7` (signalman) and `b2677e15` (murm). Its API is now the one
insigne publishes, so the Mere 0.4 baseline's insigne condition is met.

Repin handoff for phase B, adding to phase A's. `x.verify()` becomes
`x.check()`, which returns the conclusion or a `CheckFault`, and
notochord's `ledger.fold(&statement)` becomes
`ledger.fold(statement.check()?)`, or a match on the fault.

- turnstone: `turnstone/src/place/worker.rs:1294` (`grant.verify()`),
  `turnstone/src/publish_service.rs:390` (`fold`) and
  `turnstone/src/place/worker.rs:3801` (a test's `fold`);
- hocket: `hocket/crates/hocket-engine/src/handoff.rs:233`
  (`self.sender.verify(&salt)`). woodshed carries the same file under
  `ports/hocket/`;
- knot-editor: two test assertions in
  `knot-editor/crates/knot-editor/src/publish.rs` and the `fold` in
  `knot-editor/crates/knot-editor/examples/knot_publish_peer.rs`.

Next: phase C.

**2026-09-26.** Mark ruled that the Mere 0.4 baseline waits for phase B.
mere's djinn moved its knot pin from `c6d5b9e` to knot's main (`5ad3f67`).
That drops the git-sourced `graphshell-stdio` and a second genet
(`532f1fad`'s `fleece` and `layout-dom-api`) that the old pin kept in the
graph.


**2026-09-29.** Phase C implementation: native compiler census used temporary
warnings on all ten proof types/helpers and
`cargo check --workspace --all-targets --all-features --keep-going`. It reported
2,440 diagnostics in 90 files across 16 packages, including uses through
local aliases and inferred values. The temporary warnings were removed;
Insigne's implementation and serialized formats are unchanged. The source
rewrite also covers wasm-only imports, Signalman, and the force-tracked probe.

Knot's six affected files now import Insigne directly. `855cb75` is pushed to
Knot's main; `cargo check -p knot-editor --all-targets --all-features` passed
at its existing Mere `8fce5365` and Genet `34626a6c` pins. No red standalone
interval was necessary. Mere consumes that Knot revision and supplies the
new local Insigne patch.

Remaining sibling repin handoff (source census, not compiler receipts):

- Turnstone: `src/denizen.rs`, `src/identity.rs`, `src/place/lanes.rs`,
  `src/place/projection_host.rs`, `src/place/worker.rs`, and
  `src/remote_projection.rs`.
- Hocket: `crates/hocket-engine/src/handoff.rs` and
  `crates/hocket-genet/src/identity.rs`. Woodshed carries the same two paths
  under `ports/hocket/`.
- mer3ly: `crates/repo-graph/src/lib.rs` (test dependency).
- Cleromancy has no proof-type import to migrate in the tracked Rust source.

At each repin, add Insigne from the exact same Mere revision as Personae.
Import `DerivedKeyAttestation` from `insigne` and the delegation data/helpers
from `insigne::delegation`; retain `Issue`, `DelegationError`,
`AttestationKeys`, and providers in Personae (under `identity` where aliased).
Enable `verify` where checks are called, and `digest` where only certificate
ids or attenuation are needed. Keep Turnstone's Knot/Mere source identities
aligned. These pinned consumers remain adoption gates; they have not been
repinned by this phase.

Validation boundary: the clean `4fbcb727` baseline already fails its native
all-features check on missing `DocumentA11yNode::description` fields in
Reader and the web-host test fixture, and four linked-data test accesses to
removed `Node.properties`. The compiler census borrowed only the two Reader
field initializers already present in the concurrent primary WIP so it could
reach downstream consumers. That prerequisite is outside the phase C patch.
Mere's implementation is `7926d3a8`, rebased onto published `a31b9a14` rather
than publishing the unrelated local `4fbcb727` commit. That upstream base
already includes the Reader fields, so the final gates need no overlay.

Final automated receipts on the rebased code:

- Native workspace: `cargo check --workspace --all-targets --all-features
  --exclude mere-linked-data --exclude cambium-genet-web-host --locked
  --offline` passes. The pre-rebase unfiltered run reported only the already-recorded
  web-host test initializer and four linked-data test errors. These exclusions
  are explicit baseline limitations, not a claim that the full workspace is green.
- `cargo test -p insigne -p personae -p notochord --all-features --locked
  --offline`: **233 passed**, zero failed or ignored, including the pre-move
  signature fixtures and doctests.
- Signalman's separate workspace: every target and feature checks, and its
  tests pass **22**, zero failed or ignored. Its generated lock is retained
  with the raw receipts.
- Browser: `cargo check --manifest-path ports/graphshell/web/Cargo.toml
  --target wasm32-unknown-unknown --all-targets --all-features --offline`
  passes with `--cfg getrandom_backend="wasm_js"` in the target rustflags.
  This is a compiler receipt, not a headed browser run. Its generated lock
  was refreshed after the upstream rebase and retained with the receipts.
- `scripts/cargo_mode.py verify --metadata-only` passes; the lock resolves
  one workspace Insigne, including Knot's new direct dependency.
- `ports/distillery/probe/remote-fixture` still fails resolution, including
  online: it requires `mere-p2panda-net = "=0.7.4"`, while the registry
  offers only 0.7.2 and 0.7.1. Its root patch-table gap predates this phase.
- A final tracked-source scan finds no old proof import paths. Review of all
  **82 changed Rust files** found only imports, proof-qualified paths and
  documentation changed; the proof implementation and test logic are unchanged.

Raw command logs, source fingerprints and the two generated nested locks are
retained at `C:\t\cargo-targets\mere\insigne-phase-c-receipts`. The final
tracked source fingerprint stayed unchanged during the final compiler and
proof-test gates. The phase's isolated build output and worktree are removed
once the changes reach origin/main; the primary checkout's concurrent WIP is
left untouched.

Next: phase D. The sibling repins above remain explicit downstream work.

### Phase D (2026-09-29)

Gaz's `RootKey::proof` and `AttestedKey::proof` now hold `Option<KeyProof>`.
The variants retain a `DerivedKeyAttestation` with its exact salt, a boxed
`SignedDelegationCertificate` (whose signing context is already present), or
caller-owned evidence bytes with their format identifier and `ProofMethod`.
The display scope is independent of the signed salt. Opaque evidence provides
storage for future PLC intake; it does not implement a PLC checker.

Typed artifacts must name the recorded key and root on insertion and load.
A rotation must name the immediately preceding root. Both mutation methods
take `Option<KeyProof>` and return `Result<bool, _>`; mismatches leave the
record unchanged, and replay preserves the original evidence. These are
structural rules only. Stored evidence never creates a checked conclusion or
establishes current authority: signature checks, delegation chains, expiry,
revocation, and any interpretation of a capability grant as an identity
binding remain the caller's responsibility. Gaz's production graph remains
crypto-free; Personae issuing and Insigne verification are test dependencies.

Validation on Rust 1.98.1, recorded under
`C:\t\cargo-targets\mere\gaz-receipts`:

- `cargo test -p gaz --all-features --locked --offline -j 4`: **61 tests and
  one doctest pass**, including seven new retained-proof tests. Both codecs
  reload root and attested artifacts, which then check again. Changed salts
  and corrupted signatures still fail their checks after reload.
- Temporarily removing both typed root comparisons makes the reload-refusal
  test fail with an incorrectly accepted record (exit 101). The comparisons
  were restored, and the full test suite passed again.
- `cargo clippy -p gaz --all-targets --all-features --no-deps --locked
  --offline -j 4 -- -D warnings`: **pass**. The earlier dependency-inclusive
  run found seven existing `redundant_slicing` warnings in Personae's
  passphrase/seal code on this toolchain; those files remain outside this slice.
- `cargo check -p gaz --lib --all-features --target wasm32-unknown-unknown
  --locked --offline -j 4 --message-format=json`: **pass**. Compiler artifacts
  show Insigne with no features and no Personae, dalek or BLAKE3 in this
  production graph.

There is no Gaz dependency in Gazette yet. Retinue's Signalman desktop uses
Gaz at the older Mere pin `d82afa17` and still imports the pre-M0.5
`ContactKey`; its broader model repin is separate downstream work. The current
Mere tree has no production calls to the changed key mutation methods.

Phase D meets its done-condition. Gaz M1's storage gate is implemented
2026-09-29 in the founding plan: persona-scoped Muniment save/load and
JSON/postcard disk reopening. Host sealing landed 2026-09-30 through
Castellan and Pandect, with durable-byte concealment, authentication refusal,
and retained-proof rechecking. JSContact exchange remains the M1 gate before
the remaining M2 resolver intake and trust/alarm work. Sibling phase-C repins
remain open. The scoped `C:\t\cargo-targets\mere\gaz` build output is removed
after recording its gates; receipts are retained.
