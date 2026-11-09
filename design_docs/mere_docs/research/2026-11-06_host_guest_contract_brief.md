# Host/guest contract brief: one recursive contract under Mere, Graphshell and the apps

**Status:** 2026-11-06: brief, unreviewed. Nothing is proposed to code; no ruling is recorded. Written
for Mark's question of the same day, and cited by
[cycle_breaks_plan](../implementation_strategy/2026-11-06_cycle_breaks_plan.md) D4/P6, paused on it.

**Question (Mark, 2026-11-06).** Two options: (1) split the projection contracts into Graphshell,
unarchive the Graphshell repository to sit on top and consume projections from apps, with Mere
underneath as the source of Cambium and the hosting contract; (2) make hosting and projection the same
thing, with Mere the underlying layer. And: "what's the standards-based approach … for embeddable and
embedded host applications? Applications that can essentially exist within other applications or hold
other applications within themselves". He added: "I'm not sure that this will resolve all the issues
here, but I'm willing to give it a shot."

**Proposal examined.** Draw the line between contract and implementation, not between hosting and
projection. One recursive host/guest contract at the bottom (Chirograph wire, the WIT worlds, the
projection and scene types), depending on nothing app-level. An app declares roles: guest (can be shown,
projects), host (shows others, lends capabilities), or both. Mere is the hosting runtime and depends only
on the contract; Graphshell is one host application above Mere; Knot, Cleromancy, Signalman, Woodshed
and Turnstone are guests (or hosts) depending only on the contract.

## (a) Standards survey

Short, from memory of the specifications, not re-fetched today. Anything marked *(unverified)* should be
checked before it carries weight.

| System | What it standardises | Relevance |
|---|---|---|
| **Wayland** (nested compositors) | One protocol; a compositor is a server to its clients and, on another connection, a client of its parent (Weston's and sway's nested backends, Xwayland). Capabilities arrive as versioned globals in `wl_registry`. | Roles per connection, not layers. The closest match to "host, guest or both" over one wire. |
| **WebAssembly Component Model / WIT** | Typed interfaces grouped into worlds (imports and exports); components compose, and one component's exports satisfy another's imports. Capability is what is imported, nothing ambient. WASI 0.2 is stable; async-capable 0.3 *(status in November 2026 unverified)*. | Already the family's idiom: `crates/script/wit/world.wit` says "Unimported means unreachable". The standard answer to recursion plus capability passing. |
| **Web iframes** (HTML Living Standard) | `sandbox`, CSP, Permissions Policy, `postMessage` and `MessageChannel`, origin checks. | The embedded-UI baseline: sandboxed guest, message bridge, host decides grants. Carrier-level, no typed contract. |
| **MCP app embedding** (the MCP Apps extension: `ui://` resources rendered in a sandboxed iframe with a JSON-RPC bridge to the host) | Sandboxed UI plus a standard host bridge. *(Name, number and status unverified; a draft as far as I know.)* | The web converging on guest UI plus a typed bridge. Same shape as `graphshell-mcp`, which already exposes projections as MCP resources. |
| **OpenDoc / OLE** | Parts embedded in shared documents. | Counterexample, already in the [composition brief §5](../../2026-08-12_family_composition_thesis_brief.md): died of contract breadth and heavy shared runtime. Keep the contract narrow. |

Two lessons, neither a standard: roles are properties of a connection, not of a repository; and the
contract has to stay smaller than the runtime that implements it. The Sandstorm and Plan 9 rows of the
same §5 say the same thing from the other side.

## (b) Mapping today's crates

The repository already has the vocabulary. Doctrine Stack 3 (`design_docs/doctrine.md`) names producer,
host and runner, "Graphshell its reference" host, and says "the producer's graph core is contract tier,
published like scene crates; Mere consumes producers, never the reverse". The proposal makes that a
dependency rule.

**Contract candidates, clean today.**

- `graphshell/crates/chirograph`: `hest`, `sceno`, `scenotime`, `serde`, `serde_json`, `blake3`, `sha2`, `base64`
  (`graphshell/crates/chirograph/Cargo.toml`). No Mere runtime crate.
- `crates/cambium/scenes/sceno` (serde only), `scenotime` (sceno), `crates/cambium/hest` (serde),
  `crates/forme/curation` (serde). `scenograph` adds `mere-curation` and optional `backstep`,
  `script-engine-api`.
- The WIT: `crates/script/wit/world.wit` and a byte-identical copy at
  `crates/graphshell/graphshell-endpoint/wit/world.wit` (437 lines each, `diff` clean today). One home
  removes a drift hazard. Package name `mere:script@0.1.0` is a wire name. The `projection-session`
  interface and the `projection-provider` and `app-core` worlds are the contract; `document-core` and
  `document-host` (Genet DOM scripting) and `settings` are runtime or domain material and stay out.
