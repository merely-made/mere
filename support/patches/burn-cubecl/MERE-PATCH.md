# Mere patch provenance

This directory is the published `burn-cubecl 0.22.0-pre.2` crate from Burn
commit `89bcc85f75c55e3451442f5371de45b243865340`.

Mere adds one BrowserWebGpu correctness guard to the numeric, integer, and
float binary launchers. When both logical operands reference the same CubeCL
allocation and view, the published launcher binds that allocation twice as two
kernel inputs. Headed Chromium executes the kernel without a validation error
but returns stale or unmodified storage. Burn LayerNorm reaches this path at
`centered.clone() * centered` and therefore returns its input unchanged.

The patch exposes logical-allocation identity through Mere's existing
`cubecl-runtime 0.11.0-pre.2` patch. A same-allocation binary launch now binds
the allocation once, aliases the second tensor argument to input zero, and
writes to a distinct output. Independent allocations retain Burn's existing
in-place selection.

The headed reproducer is
`ports/distillery/probe/repros/burn_browser_embedding`. Its exact Burn unit
LayerNorm and `8 x 384` BERT-width cases fail on the published row and pass with
this patch, with an empty WebGPU error list. Remove both identity and launcher
changes when a released Burn/CubeCL row passes the same cases unpatched.

## 2026-09-27: preserve broadcast layout for aliased inputs

Ruling 380 corrects the same-allocation branch's RHS view: its alias now uses
the fresh output's reference shape, just as the LHS does. The original alias
view omitted this shape and indexed a broadcast RHS as an ordinary linear input.
`as_linear_view_alias_like` and the three launcher call sites are the production
delta; allocation/view identity and separate-output behavior are unchanged.

Nine direct native WGPU cases cover generic subtraction, float atan2 and integer
XOR with broadcast aliases, equal-shape aliases and separate inputs. Each checks
real identity, scalar values, shape/length, fresh output and preserved input;
float checks also require finite values. Removing only the broadcast mapping
fails the three broadcast cases, while the six controls pass; restoration passes
all nine. Full Seiche passes 96 tests with its existing tolerances and one ignored
test. This native receipt does not rerun or extend the historical headed claim.

The dev-only burn-backend dependency enables cubecl-wgpu/std for those explicitly
ignored GPU unit tests. Detailed command, source/lock, mutation and concurrency
qualification is in the canonical Burn migration plan §13.21 and external
`Code/testing/mere/receipts/2026-09-27/burn-pre4/pre2-repair-checkpoint.json`.
