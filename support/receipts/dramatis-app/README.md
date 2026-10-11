# Dramatis app Windows receipt

**Status (2026-10-11): qualified first Windows slice.** These are synthetic sealed stores, not an actual user vault.

The app selects existing personae, lists public quarantined cards, and asks Djinn to accept/delete an exact item or assign an older import namespace. Djinn owns custody and native consent. Unsupported credentials remain quarantined. The app dependency graph excludes Castellan.

| Gate | Result |
| --- | --- |
| Castellan library | 303 passed |
| Djinn custody wire | 16 passed |
| Resident startup store discovery | 1 passed |
| App state | 4 passed |
| Retained host interaction | 7 passed |
| App wire with absent / Locked / unlocked Djinn | All passed |
| Consent bypass disabled / enabled / restored | Pass / expected semantic failure (101) / 16 passed |
| Real native consent | No preserves 3 cards; Yes deletes only the selected card |
| Native default button | No, verified through the observed process-owned control |
| Production Djinn, portable metadata, dependency bans | Passed |

Five native GPU captures were visually reviewed: absent, Locked and unlocked, with wide and narrow Locked/unlocked windows. They show readable status, public roster, disabled choices while Locked and secret-free review cards. The retained harness uses production views and focus hooks to exercise pointer, Enter, keyboard order, focus reveal, resize, zoom and manual scrolling. The native consent PNGs and `native-consent.json` qualify the separate resident-owned No/Yes dialog. No actual user's dialog was operated.

Tests caught changing a store's sealing keys after persona selection and lock/unlock. Original store keys now remain fixed, with multi-store and startup discovery receipts. UI qualification caught a black root background, disabled controls in Tab order, missing focus reveal and Windows defaulting native consent to Yes. Each is corrected and requalified.

`receipt.json` records exact source hashes, commands, results, executable hashes and artifact hashes. Raw source hashes preserve tested worktree bytes; `source_sha256_lf` permits comparison across Git line-ending conversions. Receipt logs and JSON preserve their exact bytes through local attributes. `window-scope.json`, `control-scope.json`, `native-consent.json` and `client-dependencies.json` retain the bounded later qualification. Only the views, interaction tests and native dialog changed after the full custody run; the aggregate manifest verifies that all other source hashes remain identical. The consent source was restored byte-for-byte, and the synthetic bypass flag is absent from production source.

Reproduce with `python support/dramatis_review_receipts.py --jobs 1`, then `python support/dramatis_native_consent.py`. `--window-only` requalifies views; `--control-only` requalifies the scoped bypass and restores source and the fixture binary. The marker-identified synthetic runtime directories are removed after their owned processes exit. The consent bypass is enabled only in the negative-control test process.

The primary documentation baseline has zero broken relative links or missing current source paths, but still lacks the unrelated producer-gates D2 record and the MPL header on `projection_dynamics_tests.rs`. The worktree audit also reports references that resolve only from the primary checkout. The new app plan produces no findings. Both baselines are retained in the audit logs; they do not establish a clean repository-wide documentation gate.

`preliminary.zip` preserves the original failing sealing-root control, preliminary builds, interrupted resource-heavy checks, earlier window/dialog captures and lane run logs. It is historical evidence, separate from final green logs.

The app lives in Mere's `ports/dramatis`. Start Djinn first, then run `cargo run --locked -p dramatis-app --bin dramatis-ui` from Mere. Portable Dramatis crates remain independent of Mere. Persona creation, contacts, credential fill, CXF export, distribution packaging and Linux/macOS host qualification remain follow-on gates. Builds reuse `C:/t/cargo-targets/mere`; no isolated Cargo home is retained.
