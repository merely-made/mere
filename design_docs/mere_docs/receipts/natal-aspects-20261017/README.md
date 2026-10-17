# Natal aspects resident-validator intake, 2026-10-17

Mere receiving baseline `8c299283f2145d38069f100fbedb25a54b00b746`, branch
`codex/personal-facts-20261016`, consumes the published portable Cleromancy core
`3fa5a6db87cfa34df9fc1de0a0877181c9b4526f` through the canonical `branch = "main"` dependency.
The source is locally committed as `8c8e048ea520053d32ff788d64cb7d0816dbff08`.
Receipt documentation is recorded in a separate local commit. Parent owns publication; no push was made.

The [owning resident test](../../../../ports/djinn/src/resident_mere.rs) creates an explicit
`AspectRules::default()` reading, admits the product's captured transaction through
Turnstone, then reads the identical record through Knot. It submits forged aspect
output and changed rules with stale results from Knot. Both variants have recomputed
content seals and matching canonical node URLs. The resident must reject each with
`Natal aspects differ from their retained rules and chart`.

Before either forgery, the fixture retains separate full session graph resources for
each app. After each refusal, each complete app envelope must equal its own baseline,
with no field stripping or normalization. Both graph revisions stay unchanged, and all
durable session files, including `changes.jsonl` and manifests, stay byte-identical.
The runtime authority remains the product's portable validator already installed by
`MereRoutes::with_divination_authority`; this intake adds its owning regression test.

| Check | Passed | Failed | Ignored |
| --- | ---: | ---: | ---: |
| Focused natal owner tests (overlaps library) | 4 | 0 | 0 |
| Post-source-commit natal owner tests (overlaps library) | 4 | 0 | 0 |
| Djinn full library | 217 | 0 | 0 |
| Embedded reservoir, two processes | 4 | 0 | 1 |
| Embedded reservoir validation | 8 | 0 | 1 |
| Personal facts, fresh owner | 1 | 0 | 1 |

Both focused four-test natal runs overlap the full library suite; do not add them to the
broader gate total. The full gate passed 230 tests with 3 helper tests ignored.
The [post-commit run](natal-committed.log) and its [process receipt](natal-committed.json)
confirm four passing natal tests from the source commit. Its two input hashes exactly
match the broader gate inputs recorded before that commit. Locked offline metadata also passed and resolves the published Git
core at the exact required revision. [Qualification](qualification.json) records actual
process exits, commands, environment, input hashes, per-suite counts, and log hashes.
The [resolved core](core-resolution.json) records its cache manifest and source identity.

All checks use explicit rustc/rustdoc 1.97.1, Cargo 1.99.0, `TMPDIR=/tmp`, and
`CARGO_BUILD_JOBS=2`. From the receiving root, the Cargo arguments were:

```sh
cargo test --locked --offline -p djinn --lib natal_ -- --nocapture
cargo test --locked --offline -p djinn --lib \
  --test embedded_reservoir_two_process \
  --test embedded_reservoir_validation --test personal_facts -- --nocapture
cargo metadata --locked --offline --format-version 1 \
  --filter-platform aarch64-apple-darwin
```

[Cargo's final lock diff](Cargo.lock.diff) changes only Cleromancy core from
`da495cfc75b2ee6bbb9476c8c03386e9057c36d3` to `3fa5a6db87cfa34df9fc1de0a0877181c9b4526f`.
Genet `7f13b85f53c48fa90f433a6cd2ad45cd744e8b02` and all other supplier and registry
entries remain identical to the receiving baseline. No manifest or local patch was
changed. [Scope verification](scope-verification.json) binds the exact source hashes and
confirms preservation of all recorded manifests, configuration, and old receipt files.

The initial online update was terminated while fetching Genet. Isolated offline
resolution exposed cache misses for the existing netrender and Bole pins; exact
published commits were fetched without repinning. A second successful Cargo update
removed incidental Windows dependency-edge rewrites from the first successful pass,
leaving the one-line core change. The lock was never hand-edited.

The first focused run failed only because the new test compared Turnstone's full
session envelope against Knot's baseline, including their different `application`
fields. The fix captures each app's own baseline; all replay, identity, revision, and
durable byte guards remain intact. The original failure log is preserved at
`/Users/markik/Code/testing/cleromancy/aspects-release-20261017/mere-intake/natal-owner.log`,
SHA-256 `9ebf87bbcc02782c4b216b063567fae9ba9f47a9ddecd625fbf519f26559b140`.
The passing rerun is [natal-owner-02.log](natal-owner-02.log), and the complete
broader gate is [owner-gates.log](owner-gates.log). Earlier update failures, fetch logs,
cache preparation scripts, metadata, and process receipts remain under that same
external intake directory.

This receipt qualifies software resident admission and the existing embedded/personal
owner gates. Native UI acceptance belongs to the separately coordinated Cleromancy
run. Earlier receipts, including the historical 225-test result, retain their original
source identities. The primary Mere checkout was not used or modified.
