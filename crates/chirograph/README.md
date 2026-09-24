# chirograph

The projection contract, executed in duplicate.

A chirograph was a deed written twice on a single sheet, with a word inked
across the middle and the sheet cut through it — so the two halves proved each
other by matching. That is a wire protocol: two parties holding the same
agreement, each able to check the other's copy against its own.

This crate is the versioned, carrier-neutral vocabulary by which an endpoint
offers a scene, a resource, or an action, and a host asks for one. Transport,
authorization, application models, and rendered content all stay outside it.

- Projection sessions, requests, offers, and snapshots.
- Resource requests and chunked responses, addressed by content hash.
- Advertised actions and typed intent invocations.
- Presentation manifests over Scenograph's product-free scene types.

The card vocabulary an endpoint uses to describe a resource lives in the
`titulus` module and is re-exported at the crate root. It was the separate
`titulus` crate from 2026-08-14 until 2026-09-23, when it folded back:
nothing but chirograph used it.

Renamed from `graphshell-protocol` 2026-08-14: the contract belongs to the
family, not to the portal that first defined it.

Part of the [mere](https://github.com/merely-made/mere) workspace.

Written with AI assistance (Claude).

## License

MPL-2.0
