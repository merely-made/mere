# pandect

Everything a Mere session gathers under one cover.

A pandect is a complete digest of a body of law — the whole corpus collected
into one volume. This crate is that for a running session: the record it
accretes while it is alive.

- Session manifests, graph and view sidecars, worker declarations.
- Graph image-reference adapters, facet and manifest stores.
- Application and persona settings, with references to host-owned identity.

Eidetic owns portable intake, retained content and image bytes, content caching,
and schema validation. Pandect binds those records to the session and graph,
including graph codicils and retention decisions. Muniment supplies the backend.
Published schema helper names remain as forwarding reexports for consumers such
as Knot; their implementation and schema identity are owned by Eidetic.

Renamed from `session-runtime` on 2026-08-15 when it was published: the crate
was named for what it did in one host, and the family names published crates
for what they are.

In 0.2.0 the wallet store, device settings, grants, epoch sealer and sealed
backend moved to [sporran](https://crates.io/crates/sporran). Callers import
those APIs from Sporran directly. The carry model remains in
[personae](https://crates.io/crates/personae) as `personae::carry`; secret
custody and release policy belong to Castellan and its host.

Part of the [mere](https://github.com/merely-made/mere) workspace.

Written with AI assistance (Claude).

## License

MPL-2.0
