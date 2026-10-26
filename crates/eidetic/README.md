# eidetic

Eidetic does not live here any more. The family promotion of 2026-10-19
([`2026-10-19_family_promotion_plan.md`](../../design_docs/mere_docs/implementation_strategy/2026-10-19_family_promotion_plan.md),
landed in commit 883ccdc4c) moved its sources to
<https://github.com/merely-made/eidetic>, which owns `muniment`, `chartulary`,
`eidetic` (eidetic-core), `hagiograph`, `stickleback`, `luggage`, `mere-proofs`
and `servitor` as separate crates. Their design documents went with them.

Mere consumes those crates as git dependencies pinned to one eidetic revision
in the workspace `Cargo.toml` (search for `merely-made/eidetic`); the `[patch]`
tables there hold every consumer on the same pin. Change Eidetic in its own
repository, then repin here. Do not add crates under this directory.

What stays in Mere:

- [`crates/intel/eidetic-search`](../intel/eidetic-search): the tantivy trail
  index. It indexes browsing traces and reaches `esp` and `import`, which are
  Mere product concerns; turnstone pins it from `mere.git`.
- [`crates/fleece`](../fleece): Mere's intake wrapper over Genet's `fleece`
  extractor, with the Eidetic bridge in `src/eidetic_bridge.rs`.

This file is the only item left here, kept so that the old path explains
where the code went.
