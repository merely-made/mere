# Retired relation decay input

The current semantic relation API no longer accepts an input its writers
ignored. `EdgeAssertion::Semantic` contains the recognized kind and optional
label. Current JSON rejects `decay_progress`, including null. The obsolete
Pandect finite-value check and its sole-purpose error variant are removed.
Assertion strength and lifecycle behavior are unchanged.

The qualified Mere source is `1cbae9acd9b4cb3470d99d29235701ffc25676ec`,
following implementation `20e4a08c4a3c9b5d370cdce6d9245a9f24d1f6e8` and
baseline `904d7391bf75d46c42f6c10698e90a01ab468195`. The later receipt commit
changes documentation and evidence only.

## Stored-history boundary

One private adapter on `CapturedDelta::ReplayAssertRelationByIds.assertion`
reads the frozen historical shape and discards the unused value. Human-readable
output omits the field. Positional postcard and rkyv output keeps the empty
historical slot, preserving the following entries. The historical reader keeps
its existing tolerance for unknown fields; strict rejection belongs to current
command input. Direct standalone `EdgeAssertion` binary archives are outside
this compatibility promise.

The three committed fixtures were produced by the actual unmodified baseline
kernel, before the API edit. They contain nine deltas: semantic None, 0.0, 0.25
and 1.0 values, the other four assertion families, and a following title
sentinel. Tests decode all three codecs into equal cleaned deltas, rewrite the
binary codecs, replay semantic records without splitting the relation, and
apply the sentinel. New JSON omits the key. `legacy-fixtures.json` records byte
lengths and hashes; the raw writer source, Cargo manifest/lock, process record
and log are included. Snapshot `agent_decay_progress` fields and readers,
workspace manifest and standalone web lock are unchanged; their hashes are in
`preserved-readers.json`.

## Owning caller migration

The first Djinn check on clean implementation source failed: pinned Cleromancy
core `3fa5a6db` still constructed the removed field at three sites. That actual
failure is retained in `djinn-check.json/.log`; it is not qualification.

Cleromancy `73dccb6f7db2a49350bf1f15c93983e457351c62` migrates those sites to
the existing scoped semantic writer. It preserves endpoints, kinds, labels,
default scope and author context. Five existing authored-spread/reopen,
fixed-spread and reflection/session tests pass against its existing Mere SDK
`9b8c161a831aa9615c84a4bbfaea9b15021e3659`. This caller commit published before
the Mere removal. Its process record, harness dependency identity and commit
diff are included.

Mere's Cargo-generated root lock then takes that revision. All 1,774 package entries remain; this is the only changed package source identity.
Cargo also reselected 16 dependency edges among already present packages,
including Windows-target `windows-sys` selections and existing Oxigraph-family
`rand` links. `dependency-intake.json` enumerates the complete change; no other
version, Git revision or checksum changed. The original Dramatis pin is kept.
These checks qualify macOS and WebAssembly; no Windows build is claimed.

## Qualification

All accepted gates start from clean committed source and record actual process
exit, log hash and Cargo suite summaries in `qualification.json`:

| Gate | Passed | Ignored |
| --- | ---: | ---: |
| Kernel, all features, including integration/reopen and doctest suites | 481 | 3 |
| Pandect | 359 | 1 |
| Mere, linked-data, cartography, view and Pictograph consumers | 848 | 19 |
| Graphshell web/query library | 320 | 6 |
| **Mere total** | **2,008** | **29** |
| Cleromancy caller regressions on its existing SDK | 5 | 0 |

Djinn all-targets check, product WebGPU WebAssembly check and port boundaries
pass. The final native dependency check includes the migrated Cleromancy core.
This API/history cleanup has no new visual acceptance run or receiving-host
repin claim. Woodshed was notified; Knot checkpoint confidentiality remains
separate.

Existing documentation and judgment audits still report their baseline
findings. Their parsed outputs are unchanged; these are not passing global
gates. `doc-audit-comparison.json` records the comparison. `iterations.json`
indexes preliminary compile/harness failures and the dependency correction;
raw attempts remain under the named external qualification root.

Run with Python 3.11 or newer:

```sh
python3 support/receipts/link-model-retired-decay-20261019/verify.py \
  support/receipts/link-model-retired-decay-20261019
```

The verifier reuses existing link-evidence checks for process exits, clean
source, raw logs and suite totals. It also checks historical fixture hashes,
unchanged readers, the caller SDK identity and the exact generated lock change.
The implementation plan remains the backlog owner:
[link-model plan](../../../design_docs/mere_docs/implementation_strategy/2026-10-15_link_model_plan.md).
