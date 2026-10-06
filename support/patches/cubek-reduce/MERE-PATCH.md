# Mere cubek-reduce patch

Source: crates.io `cubek-reduce 0.3.0-pre.4`, upstream Cubek commit
`648520b6fd8bfe18bedeb03efe61063dc1efafc5`.
License: MIT OR Apache-2.0. The two license files are retained from the previous
vendor copy because the published crate omits them.

The floating-point extrema identities in `src/components/instructions/extrema.rs`
materialize infinity bits through a mutable runtime local. Browser WGSL
validation otherwise folds a literal bitcast and rejects non-finite `f32`
before dispatch. Infinity and NaN semantics are preserved.

Rebased on 2026-09-27 from the preserved pre.3 delta at `610a32c5`: exactly one
helper and two call sites apply unchanged to pristine pre.4. The normalized
manifest additionally carries an empty workspace for direct upstream tests.
This source rebase does not prove the new IR still emits the required runtime
value. All four headed browser extrema cases remain required by the migration
plan. Remove this patch once a released Cubek row passes them without it.
