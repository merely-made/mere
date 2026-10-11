# Retained signed-object receive prototype

A Mere-owned host experiment connecting Retinue's retained-repair simulator to
an existing domain admission path. It authors one current V2 mesh job post,
recovers its original signed operation record as opaque bytes, and passes the
freshly decoded operation to the production owner before storage changes.
The prototype and its inputs may change; this is not a conformance gate.

## Run locally

Build Retinue's current peer from its repository root:

```sh
cargo build --manifest-path testing/transport-conformance/Cargo.toml --locked --offline
```

From the Mere repository root, supply that actual executable:

```sh
cargo run --locked --offline -p mere-mesh --example retained_receive -- \
  --retinue-peer ../retinue/testing/transport-conformance/target/debug/transport-conformance \
  --output receive-exploration.json
```

The output path must be new. This example caps its serialized record at 16 KiB,
subprocess requests/responses at 4 MiB, and each child at 60 seconds. It uses a
scratch redb database and public
fixture signing keys, preserving the canonical workspace dependency pins. It
starts the bounded local simulator process; it does not start network endpoints
or talk to other people's nodes. Once dependencies are cached, both commands
run offline.

## The owner boundary

Mere authors a `MeshEvent::JobPostedV2` with its existing wire builder.
Stickleback's `operation_record` preserves signed header bytes and inline body;
its CBOR-encoded `DropRecord::Operation` is the retained object. Retinue treats
that whole object as opaque, performs actual bounded RaptorQ coding/recovery,
and returns the actual decoder output after length/manifest-digest checks.

Mere strictly decodes the record, requires exact re-encoding, and calls the
existing `decode_operation_record::<MeshExt>`. That freshly decodes and
authenticates the original header. The unchanged typed operation then enters
`MeshStore::accept_outcome(expected_mesh, operation)` → existing `MeshPolicy` →
Stickleback `OperationProcessor` → Muniment's indexed write. The expected mesh
comes from the receiving owner, independently of the offered operation.

The p2panda operation ID hashes the signed header. The retained-object digest
covers the whole record. Exact record, encoded header and body comparisons are
needed to demonstrate preservation; operation equality alone compares IDs.
With the pinned p2panda implementation, authentication occurs during header
byte decoding, so this example never mutates a verified typed header.

## What the observations answer

The same original object is collected by named one-radio and four-radio modeled
receivers. Its first accepted arrival inserts one operation and produces one
posted job; another arrival is a duplicate. Reopening the same redb store checks
the retained bytes and board effect. Once-only refers to the durable insertion
and semantic effect, not to transport delivery or the number of policy calls.

Correctly authenticated outer delivery also carries three intentionally invalid
inner inputs: a validly signed operation for another mesh, a changed body, and
a changed signed header. Their refusals occur at the existing codec/processor/
domain owners; operation count and board remain unchanged. Ordinary job posts
have no general signer whitelist in the current own-devices mesh model, and the
prototype does not invent one.

The report records actual byte preservation, operation IDs, receive completion,
owner outcomes/refusals, store counts, board state and source/executable identity.
The selected [2026-10-11 report](receive-exploration-20261011.json) uses a
1,015-byte record and eight source symbols: four radios reconstruct at slot 2;
one radio joining at slot 6 reconstructs at slot 16. Both arrivals recover the
same original bytes and operation ID, with one durable insertion and posted job.
It retains all six owner observations, including the three refusals and
in-process store reopen. Its dirty-base and compiled-source fingerprints name
the exact prototype that ran; it is a design experiment, not a qualification
matrix.

Radio opportunities are modeled slots with parallel transmitter lanes and a
known channel/manifest cadence. This demonstrates host composition, without
establishing live Reticulum/Dash/QUIC carriage, RF throughput or MCU behavior.
The job post is metadata: this slice neither transfers its referenced blobs nor
runs a worker.

Cross-repository findings live once in Mere's
`design_docs/2026-08-24_standards_survey_brief.md` §10. Retinue's
`retinue/design_docs/2026-10-11_transport_conformance_plan.md` owns implementation
sequencing and later stable-contract evaluation. Publication uses direct
commits/pushes to `main`; local checks support prototyping. No workflows or PRs
are part of this work.
