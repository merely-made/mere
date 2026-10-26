# Mere cubek-reduce patch

Source: crates.io `cubek-reduce 0.3.1`, upstream Cubek commit `245851cc83e95e179b91bc145c2b3a1e305bfdba`.
License: MIT OR Apache-2.0. License files retained from the previous vendor copy.

The infinity workaround in `src/components/instructions/extrema.rs` materializes
infinity bits through a mutable runtime local so WGSL validation does not
constant-fold a non-finite float. The carried source also includes NaN-aware
extrema/top-k selection, stable ties and associated CPU reference/tests.
The normalized manifest adds an empty workspace for direct tests. All these
local deltas are preserved alongside the released upstream changes.

Retire the infinity workaround once a released Cubek passes the four headed
browser extrema cases without it. Retiring the other carried deltas requires
their corresponding NaN/top-k/tie regression coverage. A source rebase alone does not establish browser
acceptance.

## 0.22.1 cohort rebase (2026-10-24)

Released archives were checksum-verified before a three-way rebase. Existing
local source and regression deltas are preserved, including changes added
after the original stable-release receipt. New upstream source changes are
retained. Selected execution gates pass in the Burn migration plan, section 13.48.
The bounded macOS receipt and remaining platform/model limits are recorded
there; the source rebase alone is not a wider acceptance claim.

The previous vendor README described an older, smaller patch cut. This
revision corrects that description rather than discarding later local fixes.
