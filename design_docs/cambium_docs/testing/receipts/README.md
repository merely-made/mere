# Component catalog fixtures

These self-contained HTML files are generated from the same retained
`ScriptedDom` and application stylesheet as the executable component catalog:

- `graphshell/host/crates/cambium/cambium/tests/fixtures/component_catalog_narrow.html`: 420 px browser viewport, 22 rem specimen
- `graphshell/host/crates/cambium/cambium/tests/fixtures/component_catalog_regular.html`: 900 px browser viewport, 48 rem specimen

These are executable test inputs under the Cambium crate's `tests/fixtures/`,
relocated unchanged on 2026-11-04. The explicit writer below updates that same
fixture home; it does not write historical run evidence into documentation.

Regenerate them from the workspace root:

```console
cargo run -p cambium --example component_catalog -- --write-receipts
```

The example test compares both committed files byte-for-byte with fresh output.
Open them in a browser for visual review. Browser HTML preserves custom-leaf
geometry and interactive overlays; Sprigging's actual paint commands remain
covered by the fast retained-leaf acceptance assertions in the example.

**Qualification, 2026-11-04:** the relocated example compiles and its component
acceptance test passes. The byte-comparison test fails against the unchanged
goldens. The original source and original fixture paths reproduce the same
one-pass/one-fail result on the same locked stack, with identical actual and
expected HTML hashes. The [comparison record](../../../../../../testing/mere/closure-audit/batch-04-semantic-verification.json) preserves that
limit; this relocation did not regenerate the goldens or claim fresh browser
acceptance.
