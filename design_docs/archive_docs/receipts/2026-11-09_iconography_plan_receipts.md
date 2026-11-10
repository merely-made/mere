# Iconography plan receipts

## 2026-11-10 · F1 blazon source

Mere baseline `28e2350a2`. Guarded Rust 1.99.1
`test --locked --offline -p pictograph --no-default-features --lib -j 4`
passed 26 tests in lane `pictograph-grammar`. The four F1 tests cover 2,304
distinct generated text round trips, the plan's paste example, typed
malformed-text failures and structured serde round trips with slot bounds.
The existing `cells5` digest fixtures passed.

Local evidence lives in `Code/output/pictograph-grammar/2026-11-10-f1-test.log`
and `Code/output/build-storage-receipts/2026-11-10/build-c138529fe1d94ce0a537282a71328f78.json`.
