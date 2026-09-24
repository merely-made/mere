# insigne

**insigne**, the graded identity proof of the Mere platform's dramatis tier.

An insigne is what a persona presents: a proof of identity made to be shown and
surviving showing. The grade is chosen per audience, from a bare key
(continuity) through signed cross-attestations to delegation certificates,
and it needn't be the most stringent proof available. Your insigne is what
someone else's [gaz](https://crates.io/crates/gaz) keeps; a published insigne
is what [gazette](https://crates.io/crates/gazette) resolves.

*Insigne* is the Latin singular of *insignia* — a plural English uses so
exclusively that its singular has dropped out of ordinary use. It names one
badge of office or rank, and rank badges are graded by construction: which one
you wear is chosen for the occasion, which is this crate's whole argument.

The boundaries are the point: not the secrets (that is
[chatelaine](https://crates.io/crates/chatelaine): bearer material, damaged by
disclosure, where everything here is a public-key artifact designed for it),
and not the keeper (that is
[castellan](https://crates.io/crates/castellan)).

The core is plain, serializable data: the artifacts that travel and that a
gaz keeps. Checking belongs behind a feature, and a passing check yields a
local conclusion that is never serialized.

Built today:

- `TypedKey`, the bare-key grade. A public key typed by its family (Ed25519,
  secp256k1, P-256, or a whole Reticulum identity) and written in that
  family's standard text form: `did:key`, or `rnid`'s hex.
- `DerivedKeyAttestation`, a master key's signed word that a derived key is its
  own, and the `delegation` grammar: certificates, revocations and the
  canonical bytes their signatures cover. Both moved here from
  [personae](https://crates.io/crates/personae) with their formats unchanged.
  Issuing stays there, since it needs a persona's keys.

Features: `digest` gives certificate ids and attenuation, on BLAKE3. `verify`
gives signature checks on ed25519-dalek, in the lax mode personae always used,
and implies `digest`.

Lives in the [mere](https://github.com/merely-made/mere) workspace under
`crates/dramatis/`.

## License

MPL-2.0
