# Crate consolidation plan

**Date:** 2026-09-23

**Status:** in progress, 2026-09-23. C1-C3's clear-cut moves are landed and
pushed. Mark ruled every open question the same day (see Rulings): C4's folds
are in progress in this session, insigne's delegation split is the dramatis
session's, and chatelaine waits on a CXF-shaped taxonomy.

## The ruling this serves

Mark, 2026-09-23, reviewing an audit of the workspace's publishable crates:

- The reserved crates (`chatelaine`, `dramatis`, `insigne`, `mere-alembic`,
  `mere-apparatus`, `mere-athanor`, `mien`) exist to receive their
  capabilities. Capability that grew in a crate not meant for it moves into
  its named home; stubs are not left standing beside the kludge.
- A component is a module, not a crate. Components are folded into their
  parent crate rather than kept, published or given repositories of their
  own. No new crate or repository is proposed for something that is a
  component.
- The `sibylla` and `vates` shims, and their names, are killed.

The same day's version-baseline pass (a one-time bump, then lockstep workspace
versions at the first release) waits on this plan, so the baseline is set on
the consolidated set of crates rather than churned twice.

## Phases

### C1. Shims out

Delete `sibylla` and `vates`, the deprecated re-export shims over
`esp::embed` and `esp::infer`, and every use of their names in live code.

*Done when:* no workspace member, manifest, environment variable or test
instruction names either crate; historical docs cite them as history.

### C2. Named homes filled

For each reserved crate, find where its capability lives and move it in, in
the direction that makes no dependency cycle.

| Home | Capability | Source | State |
|---|---|---|---|
| `mere-athanor` | the furnace passes: forgetting, image cleanup, consolidation, retirement | `pandect::athanor` | landed `1bda73d5` |
| `mere-alembic` | the three memory levels, behind its `recall` feature | `pandect::memory_levels` | landed `1bda73d5` |
| `mien` | standing (event grammar, ledger, persona chains and vault, gate, wire, store) and the composite reputation lens | `gemot::moot::standing`, `moothold::concord` | landed `a1551086` |
| `chatelaine` | the secret-item taxonomy, shaped against CXF's credential kinds | castellan's secret-free OTP item types, once the taxonomy exists | ruled: design first |
| `insigne` | presentable proofs: typed keys, delegation certificates and revocations, derived-key attestations | gaz's `TypedKey`; personae's delegation data types | `TypedKey` landed `0f3f854f`; delegation split ruled, in progress (dramatis session) |
| `tabard` | theme and stylesheet authoring over tinct, illume and CSS | not yet located | named home kept; source to be found |
| `dramatis` | the tier facade | none misplaced | nothing to move |
| `mere-apparatus` | the inspector pane | already its own code | nothing to move |

*Done when:* every row is landed, ruled out by Mark, or found to have
nothing misplaced, and each landed move carries its tests with it.

### C3. Components folded

Fold crates whose only role is a component of one parent, where no
repository outside Mere imports them.

| Fold | Into | State |
|---|---|---|
| nine `register-*` crates | `mere-registry` (`registry`), one feature per registry, `publish = false` | landed `3430ba2b` |
| `mere-trail`, `mere-glossary`, `mere-subgraph`, `mere-roster`, `mere-gloss` | modules of `mere` under the features that re-exported them | landed `61894570` |
| `mere-capability` | `servitor::cap`, replacing servitor's re-export module | landed `5335a869` |
| `eidetic-https-fetcher`, `eidetic-iroh-fetcher` | eidetic features `https-fetcher`, `iroh-fetcher` | landed `254b23b6` |

*Done when:* each fold compiles feature by feature, every moved test passes
in its new home, the portable verify gate passes, and docs citing the old
paths are repaired or annotated as historical.

### C4. Candidates with outside consumers or chosen names

Ruled by Mark 2026-09-23:

| Candidate | Ruling | State |
|---|---|---|
| `titulus` | fold into chirograph | landed `83feb122` |
| `eidetic-fjall` | fold into eidetic as a `fjall` feature (Turnstone changes its imports) | landed `3943874f` |
| `mere-resident`, `mere-mesh-host` | fold both into distillery; djinn already depends on distillery, so djinn would have made a cycle | landed: `distillery::lifecycle`, `distillery::mesh_host` |
| `graphshell-stdio`, `graphshell-local`, `graphshell-network` | fold into `graphshell-endpoint`, one feature per carrier (five repositories change imports) | landed: `graphshell_endpoint::{stdio, local, network}` |
| `mere-canvas`, `mere-signals` | fold into `pictograph`, not the reverse; canvas becomes `pictograph::canvas` and the mere facade keeps exposing it at `mere::canvas`, so consumer paths do not change | to do |
| `scenograph` | stays a crate: its scene editing is useful beyond graphshell | ruled out |
| `tabard` | stays a crate: the named home for theme and stylesheet authoring (C2) | ruled out |