- `crates/script/wire` and `witgen` (serde, ipc-channel, wit-parser): framing and codegen, clean.
- `graphshell-endpoint`'s default surface (`ProjectionSource` and kin) depends only on `chirograph`;
  `graphshell-client` on `chirograph`, `hest`, `sceno`, `scenotime`. These are the host-role kit,
  contract-adjacent.

**Blockers inside otherwise-contract crates.**

- `graphshell-endpoint` features `door`, `profile`, `projection-door`, `projection-provider` pull
  `dramatis`, `ostiary`, `castellan-client`, `pandect`, `insigne`, `personae`. The source homes map
  (`design_docs/mere_docs/technical_architecture/2026-10-28_source_homes_map.md`, "Graphshell names three
  things") already calls the `door` module a custody and admission client, not projection. The guest-facing
  wire (`wire`, `frame`, hello, `AppId`, grants) would move down; custody and identity stay out.
- `scenomise` (the compiler) depends on `scenograph`; its `authoring` feature pulls `backstep` and
  `workbench` from `crates/cambium`. It is an implementation, not a wire. But Signalman's
  `portable-scenes` feature uses `scenomise::backdrops` (`murm/apps/signalman/src/survey/scene.rs:8`, read
  from the local clone), so a vocabulary-only contract leaves that feature one crate short (F1).

**Runtime (Mere).** `ports/mere-resident` (doors, reservoir), `graphshell-host` (admission, WebRTC door,
fixtures; its `native` feature pulls `pandect`, `transport`, `ostiary`), `crates/cambium/*` as the GUI
hosting runtime, `pandect`, Djinn. The map counts `chirograph` in 23 Djinn files (2026-10-27); that heavy
use inside the runtime is the evidence against option 1, since it would make Mere depend upward on
whichever repository owns projection.

**Host app.** `ports/graphshell` (93.6k lines in `src`, about 9.1k of them adapters for other owners, per
the map). The old repository exists archived at `github.com/mark-ik/graphshell` ("Archived 2026-07-23:
moved into merely-made/mere", checked today); `merely-made/graphshell` does not exist. Reviving it means
unarchiving and transferring that one, or creating a new one.

**Guests.** Knot (`knot-editor`); Cleromancy (`cleromancy/core` names `mere`, `graphshell-client`,
`pandect`, `chirograph`, `sceno`, `seiche`, `mere-facts`); Woodshed (`mere-curation`, `pictograph`,
`sceno`, `scenograph`); Signalman; Turnstone (`mere-curation`, `athanor`, `alembic`, `tabard`,
`cambium-desktop`, `cambium-rootstock`, all `mere.git`). "Depend only on the contract" is a target none of
them meets: they reach Mere for data and toolkit crates, not just projection.

**Not contract: `mere-comms`** (`crates/shell/comms`). A domain model (conversations, messages) with
optional Murm and Misfin adapters. D4 sends it to Murm; the proposal does not change that and it should
not be pulled into the contract.

## (c) How each cycle resolves

- **D1, Mere↔Knot and Mere↔Cleromancy.** Mostly unchanged. The break is the resident ceasing to name
  them, through the Stickleback domain interface (domain contract C5), a data contract in Eidetic and not
  the host/guest contract. The proposal adds one thing: the part of P1's seam about presenting
  (projection registration) rides the contract's `projection-provider`. The rest (Knot's `peers` sync
  host, relation intake, the divination validator) is in-process Rust and stays on Stickleback. P1 gets a
  sentence; P2 and P3 are unchanged.
- **D4, Mere↔Murm, and F1.** Where it helps. Signalman's edges are `mere-comms` (D4: to Murm, independent
  of this proposal) and `sceno`/`scenomise`. If `sceno`, `scenotime` and `scenograph` live in a contract
  home that depends on nothing, the `sceno` edge is legal. `scenomise::backdrops` is the residue: its
  needed slice becomes contract-tier helpers, or `portable-scenes` stays behind the separate
  `signalman-desktop` root. That gives F1 a rule, not an answer. P6 stays paused until Mark chooses.
- **D5, Mere↔Woodshed.** Direction only. With Graphshell a separate host above Mere, Woodshed's adapters
  (`ports/graphshell/src/block_editing/bars.rs`, `src/web_tree/blocks.rs`) have a non-Mere landlord. P7's
  seam (a Graphshell-side trait, not a Mere type moved down) is still needed, and Woodshed's Mere
  dependencies are data, not hosting. P7 unchanged.
- **D2 (Eidetic↔ESP) and D3 (Gemot↔Murm).** Not hosting; unchanged.
- **P0, the check.** D6 ("Count them") applies: dev edges count. Under the proposal a guest's dev
  dependency on the real runtime is fine (the runtime does not depend on the guest).

