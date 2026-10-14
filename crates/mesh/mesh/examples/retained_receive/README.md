# Retained signed-object receive prototype

A Mere-owned host experiment connecting Retinue's retained-repair simulator to
an existing domain admission path. It authors one current V2 mesh job post,
recovers its original signed operation record as opaque bytes, and passes the
freshly decoded operation to the production owner before storage changes.
An opt-in QUIC mode carries the recovered record between real p2panda-net/iroh
endpoints before receiver-side admission. The prototype and its inputs may
change; this is not a conformance gate.

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

For the real carrier handoff, use the same built Retinue peer and add the
explicit carrier selection:

```sh
cargo run --locked --offline -p mere-mesh --example retained_receive -- \
  --retinue-peer ../retinue/testing/transport-conformance/target/debug/transport-conformance \
  --carrier quic --output quic-handoff-exploration.json
```

This mode uses the production `transport::P2pandaTransport` with two public
fixture identities, a dedicated `mere/retained-receive/v1` ALPN, and explicit
peer tickets narrowed to direct loopback routes. Relay, mDNS, random-walk
discovery, gossip and blob serving are not enabled. The endpoints retain the
production bind behavior; narrowing the dial routes does not restrict their
listening sockets to loopback. The report records the authenticated incoming
peer and actual active loopback path separately from the operation's author.

## Bounded real carrier handoff

The data path is:

```text
Retinue decoder output → bounded opaque object → real QUIC stream
  → complete object + clean FIN → fresh operation decode → existing mesh owner
  → bounded acknowledgement on the reverse stream
```

Stickleback's `read_object` / `write_object` leaf operates on generic Tokio
streams. One object is a four-byte big-endian length, its exact opaque bytes,
then clean end of the sending half. The reader checks the declared size before
allocation and requires EOF after the payload. A deadline covers the whole
read or write, including FIN/EOF; trailing bytes, truncation and a stalled peer
cannot produce a completed object. The leaf has no operation decoder or store.
This is an authored object-stream profile over standard QUIC, not a FLUTE,
NORM, native-drop container or Reticulum wire compatibility claim.

The example chooses a 16 KiB object ceiling, a 2 KiB acknowledgement ceiling,
10-second transfer/connect deadlines and one in-flight exchange. Across all
attempts, including refusals, it caps sessions at 12 and request frames at
64 KiB, reserving at most 128 KiB for request plus acknowledgement frames. Each
exchange has a 30-second deadline within a 90-second carrier campaign; endpoint
setup and close have separate bounds. Only a complete
object with the expected protocol and authenticated carrier peer reaches the
existing admission callback. The callback independently chooses the expected
mesh and freshly verifies the original signed record.

The receiver sends an acknowledgement after the owner returns a result. It
binds the actual record digest and, when authenticated decoding succeeds, the
operation ID to that result. The report distinguishes the owner's result from
the sender receiving this acknowledgement. The statement is an advisory result
from the authenticated carrier peer; it does not become another local admission
marker. A missing acknowledgement does not establish whether an operation was
committed. Retrying the same signed object retains its ID and uses the owner's
existing duplicate behavior.

The local run observes oversized declaration, truncated object and stalled
stream refusal before any owner callback, with valid duplicate handoffs
afterward. Application transfer bounds belong to this finite host run; the
existing transport's global accepted-session queue remains a separate service
admission concern.

The actual-path check holds both stream handles and records both endpoints'
route snapshots. The transport API describes these as momentary actor state,
so an empty active set may be observed despite an established connection.
The example permits at most two seconds to observe active routes, further
capped by the exchange and campaign deadlines. Only empty active state is
retried; any active non-loopback route fails immediately. Success records the
initial snapshot, final snapshot, read count and elapsed time. An intermediate
capture failed the earlier undiagnostic route check; its cause could not be
classified. The subsequent run passed on its first snapshots and does not
establish snapshot lag as the earlier failure's cause.

The selected [QUIC handoff report](quic-handoff-exploration-20261011.json)
records actual carrier exchanges, peer/path facts, owner outcomes, received
acknowledgements and the three framing refusals. The captured run completed 11
attempts: five primary object handoffs, three framing refusals and three valid
duplicate handoffs afterward. It accounted for 8,162 attempted request-frame
bytes, 2,018 received acknowledgement payload bytes and 30,734 reserved
request-plus-acknowledgement bytes. The owner retained one operation and one
posted job, including after closing and reopening its redb handle. Both
transport endpoints closed. The earlier simulator-only report remains a
separate historical experiment.

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
`murm/design_docs/2026-10-11_transport_conformance_plan.md` owns implementation
sequencing and later stable-contract evaluation. Publication uses direct
commits/pushes to `main`; local checks support prototyping. No workflows or PRs
are part of this work.
