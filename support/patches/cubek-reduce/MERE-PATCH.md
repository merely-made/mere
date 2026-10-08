# Mere cubek-reduce patch

Source: crates.io `cubek-reduce 0.3.0`, upstream Cubek commit `b6ac1f2ee54c773f345c0bfbd9f330c9022cc3a8`.
License: MIT OR Apache-2.0. License files retained from the previous vendor copy.

Rebased on 2026-10-06 onto the stable release. The sole source delta is one
helper and two calls in `src/components/instructions/extrema.rs`: materialize
infinity bits through a mutable runtime local so WGSL validation does not
constant-fold a non-finite float. Stable upstream's NaN-propagating intrinsics
and all other release changes are preserved. The normalized manifest adds an
empty workspace for direct tests.

Remove once a released Cubek passes the four headed browser extrema cases
without this workaround. A source rebase alone does not establish browser
acceptance.
