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

Both kinds of key retain an optional `KeyProof`, rather than only a method
label: an Insigne `DerivedKeyAttestation` with its exact salt, a
`SignedDelegationCertificate`, or caller-owned evidence bytes with a format
identifier and `ProofMethod` (for example, a PLC operation). The display
scope is separate from an attestation's signed salt. Typed artifacts must name
the recorded key and its root on insertion and reload; a rotation names the
immediately preceding root. Opaque evidence is interpreted by its caller.

`rotate_to` and `attest` take `Option<KeyProof>` and return `Result<bool, _>`.
A mismatch leaves the contact unchanged; replaying a known key preserves its
original evidence. The caller must check authenticity again after reload and
apply current authority, expiry and revocation policy. A delegation certificate
proves a capability grant, so its use as an identity binding also needs the
caller's policy. Gaz stores the artifact, never a `Checked*` conclusion.

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

Pre-1.0. The data model retains proof artifacts and is tested with JSON and
postcard reload followed by real signature checks. Persona-scoped persistence
is available behind `muniment`, with host sealing through Castellan/Pandect.
JSContact exchange is available behind `jscontact`. M1 library gates are complete;
M2 resolver intake and application wiring remain in the founding plan.

## Persistence

Enable Gaz's `muniment` feature to use `save_book` and `load_book`. Both take a
host-supplied `muniment::SlotStore<B, C>`, generic over its backend and codec.
The host enables its chosen Muniment features, such as `json`, `postcard` or
`redb`; Gaz's optional dependency selects none of them.

```rust
use gaz::{PersonaScope, load_book, save_book};
use muniment::{JsonCodec, SlotStore};

// The host chooses and opens the backend.
let slots = SlotStore::<_, JsonCodec>::new(backend);
let scope = PersonaScope::new("work");
let book = load_book(&slots, &scope).await?;
save_book(&slots, &book).await?;
```

Books live at `personas/<scope>/contacts`, using the opaque scope label exactly
as supplied. Saving replaces that persona's book. A missing slot loads as an
empty book for the requested persona; malformed data and mis-filed books are
errors, never empty or partial success. `PersistenceError::Scope` carries both
the expected and stored persona names, while `PersistenceError::Store` retains
the backend or codec failure.

The contact model is validated on load, and retained signatures still need the
caller's checks. At-rest sealing belongs to the host's supplied backend. The
JSON/postcard tests use memory and real redb reopen, proving persona isolation,
overwrite behavior, malformed-record refusal and artifact rechecking.

Hosts composing Castellan's `keeper` feature can require sealing with
`PersonaeHost::sealed_backend(persona, backend)` before constructing the
`SlotStore`. It refuses a missing wallet epoch; it never selects cleartext.
Pandect supplies the adapter, so Gaz gains no cryptographic dependency. The
host maps its persona to Gaz's opaque scope and owns historical epoch loading
and rollback policy. The host receipt also checks the entire closed redb file
for cleartext petnames and refuses damaged or transplanted ciphertext.

## JSContact exchange

Enable `jscontact` and construct `JsContactFormat` with a domain controlled by
the host (for example, `JsContactFormat::new("example.org")` in documentation).
Gaz keeps its own Contact model at rest. The format is a bounded RFC 9553
projection, not a complete schema validator or vCard converter.

- `publish(&PublicCard)` builds a persona card from explicitly selected public
  name, identity artifacts, handles and endpoints. Its input cannot carry local
  notes, trust, tier or recency. Publication here means constructing bytes for a
  host to share; Gaz does not fetch or publish on the network.
- `import(&Card, local_id)` returns unverified kith claims and the preserved
  source Card. It ignores private state from peers, performs structural identity
  validation, and leaves cryptographic checking and book updates to the caller.
  Foreign free-text/URI identifiers need a caller-selected local anchor; bare
  UUIDs and Gaz anchors parse directly. A PLC claim needs a root key.
- `export_contact` / `restore_contact` are lossless PRIVATE backup operations.
  Restoration includes notes and trust and must only read the caller's trusted
  backup. Public fields must agree with the validated private record.

`uid` keeps the anchor's standard text, including Reticulum's free-text rnid.
`cryptoKeys` contains root and attested resources; Reticulum carries all 64
public bytes in a data URI. Domain-prefixed `gazIdentity` retains root history,
attested keys and proof artifacts; `gazHandleKind` and `gazEndpointKind` retain
protocol roles. Private `gazLocal` stores Contact JSON as a string so u64
history remains exact. No stored proof becomes a successful signature check.

Parse untrusted bytes with `Card::parse`, which refuses duplicate object names
and invalid mapped shapes without dereferencing URIs. `ImportedCard::source`
retains original map IDs and unmapped fields; hosts must retain it separately
when re-exchanging those fields. Regenerating from Contact alone does not
preserve foreign fields. Generated map IDs stay stable across reordering and
unrelated insertions; duplicate public entries collapse while private backups
preserve original arrays. The optional feature adds JSON and URI validation,
without crypto, storage, network, clock or random-source dependencies.

## License

MPL-2.0 (see LICENSE).
