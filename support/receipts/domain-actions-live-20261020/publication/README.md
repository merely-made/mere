# Main integration qualification

The source in `qualification.json` integrates the live Mere handler checkpoint
`c2db56559`, canonical recall/replay changes, co-op relay and shared-shell work,
and the current H4 command adapters. The only content conflict was two adjacent
MereShared field additions; both the handler registry and relay configuration
are retained with their existing defaults.

All 94 focused tests pass on this source (39 native, 31 resident Mere, 24 owner
settings), with locked/offline Rust 1.99.1, debug information and incremental
compilation disabled and four jobs. The native host, resident and helper were
rebuilt independently and frozen outside Git before the final native run.
Source-formatting, whitespace and dependency-boundary checks pass.

The final native run uses these exact binaries and three fresh native processes
and three resident starts. One native Open in browser activation receives exact
owner acceptance, produces one actual default-browser visit and one durable
access-history entry. Cold native/resident reopen produces no new visit.
Disabling the owner setting removes the current offer without changing history.
All four presented captures were inspected. Original and earlier integration
receipts remain unchanged; intermediate checks, frozen binaries and the complete
relay-stage native run remain in the external delivery output directory.

Compiler-declared repository inputs and the lockfile are hashed separately.
The final Git integration may add receipts and documentation; those do not
change the qualified native inputs. macOS/local-route/retained-control scope
and all original runtime qualification limits remain explicit in the receipt.
