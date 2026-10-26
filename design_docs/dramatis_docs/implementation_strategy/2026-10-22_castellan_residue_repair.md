# Castellan public-summary residue repair

Status (2026-10-26): on main. The structural repair passes `vault_residue` (every scenario clean, including passphrase storage after profile switching and the passphrase vault after lock) and `no_residue`, plus 306 keeper tests and the golden snapshots, on current main. The 2026-10-22 cut additionally passed optimized heap-canary checks and 236 Djinn tests. Real consumers are still gated.

## Confirmed cause

The failed 88-byte and 176-byte allocations were respectively one and two shared `SlotSummary` entries in the public profile retained by `IdentityVault`. Diagnostic labels matched the exact vector addresses. Both master-seed canary matches began at byte 32. On the measured x86_64 macOS build, the optional instance field begins at byte 24 and occupies 24 bytes: the 16-byte match was inside the inactive payload of `None`.

The old constructor built a temporary summary with an inline `Option<String>` and pushed it into a vector. The inactive representation could carry stale stack bytes into retained metadata. One vector was freed uncleared after profile switching; the other survived vault lock. This traces an internal heap-residue path, not an external key-recovery exploit.

## Rejected first candidate

Materializing initialized strings before clearing missing instances in place passed the original checks and normal regressions. The expanded optimized check rejected it: an ordinary clone of the locked public snapshot acquired a 16-byte passphrase-KEK canary in the same inactive field. Constructor discipline alone did not protect the shared type's derived Clone. Its source and all results, including the failure, are retained under `candidate-v1` in the repair evidence. It is not the final repair.

## Structural repair and API impact

The vault now retains `RetainedPublicProfile`, `RetainedSlotSummary` and `RetainedSlotKey`. Optional instance names use `Option<Box<String>>`; the absent value occupies a fully defined null pointer, without dormant inline string words. This applies to ordinary construction, moves and derived Clone. The guarantee for a sized boxed value comes from [Rust's documented Option representation](https://doc.rust-lang.org/std/option/index.html#representation). No layout offsets or unsafe byte rewriting are used by production code.

`IdentityVault::public_profile()` now returns `&RetainedPublicProfile`. This is an intentional Rust API change for in-memory custody retention. All known consumers are checked by the Castellan, custodian-core and Djinn suites. Shared personae/Dramatis types, the C ABI, IPC views, vault records, encryption and derivation identities remain unchanged. A retained view is not a new storage or transport format.

The constructor preserves canonical sort order, None versus Some(empty) versus named instances, tiers and lineage. Tests cover replacement, removal, profile switching, locked metadata, cold reopen, derived identity continuity and byte-identical encrypted records. A layout guard requires the retained optional instance to remain one pointer wide.

## Evidence boundaries

The scanner retains its original 16-byte window, arming/disarming intervals and positive controls. Added diagnostics report allocation identity, byte offset and bounded allocation labels. Expanded coverage keeps a cloned locked snapshot live and frees another clone while armed. The original byte-identical instrument was separately restored and passed against the structural repair.

The optimized configuration sets only `profile.test.package.castellan.opt-level=3`. Dependencies retain their normal test configuration. Report it as an optimized package test, not a full release/LTO or platform qualification. The scanner measures synthetic heap canaries; stack copies and OS allocations remain outside it.

The original passing 49/49 macOS package, protected installation, public export, frozen library, source and receipts remain unchanged. A 264-file before/after inventory pins them. The historical receipt qualifies its frozen binary, not this changed source automatically. No fixture is replaced or re-signed, and no real vault or consumer is migrated. The new repair receipt separately records final source, tests, failures and preservation checks.

Final evidence, 2026-10-22: all 264 pinned files of the old fixture (source, library, export and receipts) matched their before-state hashes. All 62 protected installation and payload files kept root ownership and stayed non-writable, and strict bundle signature verification passed. The modified source was not substituted into that preserved native fixture. Those receipts were retired with the residue-fix worktree when the repair landed on main on 2026-10-26. On main the residue tests are `cargo test -p castellan --features keeper --test vault_residue --test no_residue`.
