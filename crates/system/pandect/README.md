# pandect

Everything a Mere session gathers under one cover.

A pandect is a complete digest of a body of law — the whole corpus collected
into one volume. This crate is that for a running session: the record it
accretes while it is alive.

- Session manifests, graph and view sidecars, worker declarations.
- Graph image-reference adapters, facet and manifest stores.
- Application, device, and persona settings.
- The persona wallet carry lane: the device roster, signed device grants,
  epoch history, and the family-shared identity root.

Eidetic owns portable intake, retained content and image bytes, content caching,
and schema validation. Pandect binds those records to the session and graph,
including graph codicils and retention decisions. Muniment supplies the backend.
Published schema helper names remain as forwarding reexports for consumers such
as Knot; their implementation and schema identity are owned by Eidetic.

Renamed from `session-runtime` on 2026-08-15 when it was published: the crate
was named for what it did in one host, and the family names published crates
for what they are.

Note for anyone building on the wallet half: the carry *model* lives in
[personae](https://crates.io/crates/personae) as `personae::carry`. What stays
here is the adapter: the on-disk layout under the data root, the unlock ladder,
sealed-record wiring, grant envelopes, and the epoch seal. That split was ruled
deliberate on 2026-08-10, so this is a home rather than a way-station.

`WalletSealedBackend<B>` wraps the host's selected Muniment backend with the
persona's `WalletEpochSealer`. It seals each slot value and binds the logical
key inside the authenticated payload. Its envelope contains an epoch marker
and a digest salted inside the ciphertext. Keys remain visible for listing.
Write batches seal fully before reaching the backend; read/write transactions
return `NotTransactional`. The host supplies historical epochs and any rollback
protection. Plain or damaged values are refused, without rewriting the record.

Part of the [mere](https://github.com/merely-made/mere) workspace.

Written with AI assistance (Claude).

## License

MPL-2.0
