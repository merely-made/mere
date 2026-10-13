# Exact inventory and admitted retained-record ingress

This local composition connects the live four-object Retinue producer to
Stickleback's exact inventory policy and Mere's existing mesh admission owner.
The producer retains opaque signed records, publishes current metadata, and
returns actual RaptorQ decoder output. An admitted exact snapshot from the
receiver selects missing whole-record digests. The sender rechecks the live
generation, reconstructs its original bytes, and submits those unchanged bytes
over real QUIC.

## Run

Build the Retinue persistent peer from its repository root:

```sh
cargo build --manifest-path testing/transport-conformance/Cargo.toml --locked --offline
```

From Mere, supply that actual executable and a new output directory:

```sh
python3 crates/mesh/mesh/examples/inventory_ingress/run.py \
  ../retinue/testing/transport-conformance/target/debug/transport-conformance \
  /tmp/exact-inventory-ingress-new
```

The runner builds the Cargo-reported current example with `admitted-ingress`,
runs two independent composition processes, and records source, lock, toolchain,
executable and report hashes. Semantic delivery/admission outcomes must agree;
timing and transport actor observations may differ. Each campaign has a
120-second deadline. Persistent producer IPC uses at most 64 commands, 64 KiB
per line, 4 MiB in each direction, and a 60-second child lifetime; failed or
uncertain commands kill the child on drop. Fixture keys and record contents
are public test inputs.

The selected [composition](results/20261012/composition.json),
[independent replay](results/20261012/replay.json),
[provenance](results/20261012/metadata.json) and
[validation](results/20261012/validation.json) bind the captured source.

## Owner and protocol boundaries

`mesh::ingress::RecordIngress` is an opt-in service. Distillery's existing
`ResidentAuthority::enable_record_ingress(config)` mounts it on the resident's
existing endpoint and clones that host's existing `MeshStore`. The owner supplies
its closed/open network policy, trusted roots, service rules, revocations, clock,
call deadline and availability lifetime. It allocates no second endpoint or
operation store. Policy changes require stopping and remounting the service.

The dedicated ALPN is `mere/mesh-retained-admitted/v1`. Its Notochord service
path is `/services/retained-records` in domain `mere.mesh`. The namespace is
bound by both the admitted network and the request. `inventory` and `submit`
are separate actions: a submit-only grant cannot read inventory, and an
inventory-only grant cannot submit an operation. Carrier subject and original
operation author may differ.

Both actions share two active calls and four pending/queued carrier reservations.
The transport reserves queue capacity before waiting for a first bidirectional
stream, retaining that reservation until service dequeue. Full queues close
new connections immediately. A deadline starts at carrier handler arrival and
covers queue wait, session admission, request framing and clean FIN, decoding,
existing owner admission, result framing and response FIN. Active capacity stays
held through that entire call. Each call additionally caps actual reads at
24 KiB and writes at 8 KiB, including partial/refused/timed-out traffic. Resident
shutdown closes the queue, cancels and joins active service calls, then closes
the shared endpoint and existing owner/storage lifecycle.

After the bounded session hello/reply, inventory carries one framed `MIR1` plus
32-byte namespace request with clean FIN. The receiver returns one complete
`MIV1` exact page, capped at 1 KiB. Four objects need no continuation. The sender
authenticates/admit-checks before decoding and placing the summary in
`PeerInventories`; bounded `offers` skips IDs already available at that receiver.
Inventories describe available whole-record digests, not signed operation IDs.
Sender logical slots and receiver owner milliseconds remain separate clocks;
this digest-only exchange does not compare expiry timestamps across them.

Submission carries the expected 32-byte BLAKE3 record digest, then one framed
opaque original record capped at 16 KiB with clean FIN. The receiver verifies
that digest, strictly decodes CBOR and requires exact re-encoding, freshly
authenticates the signed operation through `decode_operation_record::<MeshExt>`,
and calls the same `MeshStore::accept_outcome` used by the existing domain.
Only that owner decides insertion, duplication, payload hydration or refusal.
The result is bounded at 2 KiB and binds record digest plus the operation ID
when fresh authenticated decoding succeeded. A missing result leaves commitment
ambiguous; retry the identical signed original to use the owner's duplicate
behavior. No remote receipt becomes a second local admission marker.

Retinue refuses a fifth live generation without evicting its four originals.
The receiver's volatile availability window retains four accepted originals,
up to 64 KiB, and FIFO-evicts the oldest digest when a fifth accepted original
arrives. Owner expiry removes original bytes and exact inventory together.
Neither cache eviction nor expiry deletes durable mesh operations. Restart
starts an empty availability window; later accepted or duplicate originals can
repopulate it. Inventory presence is advisory availability, not durable custody.

## Captured acceptance and limits

The campaign first inserts one original. It then queries the admitted receiver's
exact snapshot and selects exactly the three missing producer generations.
All four recoveries use actual fresh repair-only RaptorQ decoder output, with
no cached-original fallback, before the opaque QUIC handoff. It compares every
original record, encoded signed header, body and operation ID with the existing
owner's stored operation. Closing all store handles and reopening in a separate
process repeats those signed-byte and four-job checks.

The run also exercises producer capacity refusal without mutation, duplicate
submission, both action mismatches, damaged body, damaged signature, wrong mesh,
oversized declaration, truncation and trailing bytes. Framing failures never
call the operation owner. Six stalled raw handshakes occupy two active calls and
four pending sessions; a seventh is refused. Arrival deadlines drain that work,
and a healthy duplicate succeeds afterward. The separate Distillery test mounts
on its existing endpoint/store, checks expiry without durable deletion, rejects
sub-millisecond retention settings before mounting, shuts down with a stalled
call, and confirms durable state plus an empty restarted availability window.

Mesh's 124 library tests, Stickleback's 99 library tests and the resident
acceptance test pass. The first full transport run passes 56 tests with one
existing ignored child fixture. A repeated full run passes 55 but encounters
`pair 5: the link stopped delivering` in the existing simultaneous-dial gossip
test; its isolated retry passes. Both full outcomes and the retry are retained
in the validation receipt. The new ingress campaign passes in two independent
processes. Scoped Clippy reports no warnings in the new ingress/example source;
pre-existing dependency and transport documentation warnings remain.

This qualifies a bounded host composition with actual QUIC, Notochord and
RaptorQ. Coded-frame delivery is a host repair-only loopback inside Retinue
before the recovered original crosses QUIC. It does not qualify physical RF,
MCU memory/power, wall-clock radio delivery, worker execution or native UI.
Dial tickets are narrowed to direct loopback routes; the endpoints retain their
production listening-socket behavior. The report requires an observed active
loopback route. The earlier finite lossy-link/partition/restart comparison remains
a [separate policy experiment](../../../../stickleback/examples/inventory_compare/README.md).
No upstream MemoryLAN submission, PR or workflow is created by this work.
