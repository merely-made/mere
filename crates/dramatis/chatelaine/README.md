# chatelaine

**chatelaine** is the item taxonomy for the secret half of the Mere
platform's credential model: plain data that describes what is kept, with no
secrets in it.

Named for the waist-worn chain that held the household's keys, and by
extension the keeper of them. The chatelaine is what must never be shown:
passwords, 2FA seeds, tokens, foreign key material. Those secrets are damaged
by disclosure, so they are exercised (filled, generated, released through the
gate), never presented. This crate describes them without containing them,
which is why any view may show what it holds.

The boundaries are the point: not the proofs (that is
[insigne](https://crates.io/crates/insigne): public-key artifacts made to be
shown; the boundary is cryptographic, not filing), not the keeper (that is
[castellan](https://crates.io/crates/castellan)), and not the storage
substrate (that is [personae](https://crates.io/crates/personae)'s vault; the
chatelaine is the item taxonomy kept there).

Lives in the [mere](https://github.com/merely-made/mere) workspace under
`crates/dramatis/`. Built: the item taxonomy (items, credentials of every CXF
v1.0 kind, collections, links, the import disposition per kind, and the OTP
display enums), plain serde data depending only on serde and uuid.

Ruled 2026-10-01: chatelaine becomes a plain taxonomy, like insigne's core.
It holds CXF-shaped item kinds and only their identifying metadata (ids,
titles, each item's scope of sites and apps, and per kind what tells two
items apart), with no secret bytes, no storage and no cryptography. Castellan
keeps the sealed store, which carries the persona it belongs to, and
exercises the items. The CXF import
policy for all 17 credential types, including which ones are quarantined for
the user's review, is in the dramatis tier architecture in mere's
`design_docs`.

## License

MPL-2.0