*Done when:* each ruled fold has landed with its tests and gates.

### C5. Version baseline, after consolidation

Mark's rulings: a one-time baseline of genet 0.6 and Mere 0.4 once the
consolidated set settles; genet's forks (`genet-taffy` 0.14.0,
`genet-parley` and `genet-fontique` 0.10.0) keep their upstream-derived
numbers and are listed with their own republish rule; lockstep
`version.workspace` follows at the first release.

### C6. Registry names

`sibylla`, `vates` and `mere-capability` remain on crates.io with no crate
behind them. Deletion is done on the site by Mark and feeds the
workspace-wide crate inventory at the Code root.

## Findings

- 2026-09-23. `mere-athanor` was a 35-line stub while Athanor's 848-line
  forgetting pass sat in `crates/system/pandect/src/athanor.rs` *(historical citation)* <!-- doc-audit: historical-path -->. Its only
  outside user is Turnstone's `src/recycle.rs`. Dependencies now run
  athanor -> pandect -> alembic.
- 2026-09-23. `gemot::moot::standing` imported nothing else from gemot, so it
  moved whole. `RepLens` became generic over the moot key: mien sits below
  gemot and cannot name gemot's `MootId`. gemot's deprecated `tessera`
  module (a source alias for standing) had no user in any repository and is
  deleted; the on-disk `tessera.redb` read path stays, since it serves data.
- 2026-09-23. servitor's `cap` module was a compatibility re-export of
  `mere-capability`, a leaf split out so servitor and gemot shared one
  definition. gemot already depended on servitor, so the leaf separated
  nothing.
- 2026-09-23. The dramatis session (M0.5, gaz) established the C2 limits:
  castellan's sealed OTP store must stay in castellan, because its
  crate-private release path is what makes the gate the only way to mint a
  code, so only secret-free item types could move to chatelaine. insigne's
  code is personae's (`delegation.rs`, `provider.rs`'s
  `DerivedKeyAttestation`), so moving it makes personae depend on insigne,
  and gaz's no-cryptography rule needs insigne's core serde-only with
  verification behind a feature.
- 2026-09-23. Mark ruled on insigne: gaz's `TypedKey` moves into insigne, and
  insigne's core is plain serializable data with verification behind a
  feature. Moving personae's delegation certificates too would reverse a
  recorded ruling: the
  [device grant delegation reconciliation](../technical_architecture/2026-08-11_device_grant_delegation_reconciliation.md)
  keeps the identity proof and attenuation grammar in personae ("This needs
  no change to personae"). This plan's earlier insigne row, which named
  delegation certificates, contradicted that and is corrected. The dramatis
  session is putting the split to Mark: data types in insigne, issuing in
  personae, checking behind insigne's feature.
- 2026-09-23. The first fold tool rewrote sibling references after prefixing
  module paths, so a module named like its old crate produced
  `crate::crate::subgraph`; caught by the compiler in one test file and
  fixed before commit.
- 2026-09-23. The C1-C3 moves broke two relative links and 27 path
  citations across ten docs; `scripts/mere_doc_audit.py` run at
  the pre-move commit and at head isolated them.

## Progress

- 2026-09-23. C1 landed (`d69be378`). esp's model-directory variables became
  `ESP_MINILM_DIR` and `ESP_TINYLLAMA_DIR`, and esp's persistence dropped a
  private newtype that only existed to dodge the orphan rule across the old
  crate split.
- 2026-09-23. C2: athanor and alembic (`1bda73d5`), mien (`a1551086`).
  Tests moved with the code: mien 59, gemot 147 -> 97, moothold 18 -> 9.
- 2026-09-23. C3: registry (`3430ba2b`, 141 tests), mere modules
  (`61894570`, 44 tests), `servitor::cap` (`5335a869`, 8 tests), eidetic
  fetchers (`254b23b6`, 9 tests), all passing. Mere's workspace went from
  124 members to 106; the portable build from 1,549 packages to 1,531.
- 2026-09-23. Docs citing the moved paths repaired or annotated as
  historical; the doc audit reports zero broken relative links. Pushed with
  Mark's go-ahead as part of `254b23b6..03c05dbd`.
- 2026-09-23. C2 insigne ruled; the dramatis session is moving gaz's
  `TypedKey` in. The delegation-certificate question is open.
