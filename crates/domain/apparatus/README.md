# apparatus

Shared diagnostics home for [mere](https://crates.io/crates/mere).
Package `mere-apparatus`, library `apparatus`. The September 29 ruling assigns
bounded observations and inspection to this crate; the current implementation
provides a renderer-independent bounded record store and retains the earlier
peripheral system-inspector skeleton behind its default `projection` feature.

See the [diagnostics plan](../../../design_docs/mere_docs/implementation_strategy/2026-06-08_system_diagnostics_and_accessibility_plan.md)
for the accepted ownership boundaries and pending two-consumer qualification.

## API

| Item | Role |
| --- | --- |
| `ObservationStore<P>` | Retains already redacted product payloads under count, accounted byte and age bounds. |
| `ObservationMetadata`, `Envelope`, `RecordRef` | Optional producer-supplied causal/subject/revision/frame links, scoped run/source sequences and distinct source/receipt times. |
| `RetentionLimits` | Product-configured count, byte and age limits; any zero disables retention. |
| `Cursor`, `Batch<P>`, `Gap`, `RunBoundary` | Independent non-destructive readers, copied batches, explicit unavailable ranges and reset boundaries. |
| `StoreStats`, `LossSummary` | Retained range/bytes and cumulative per-run rejected, evicted, expired and known dropped counts. |
| `project_skeleton() -> uxtree::UxTree` | Emits the v0 skeleton subtree when `projection` is enabled. Takes no input. |
| `VERSION`, `STAGE` | Crate version string and lifecycle marker (`"pre-alpha"`). |

## Storage example

```rust
use apparatus::{ObservationStore, ObservationMetadata, RetentionLimits};
use std::time::Duration;

let mut store = ObservationStore::new("unique-run".into(), "desktop".into(),
    RetentionLimits {
        max_records: 128,
        max_bytes: 256 * 1024,
        max_age: Duration::from_secs(300),
    });
let mut inspector = store.cursor();
let mut receipt = store.cursor();
let payload = "redacted product observation";
store.record(payload, payload.len(), ObservationMetadata::default(), Duration::ZERO)?;
let visible = store.read(&mut inspector, Duration::ZERO, 64)?;
let evidence = store.read(&mut receipt, Duration::ZERO, 64)?;
assert_eq!(visible.records, evidence.records);
# Ok::<(), apparatus::StoreError>(())
```

The example limits are product policy choices, not crate defaults. Supply host
monotonic elapsed time on every record, read, expiry or limit update. A backward
time fails without changing store/cursor state. Age equality expires a record.
`stats()` observes the last enforced state; `expire(now)` refreshes it without
reading. `tail_cursor()` explicitly starts after existing records. `read` with a
zero record limit leaves the cursor untouched; positive limits never skip a
retained successor. Batches copy payloads and remain valid after store eviction.

Rejected and known dropped observations consume sequence positions, so cursors
see unavailable ranges even when retention is disabled. `note_dropped` accepts
only actual known ingress loss. Missing instrumentation is not counted loss.
Counters saturate at `u64::MAX`. `reset` returns prior run statistics, clears
retention/counters and starts the new monotonic clock; use a unique run identity
that was never used before. Existing readers receive an explicit run boundary.
Resetting with the immediately previous run identity is rejected; Apparatus
does not retain an unbounded identity history to check all earlier runs.

Products must redact and bound payload construction before recording. The payload
byte argument declares the length of the product's chosen encoded payload; generic
`P` prevents Apparatus from independently validating this number. The store adds
deterministic envelope accounting for every ID, option and time field, and rejects
oversized records before admission. This is an accounted encoded-byte bound,
not an allocator-memory bound or a JSON export-size bound. `map_payload` preserves
original admission accounting when adapting a batch to a receipt payload.

No action dispatcher, terminal-outcome matcher, ingress queue, exporter or durable
storage is installed. Products retain operation and persistence authority.

## Skeleton node shape

```text
apparatus (Role::Group, label "Apparatus")
  ├─ tracing events               (Role::Group, empty)
  ├─ register-diagnostics channels (Role::Group, empty)
  ├─ uxtree                       (Role::Group, empty)
  └─ accesskit                    (Role::Group, empty)
```

Node ids come from `uxtree::node_id_for_path`: `apparatus` for the root,
`apparatus/section/{label}` for each section. Ids are stable across runs.

## Dependencies

The storage core uses only `std`. Disable default features for core-only consumers.
`projection` enables `accesskit`, `uxtree`, `tracing` and `project_skeleton`;
it is enabled by default to preserve the earlier API. Optional `serde` enables
envelope, metadata, payload batch, statistics and cursor serialization; opaque
cursors cannot be deserialized/restored into another store.

## Status

Pre-1.0. Core storage has focused contract tests. Two product consumers and
correlated native receipt qualification remain separate gates. Skeleton sections
are empty placeholders; a working diagnostics interface is not implied.
