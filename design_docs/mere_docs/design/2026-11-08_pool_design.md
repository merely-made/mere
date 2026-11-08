# The Pool: a swatch you drop things into (design brief)

**Date**: 2026-11-08
**Status**: design brief from a Mark session. Mark recalled "design work on something called a pool, as a clipboard utility", then described it: "something of a sharing utility as like a little card or perhaps swatch that you could drag data into meant to be used as a clipboard". No document in the family carries it (searched every `design_docs/`, `TERMINOLOGY.md`, the naming ledger and memory on 2026-11-08; every *pool* in the tree is a producer, fetcher or gnode pool), so this is its first writing. It states the model, what the tree already has for it, and the forks, ruled the same day in §7. It is not a plan.
**Related**: [swatch primitive design](2026-06-27_swatch_primitive_design.md) (the fourth primitive; §3 the instance config, §11 "field as a scope" as a grab-bag), [nodes and cards](2026-07-01_node_card_summoning_design.md) (the card family and summoning), [subgraph derivation from selection](2026-06-13_subgraph_derivation_from_selection.md) (what a selection becomes), [one state, n windows](2026-07-05_one_state_n_windows_design.md) (view instances over shared truth), the [graph view curation plan](../implementation_strategy/2026-10-28_graph_view_curation_plan.md) (U2's shared drag machine; "share a proven view"), the [engine split plan](../implementation_strategy/2026-11-07_engine_split_plan.md) D27 (backstep's clipboard contract) and the scenograph editor plan's SE63, SE65 and SE83 (scope, the edit layer as intents, `Scope::Selection`).

## 1. What Mark described, in the tree's words

A small surface, card-sized, that accepts drops and holds what was dropped; a person draws from it later, in the same window, another window, another device, or hands it to someone else. The operating system's clipboard is the nearest everyday thing, and the weakest version of it: one slot, invisible, lost on the next copy, never shared.

The tree already has every piece but the drop:

- **A swatch** is "a mere seen at a scope, small enough to embed", and "a portable fact, not a widget" (TERMINOLOGY). Its scope may be a `Selection(Vec<String>)`, "an explicit set of ids, as a person picked them" (`scenograph::Scope`, SE83). A gloss pane shows one; a facet composes several; an embed carries one onto another surface or site.
- **The card family** is "summonable surfaces scoped to a node or selection"; the connections swatch is already "the multi-selection card (selected nodes + their edges as a live swatch)", and the aspiration is "one card system that renders anchored-on-canvas or embedded in the roster / a note / a menu".
- **Field as a scope** (swatch design §11, Mark 2026-06-28): "a convenient grab-bag that need not be connected (so it reads as a Loose / Session shape, not a structural one)". That is the pool's shape exactly, with a drop as the way in instead of a region.
- **Two planes**: truth in the kernel, curation per instance; "hiding is not deleting". A pool is curation: membership in a pool asserts nothing about the graph.
- **The edit layer**: "editing inside a swatch later as interaction intents through host authority" (SE65). A drop is such an intent.
- **Sharing**: "share a proven view" is the curation plan's standing wish (Mark, 2026-08-03); a mere "is born share-ready", and a moot is "what a mere becomes when shared".
- **The clipboard contract** (engine split D27, 2026-11-08): backstep owns a host-implemented text clipboard trait; Cambium's text input is bound; the request-shaped sites (iced, chrome's frame model, the web share) wait on a request/completion pair (F2 of that ruling, held for this brief).

## 2. The model

**A pool is a swatch whose scope is a selection that grows by drops.** Nothing new in kind: `Swatch { scope: Scope::Selection(ids), mode: Projection(recipe) }` with a loose arrangement (a pile, a grid, a shelf) and a drop acceptance the host routes to it. What is new is the gesture and the lifetime: a selection today is what a person has picked right now; a pool is a selection a person *keeps*, and keeps adding to.

The metaphor, judged by the inference it carries rather than by resemblance (More Than Cool Reason): a pool is a container that things flow into and are drawn from. The inferences that survive the crossing are the ones a clipboard lacks:

- A pool **holds many**; a clipboard holds one.
- Drawing from a pool **does not empty it**; a paste that moves is a cut, and that is a separate gesture.
- A pool has a **surface**: what floated in last is on top, what sank is still there. That is a recency order the arrangement can draw.
- A pool **overflows**: a cap, and what the cap sheds, is a visible rule rather than silent loss.
- Pools **can be shared**: several people may dip into one. The clipboard is private by construction.

The inference that does not cross and must not be drawn: a pool mixes its contents. Members keep their identity; a pool is a set of ids, never a blend.

## 3. What a pool holds

Ids only. Members are the things `Scope` already names: nodes, links, subgraphs, nested graphs, whole meres. The pool is a curation over truth, so it never holds bytes.

Foreign data dragged in (text, a URL, a file, an image, a selection from another application) therefore goes through **intake first**: the host asserts a node for it by the route the data already has (fetch for a URL, import for a file, a note node for text), then adds that node's id to the pool. Two consequences:

- The pool is the same object whether its member came from the orrery, a web page, the OS clipboard or another person; provenance lives on the node, where it belongs.
- Dropping into a pool is the one place where "make this a node" is cheap and expected, which is what the OS clipboard never offered: paste into a pool and the thing exists in the graph.

