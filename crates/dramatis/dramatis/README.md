# dramatis

Name reservation for **dramatis**, the cast-list tier of the Mere platform.

*Dramatis personae*: the persons of the drama. The tier holds both sides of
identity, your faces and the other players, which is why the name is the full
cast list rather than any one role:

- **[personae](https://crates.io/crates/personae)** — the trust-plane spine:
  master keypair, per-protocol derivation, vault, sealed records, carry.
- **[gaz](https://crates.io/crates/gaz)** — stored contacts: anchored
  records, petnames, per-endpoint trust, kith/kin tiers.
- **[gazette](https://crates.io/crates/gazette)** — handle resolution: turning a name into reachable,
  unverified address claims.

The boundaries are the point:

- **Not the data plane.** Persistence is the eidetic family (muniment journals,
  chartulary). The planes bond at the seal seam and the sync gate; dramatis
  holds keys and trust, never the bytes they seal.
- **Not a product.** *Persona* is an in-product term for a face; dramatis names
  the tier so the term stays free.

The identity/contact crates and this reservation live in the
[mere](https://github.com/merely-made/mere) workspace under `crates/dramatis/`;
Gazette is the directory port at `ports/gazette`. Gaz owns contact intake rules,
and Gazette adapts resolver output into those rules. Castellan supplies custody
and sealed backend composition; Personae supplies identity and carry.
If a facade over them ever earns its existence, it lives here. No
implementation yet.

## License

MPL-2.0
