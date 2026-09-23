# gaz

The contact layer: your records about other people.

A contact is the local rollup that says these addresses and keys are all the
same person: your petname for them, filed under an anchor that never moves,
with handles and endpoints that each carry their own trust state. Records are
persona-scoped (a throwaway persona must not share the work persona's
contacts), tiered kith / kin, and carry recency.

```rust
let mut book = ContactBook::new(PersonaScope::new("work"));
let alice_key = TypedKey::ed25519(alice_bytes);

book.insert(
    Contact::new("Alice", alice_key)
        .with_handle(Handle::acct("acct:Alice@example.org"))
        .with_endpoint(Endpoint::new(EndpointKind::Misfin, "alice@example.org")),
);

book.mark_contacted(&Anchor::Key(alice_key), now_ms);
assert_eq!(book.recent(5)[0].petname, "Alice");
```

## What a record is filed under

An **anchor**, never a name. Handles change and hosts move; an anchor does
not. It is whatever the peer's own system holds fixed:

- **a root key**, typed by its family (Ed25519, secp256k1, P-256, or a whole
  Reticulum identity). For a personae peer, that is the master key its
  attestations name, not whichever derived key arrived first.
- **a `did:plc`**, for an atproto account, whose signing keys belong to its
  server and change on every move.
- **a local id**, a `urn:uuid`, for someone who has shown you no key yet.

A person's keys come in two shapes. The **root line** holds the keys that have
stood for them as a whole, each succeeding the last, so a rotation never moves
the record and a message signed with a retired key still finds its owner.
**Attested keys** are concurrent: one per protocol and one per device, each
vouched for by a root.

Every key and anchor is written in its own family's standard text form:
`did:key` for the multicodec families, `rnid`'s hex for a Reticulum identity,
`did:plc` and `urn:uuid` for the rest.

## The boundaries are the point

- **Not the resolver.** A gazetteer turns a name, handle, or key into
  reachable endpoints. gaz is where the ones you keep live. The two are
  siblings on the persona tier, and gaz is not short for gazetteer.
- **Not identity.** `personae` owns *me*, the key-bag and its carry. gaz owns
  *them*, your own records about other people's keys.
- **Not trust arithmetic.** Trust state is stored per endpoint; how it is
  earned belongs to the trust plane. gaz depends on no cryptography, holding
  keys as bytes it compares but never verifies.

## Two habits

gaz never reads a clock or a random source: every timestamp is a `now_ms` you
pass in, and a local id is minted from bytes you supply, which keeps it
deterministic under test and usable on wasm. And every recency update is
monotonic, so a replayed or late event can never rewind a record.

## Status

Pre-1.0. The data model exists and is tested; persistence over `muniment` and
the adapters that turn resolver output into records are the next lifts. See
`design_docs/` for the founding plan.

## License

MPL-2.0 (see LICENSE).