Phases changed: P1 (one sentence), P6 (paused), F1 (a rule offered), P0 and F2 (D6 settles the check's
dev-edge rule; the Gemot to Distillery edge itself stays open).

## (d) What it does not resolve

1. **Shared graph and domain data.** The contract covers presentation and capability, not the graph
   kernel. Stack 3 says the producer's graph core is "contract tier" and that publication "is still
   owed"; until then Knot, Cleromancy and Woodshed link `pandect`, `mere-curation`, `mere-facts` and
   kernel types from `mere.git`, and Cleromancy also names `mere`. This is the largest remaining up-edge,
   and "guests depend only on the contract" is false until it is done.
2. **Turnstone's own engine integration.** Its manifest takes a dozen `mere.git` crates. Rendering and
   engine coupling are not host/guest. Not examined beyond the manifest.
3. **Eidetic↔ESP (C6) and Gemot↔Murm (C5).** Adapter and address-grammar cycles; D2 and D3 stand.
4. **Release cadence.** A new bottom repository is a new supplier tier in the family repin loop (supplier
   first, at most one unused patch per root). Versioned wire helps; the lockstep Stack 3 admits does not
   disappear.
5. **Tests needing both sides.** With D6, a guest's test against the real runtime is fine (guest above
   runtime); a runtime test against a real guest is not. The contract should ship a conformance fixture
   (`LocalCarrier` and `wit-testkit` already exist in `graphshell-endpoint`); full integration tests live
   in the host app or `Code/testing/mere`.
6. **The door's custody half** (Dramatis, Castellan and Pandect types in `door`), and the WIT's
   `document-core` and `settings` worlds: left where they are.
7. **Trust.** The contract names roles and capabilities; who may grant what is Dramatis and Castellan
   (Stack 2), not here.

## (e) Options for Mark

- **A. Adopt as proposed.** New bottom home for contract crates; Mere = host runtime; Graphshell revived as
  a host above Mere. Costs: moving about ten crates and merging two WIT copies; splitting the `door`
  module; extracting Graphshell from Mere (93.6k lines, still tangled with Woodshed, Knot and Moot
  adapters); a repin tier. Wins: F1 gets a rule; the C4 shape cannot recur.
- **B. Option 1 as stated** (projection contracts in Graphshell, on top). Re-creates a cycle. Apps produce
  projections, so they depend on the repository that owns the contract; the runtime (Djinn, resident,
  `graphshell-host`) already uses Chirograph heavily, so it depends up on it too; if that is the top
  repository, the arrow points up. Not recommended.
- **C. Option 2, unified in Mere, no extraction.** Guests depend on Mere (the legal direction). Cheapest.
  Does not clear Murm (Signalman needs scene crates; F1 stays open), and every guest stays locked to
  Mere's cadence.
- **D. Hybrid: extract the leaves only.** Move or publish `sceno`, `scenotime`, `hest`, `chirograph` and
  the single WIT; leave Graphshell in Mere until a second host is planned (Stack 1: code leaves a port
  "once a second host is planned", in a dated plan). The smallest step that settles F1 and keeps A open.

My ordering, a reading and not a ruling: D to unblock P6 now; A if Graphshell as a separate host is wanted
for its own reasons; B no; C is honest if the cycle is accepted.

## (f) Where the contract should live

- **A new repository** depending on nothing in the family. The only home the repository-level check sees as
  legally below Murm, Mere and every guest. Doctrine: Stack 1 makes it earn its place by consumers named
  today (Signalman, Knot, Cleromancy, Woodshed, Murm: nameable); DOC_POLICY §4 puts its docs there, and
  Mere cites them by path.
- **Bottom of Mere.** Cheapest and wrong at repository level: Mere depends on Murm, which would depend
  on Mere. It works only with crates.io publication (consumers take registry versions, which the check
  ignores), at the price of the source-identity seam (`[patch.crates-io]` rows) that the
  [Cambium publish plan](../implementation_strategy/2026-11-05_cambium_publish_plan.md) constrains ("one
  source per graph").
- **Revived Graphshell repository.** Wrong layer for the contract (see B); right layer for the host app.
- **Constraints that bear:** Stack 3 (contract tier, published, "Mere consumes producers"); Stack 1 (a
  component is a module, so the contract may be one crate with modules, not ten); Stack 7 (600-line
  ceiling on moved Rust sources); DOC_POLICY §2 (the WIT lives once).

## Open questions for Mark

1. Does the host role extend to non-GUI hosts (Djinn, the resident)? This brief assumes yes.
2. `mere:script@0.1.0`: rename on the move, or keep as a wire name, as the receipt schema
   `mesquite.commands/v1` was kept?
3. `scenomise::backdrops`: contract helper, or Signalman keeps its separate root?
4. Which repository name, and does Graphshell revive now or after the contract exists?