Intake is host authority (SE65): the drop arrives as an intent, the host decides, the kernel records, the pool's selection grows by the resulting id. A refused intake leaves the pool unchanged and says so.

## 4. The clipboard contract under it

backstep's contract (D27) and the pool are two layers, not two answers:

- **The OS clipboard is a host binding**: text in, text out, one slot, through `backstep::Clipboard` where a host has a call-shaped clipboard and through the request/completion pair (D27's F2) where it is event-shaped (iced, the web's gesture-bound promise, chrome's frame model ports).
- **The pool is curation**: ids, many, kept, shareable.
- They meet at two gestures. *Copy to pool* from a text selection is intake (the text becomes a note node) and then membership. *Paste from the OS clipboard into a pool* is the same intake with the clipboard as source. A host may also mirror a pool's top member as the OS clipboard's text face, so an application outside the mere sees the last thing pooled; that is a host option under the configurability rule, not the pool's definition.

So D27's F2 proceeds as ruled; the pool does not replace it or rename it.

## 5. Where it lives, by the split

- **Types**: `Swatch` and `Scope` in scenograph already carry it; a pool needs at most a marker on the swatch (that its selection accepts drops and keeps) or none, if the host treats any selection swatch it hosts as droppable. Fork F1 below.
- **The record**: a pool a person keeps is a view instance over shared truth, so it is pandect's (view keys and curation records) and durable with the session; the one-state-n-windows design already makes such instances visible to every window of one state. Sharing a pool to another person is sharing a view, the curation plan's "share a proven view", over the mere's share-ready substrate. Fork F2.
- **The surface**: Graphshell's host layer. A pool card is a new kind in the card family's summoning slot (`focus_card`), and a gloss pane can show one; Turnstone, as the reference host, is where it is first seen. The drop target joins U2's shared drag machine when that extraction happens; until then it is the host's.
- **Engine side**: nothing. Mere's engine sees asserted nodes and nothing called a pool.
- **A crate**: none. The name is a product term (TERMINOLOGY candidate), not a crate; `pool` on crates.io is a stranger's in any case (`pool` 0.1.4).

## 6. Forks (Mark's)

- **F1. Identity.** A pool is a swatch with a kept, droppable `Selection` scope (recommended: no new type, SE83's scope does it); or a new `Scope::Pool` variant so a pool is distinguishable from a transient selection in records and recipes; or a crystallized subgraph (the selection-derivation design's end state), which makes membership a graph fact and contradicts §2's "curation, not truth".
- **F2. Lifetime and reach.** Durable with the session in pandect and visible in every window of one state (recommended); or ephemeral per window, which is a clipboard with a face; or a node in the graph whose nested graph is the pool's contents, which makes a pool shareable as truth at the price of asserting membership.
- **F3. Foreign drops.** Intake first, ids only (recommended, §3); or the pool holds opaque payloads until a person promotes them, which means a second store and a second provenance model.
- **F4. Drawing out.** Drawing from a pool copies and the pool keeps (recommended, §2); or a draw removes, which is the clipboard's cut and loses the "holds many" inference.
- **F5. Cap and overflow.** A cap per pool with what it sheds shown (recommended), set by the person; or unbounded, where a pool becomes a second roster.
- **F6. Order of work.** D27's F2 request/completion pair proceeds now and the pool follows as its own plan when greenlit (recommended); or the pool's plan goes first and the request pair is designed as its binding.

## 7. Decisions (Mark, 2026-11-08, one card round of four and one of two)

- **P1. Identity:** "A swatch with a kept, droppable Selection scope (Recommended)". No new type; `Scope::Selection` (SE83) carries it and the host marks which selection swatches accept drops and persist. Outcome: applies the swatch design (§3) and SE83.
- **P2. Lifetime and reach:** "Durable with the session in pandect, seen by every window (Recommended)". A view instance over shared truth; shareable to another person as a view. Outcome: applies one-state-n-windows and the curation plan's "share a proven view".
- **P3. Foreign drops:** "Intake first; the pool holds the new node's id (Recommended)". Provenance lives on the node; a refused intake leaves the pool unchanged. Outcome: applies the two-planes rule (curation, not truth) and SE65.
- **P4. Drawing out:** "Per pool, a person's setting". Each pool declares copy-out or move-out; the default is copy, so the holds-many inference is the default and cut is a choice. Outcome: applies the configurability rule (expose the design space rather than pick one).
- **P5. Cap and overflow:** "A cap per pool, what it sheds shown (Recommended)". Set by the person with a default; the oldest member sinks out visibly. Outcome: new to this brief.
- **P6. Order of work:** "Request pair now; pool as its own plan when greenlit (Recommended)". D27's request/completion pair, the Genet selected-text query and the backstep publish proceed; the pool's plan follows with P1 to P5 as its rulings. Outcome: applies engine split D27.

## 8. When a build is greenlit

A plan under `mere_docs/implementation_strategy/` (or Graphshell's, since the surface is the host's) with done-conditions of this shape: a selection swatch in Turnstone accepts a drop from the orrery and from a web page's text, the text becomes a node through intake and the pool shows it; the pool is visible in a second window of the same state; drawing a member out onto the canvas leaves the pool unchanged; a cap sheds visibly; a share hands the pool to a second person as a view. Receipts headed, in the harness, not synthetic input.
