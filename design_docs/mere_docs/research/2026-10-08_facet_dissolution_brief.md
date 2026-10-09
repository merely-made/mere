# Facet dissolution brief

**Date:** 2026-10-08
**Status:** open: waiting for graph semantics' owner, to whom Mark hands it.
**For:** graph semantics' owner (the Codex agent on the ThinkPad, per the Scenograph editor plan's SE74), handed over by Mark.
**From:** the Scenograph editor lane. Rulings quoted are in [the Scenograph editor plan](../implementation_strategy/2026-10-07_scenograph_editor_plan.md), SE71, SE72 and SE75.

## What Mark ruled

The word *facet* goes to the dataviz sense: a grid of swatches along axes with a scale rule per axis (SE71). Mark: "the faceted classification we were doing came from pmest and that's not what this is now, so the dataviz facet has stronger provenance." Node facets therefore dissolve. Mark (SE75): "attribute, resource, tag... dissolving into those three sounds good to me", with *attribute* as "an abstraction belonging to or characteristic of an entity", which he reads as view state. The rename waits on this design ("redesign first, rename what's left").

## What node facets carry today

The store is `chartulary::FacetStore<Uuid>` behind graph-kernel's `node_facets.rs` ("JSON-shaped and unknown-forward at its boundary"), set and removed through the delta spine (`apply.rs`), saved by pandect as `facets.json`. In mere's Rust, *facet* appears 3,166 times in 141 files. The declared keys sort four ways:

| Kind | Keys | Proposed home |
|---|---|---|
| Claims about what a thing is | `semantic.classifications`, `semantic.properties`, `chartulary.class`, `provenance.import`, `provenance.derivations` | resource: assertions |
| Labels | `presentation.tags` (tag order and icons) | tag |
| How a node was met or is viewed | `arrangement.pin`, `arrangement.frame-layout`, `arrangement.split-offer-suppressed`, `visit.history`, `graphshell.access-history/v1`, `graphshell.browser-history/v1`, `graphshell.pinned-projection/v1` | attribute |
| Whole app documents at a node | `graphshell.saved-scene/v4`, `graphshell.projection-definition/v1`, `graphshell.content/v1`, `graphshell.local-file/v1`, `graphshell.transfer-offer/v1`, `graphshell.transfer-content/v1`, `receipt.run`, `receipt.artifacts`, scenograph's `explained_relationships`, `authored_order` and `occurrence_labels`, `denizen.binding`, `personae.vault-root/v1` | open, see question 1 |

`facet_projection.rs` (the "PMEST facet projection", a derived queryable map) is the classification sense Mark retired.

Other uses of the word that the rename meets: Gazette's contact facets (`ports/gazette/src/ledger.rs`, source adapter `gazette.contact-facet/v1`), a contact's own parts rather than node facets; servitor capabilities scoped to a facet key (`Cap::facet`, `crates/servitor/src/cap.rs`); forme's `SubgraphKind::Facet`.

## Questions for the design

1. Do whole app documents become resources (content the node shows), attributes, or something else? The Scenograph lane's reading is resources; it is not ruled.
2. What does an expanded tag system carry: plain labels, or key=value pairs (OpenStreetMap's tags, a term of art, are strings only)?
3. How do stored `facets.json` and every `*/vN` key migrate, with the old names still readable?
4. What happens to `facet_projection.rs` and the faceted filter surface built on it?
5. Which of graph semantics' rulings (assertions, resources, the two node senses of rulings 18 and 19) does this touch?

Each comes back to Mark as a question before code moves. The Scenograph lane's swatch grid uses *facet* in new code meanwhile (SE75).
