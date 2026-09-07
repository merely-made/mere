# D2 source-document remediation receipt

**Date:** 2026-09-06

**Base:** Mere `0b47ff25f800b17791d78ceccb4f015784d24e0c`

**Scope:** the 15 non-`none` recommendations in `batch_15_cambium.md` and
`batch_16_other.md`, applied across 14 source documents.

The two batch files remain the immutable judgment record for their audited
snapshots. Their stale, contradiction, and unverifiable counts are not
rewritten after remediation.

## Reconciled recommendations

Six Cambium recommendations were applied:

- marked the Serval-host vocabulary as extraction history and named the
  current Cambium/Genet boundary;
- marked both Chisel proposals historical and linked the current Sprigging and
  component-catalog authorities;
- replaced the obsolete Workbench push gate with Turnstone's external
  pin/adoption gate;
- scoped the host-zoom status to committed Mere work and dated Isometry
  evidence;
- separated the dated compatibility tables from the current Mere-owned
  Cambium and document-lane boundary.

Nine other recommendations were applied:

- updated the web-surface assessment, autodiff trainer, FLORA, native smolweb,
  FlipCarrier, and compatibility-view statuses;
- corrected the Distillery projection-walk and physics-catalog summaries in
  `design_docs/DOC_README.md`;
- replaced current `smolweb-views` ownership prose with
  `mere-document-lanes::SmolwebDocument`, while retaining the dated source
  citation as history.

## Residual evidence boundary

The 11 claims classified as `unverifiable` by the two batches were not turned
into completion claims. They still require the headed, physical, publication,
or sibling-repository evidence named by the source audit.

## Verification

Run from the repository root:

```text
python scripts/mere_doc_judgment_audit.py --json
python scripts/mere_doc_audit.py --self-test
python scripts/mere_doc_audit.py --fail-on-findings --json
git diff --check
```

The remediation tree retained 298/298 D2 active-document coverage with no
coverage errors. D3 retained zero failing index, link, known-root path, plan
status, and historical-annotation buckets; its planted-defect self-test also
passed.
