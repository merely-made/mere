# Family promotion checkpoint, 2026-10-19

Canonical scope and outcomes:
`design_docs/mere_docs/implementation_strategy/2026-10-19_family_promotion_plan.md`.

These are bounded migration receipts and one-time helpers, not runtime libraries
or a second dependency policy. The actual consumer manifests own their build
configuration. The checkpoint verifiers deliberately compare the recorded donor
and supplier revisions; later intentional source changes need a new checkpoint.

- `source-inventory.json`, `source-equivalence-final.json` and
  `donor-retirement.json` record the unchanged donor and receiving Rust sources.
- `supplier-revisions.json` names the tested, immutable consumer cohort.
  Later supplier documentation-only commits do not change that cohort.
- `consumer-source-validation.json` records successful resolution, locked
  revalidation and one canonical source per promoted package in 21 roots.
- `verification-summary.json` distinguishes executed tests from compile checks.
  Failed preparation and test receipts remain beside their passing follow-ups.
- `header-baseline-comparison.json` records unchanged repository-wide notice
  debt; it does not label the full notice gate successful.

Raw consumer manifests, old consumer locks and the unpublished-work preservation
record stay ignored locally. Public summaries retain their hashes and published
revision coordinates without copying private consumer source configurations.
The original donor workspace and lock were already public Mere source.

All Cargo gates reused `C:\t\cargo-targets\mere`. The total retained receipt
budget for this migration is 100 MB. Model weights, headed captures and physical
device trials are outside this source-home checkpoint.
