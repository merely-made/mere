# The stack's doctrine

What the stack holds and why, by area: short statements, each citing the cases
it rests on. This is the layer read during work; the cases are each plan's
Decisions section here and, for statements that came from the games wing,
isocosm's design record, cited as `isocosm:<n>`. Rules for keeping it:
`DOC_POLICY.md`, core §11. Not to be confused with the [data-oriented doctrine
brief](2026-07-02_data_oriented_doctrine_brief.md), which names a
representational discipline.

**Status, 2026-10-22:** begun with five statements moved from isocosm's
doctrine, the parts of its Stack area that bind any product on the stack
(isocosm's decision-links plan, L6d). This repository's own rulings, kept in
its plans, join in a later pass; the first, Stack 6, joined on 2026-10-25.
Within the budget in `doctrine_budget.toml`.

**Status, 2026-11-07:** the Stack area's budget rose to eight for Stack 9, two roots
by nature (engine-split D22).

**Status, 2026-10-28:** a second area, Surfaces, with four statements from
the [GUI catalog record](mere_docs/design/2026-10-26_gui_catalog_design.md)'s
rulings of 2026-10-26, placed here by Mark on 2026-10-28; the record keeps
the inventory. Its rulings have no typed keys, since the record is not a plan,
so each statement's note names them.

## The stack

### Stack 1. Build on the stack we have

A product builds on the stack it has. A crate it doesn't need or want is not
invited in, and a stale dependency of marginal benefit leaves rather than
being fetched to keep a build whole. A product's own crates are made when the
abstraction earns one, and a component is a module in its parent crate, not a
crate of its own. A crate reserved as a capability's home receives that
capability, which moves there when it is found elsewhere. A seam is reserved
for a consumer that can be named today, never for a speculative one. Code
leaves a port for the stack once a second host is planned, in a dated plan of
the host's own repository with a done-condition, not once one shows up. The
move is done when the code has moved and its first host builds on it; the
second host's adoption is that host's receipt. Try use, configure, compose and
extend before implementing the gap; a new dependency still earns its cost.

Links: draws on isocosm:600, isocosm:organs.5, house-style D1, house-style D2, house-style D12, house-style D14 · summary Build on the stack we have; try use, configure, compose and extend before implementing the gap; dependencies earn their cost, components are modules, and extraction serves planned hosts.

*Refined 2026-11-02 (house-style D14): "Add that reuse order
(Recommended)". D1, D2 and D12 continue to apply.*

### Stack 2. The stack supplies identity and trust

Protected identity and admitted connections come from the stack; a product
maps an authenticated peer to its own participant. A product never
manufactures keys, grants or trust roots to fill a missing stack
configuration.

Links: draws on isocosm:962 · summary Identity and admitted connections come from the stack; a product maps peers to its participants and never manufactures keys or trust roots.

*Moved 2026-10-22 from isocosm's doctrine, its Stack 2, worded for any product
("Reword to 'a product'").*

### Stack 3. Seams and formats

Seams stay clean even at some cost in speed: a missing boundary bites later,
and hardware improves. Renderers swap behind a scene contract. Open formats
come first, drafts and community reports included where we have a use and
they are cheap, each claim labeled by its status; a product makes its own
only where none holds the thing. Applications meet at the contract too. A
producer offers surfaces or datagraphs, widgets and utilities included, and
endpoints a host dials; a host owns a graph and loads producers at runtime,
Graphshell its reference; a runner carries either or both and owns no graph.
An installable app is a frame, a host with producers fixed at build. The
producer's graph core is contract tier, published like scene crates; the
host consumes producers, never the reverse, and graph-owning producers compile
in lockstep with it until published. Each capability names its supplier, who
grants its use and who owns its lifetime. Extension follows four rungs: data
(vocabulary packs hold all recognition: predicates and classes are open names,
each a row of axes the core dispatches on; it observes a finite set of values
per axis, compiled as enums that carry the rest, and an unobserved value is
kept and degrades by the pack's fallback chain, then the axis default; the
predicate enums are owed to a core pack), scripts (authored rules and packs
lowered by digest), components (outside code on Wasmtime where supported),
native crates (first-party cores). By call frequency times data volume,
cold and narrow may be pluggable components; hot or wide stays native.

Links: draws on isocosm:27, isocosm:77, isocosm:78, isocosm:1000, distillery-extraction D8, distillery-extraction D9, distillery-extraction D15, house-style D15, linked-data-platform-lessons D2, linked-data-platform-lessons D4, linked-data-platform-lessons D6, linked-data-platform-lessons D7, linked-data-platform-lessons D31, linked-data-platform-lessons D32, engine-split D1 · replaces stack.5 · summary Open contracts separate producers, hosts and runners; open formats first, useful cheap drafts included and labeled by status; capabilities name supplier, grant and lifetime owner; extension follows data (packs hold all recognition as open names over axes the core observes, compiled as enums that carry the rest), scripts, components and native crates by call cost.

*Refined 2026-11-06 (linked-data-platform-lessons D2, D4, D6, D7, D31,
D32): "1, but check isocosm's efforts, genet's efforts… for consistency and
making it consistent where that helps"; "i actually think we should just
support community drafts if we have a reason to use them. And we clearly do!
If it's cheap."; "Axes closed, names open (Recommended)"; "Same model for
both (Recommended)"; then, on "i don't see why 'closed' is necessary, as
opposed to 'observed' or 'utilized'": "Observed, not closed (Recommended)";
"Both: chain first, core default last". Refined 2026-11-07 (engine-split D1):
"I agree with the overall split"; the host, not Mere, consumes producers.
House-style D15 and distillery-extraction D8, D9 and D15 continue to apply.*

### Stack 4. Platforms and the web

Desktop is first-class; the web is a supported tier with a stated floor. Two
bindings take each platform's best capabilities where that costs nothing on
mobile or the web. Gamepads and keymapping are stack capabilities, genet's and
the standards', not one product's. Web modules run their wasm constructors
once, through cambium-genet-web-host's helper, and take the newest
wasm-bindgen as one exact pin; runners build with the repo's pinned toolchain.

Links: draws on isocosm:21, isocosm:29, isocosm:77, isocosm:532, isocosm:536, isocosm:537, isocosm:556, isocosm:567 · summary Desktop first-class, the web a floored tier; best capabilities where free; gamepads are the stack's; web modules share one helper and pin.

*Moved 2026-10-22 from isocosm's doctrine, its Stack 11, worded for any
product ("Reword to 'a product'"). Cases 532 to 567 are this repository's
burn-migration rulings as the wing recorded them; the burn plan here records
them first.*

### Stack 6. Storage follows the death condition

Data is classified by what makes it dead: a boundary (scratch, task, frame,
device epoch), an event (one entity removed from a living world) or lost
reachability. Storage and ownership follow that and the access pattern, not
allegiance to an allocator. Identity is stable; representations are
projections with their own lifetimes. Data crossing into a longer-lived domain
is copied, promoted or re-acquired through that domain's handle, never
borrowed across, and persisted state keys by durable identity, never by a
reusable slot. Each mutable state has one owner; consumers borrow within
its lifetime or carry their own handle or copy. The domains, death kinds and
crossing rule are argued in the
[data-oriented doctrine brief](2026-07-02_data_oriented_doctrine_brief.md) §7.

Links: draws on graphshell:node-body-face-model D1, house-style D15 · summary Storage follows the death condition; mutable state has one owner, consumers borrow within its lifetime or carry a handle or copy; persist by durable identity.

*Refined 2026-11-02 (house-style D15): "Consolidate within the
seven-statement budget (Recommended)"; ownership and lifetimes refine this
statement. Node-body-face-model D1 continues to apply.*

### Stack 7. Source size

No tracked source file in an owned repository exceeds 600 physical lines,
comments and blanks included, without a technical reason listed beside it.
Tests, scripts and shaders count; generated files and imported upstream
sources are excluded by declared path, never by size. An oversize file splits
along responsibilities the code already has, not into arbitrary fragments,
and the split widens no visibility. Docs are outside the ceiling and are
deduplicated instead. One shared check enforces it at commit and over the
whole tree in each repository's local gate; files over it today are listed as exceptions and leave the list through named
cleanup slices.

Links: draws on isocosm:592, isocosm:1160, house-style D10, house-style D11, house-style D13 · summary No source file over 600 lines without a listed reason; splits follow existing responsibilities; one shared check at commit and in each local gate.

*Latest outcome 2026-11-02, applies (house-style D13): "Retire it and cite
stack.7 (Recommended)". D11's "Local only" continues to apply.*

### Stack 8. Bound work and distinguish its outcomes

Background work names its owner, admission bound, cancellation path and
completion condition. Queues, concurrency, retained bytes and retries have
explicit limits supplied through configuration; a deadline bounds waiting
where the operation can stall. The owner joins completion before retiring
the resources the work can touch. Callers can distinguish refusal, failure,
cancellation and success, with enough context to decide what to do next.
Fallback and retry are explicit policy; they preserve the cause and cannot
turn missing authority or structurally invalid input into accepted work.

Links: draws on house-style D15, stack.6 · summary Background work has an owner, configurable limits, cancellation and joined completion; refusal, failure, cancellation and success remain distinguishable.

*Added 2026-11-02, outcome new (house-style D15): "Consolidate within the
seven-statement budget (Recommended)". Stack 5 first folded into stack.3;
its key is retired, not reused.*

### Stack 9. Two roots, by nature

The stack has two roots, one for each of its natures, and a leaf goes to the
root of its nature. The data root holds how a fact is stored, sealed, witnessed
and framed: the causal log and its domain interface, the container graph, the
store backends, the pack header, the state witness and the record frame. The
presentation root holds how a thing is shown and acted on across a process
boundary: the projection contract, the WIT world, the scene types and their
clocks, the command vocabulary and the colour arithmetic. Custody and identity
stand beside them. Neither root names anything above it, and the engine, the
reading and spatial suppliers, the hosts and the applications stand on both.
A shared library that is neither contract nor store is placed by asking which
root's consumers all sit beside; one that both sides of the same nature link
is a commons of that root, and the root's record says so. One trunk is not
forced: folding the presentation root into the data root would save a
repository and lose the separation.

Links: draws on engine-split D12, engine-split D19, domain-contract C5, linked-data-platform-lessons D10, cycle-breaks D9 · summary Two roots by nature, data and presentation; a leaf goes to the root of its nature; neither root names anything above it; one trunk is not forced.

*Added 2026-11-07, outcome new (engine-split D22): "i will give one more
statement for a core architectural concern. 8?". The data root is Eidetic and
the presentation root is the Graphshell repository's contract workspace as of
this date; the repositories may change, the rule does not.*

## Surfaces

### Surfaces 1. A frame hosts one presentation

A frame, a leaf of a window's frame tree, hosts one presentation: a reader's
document, or a surface producer, which is what an embedded application is. A
tile is a frame presenting a node's resource; a pane is a frame presenting an
answer to one question about the graph or the system. The two differ by what
is presented, not by whether an application is embedded: a surface producer
can be what either presents. A product does not add a pane for an application
it can present, and a node whose address names a correspondent opens the
conversation that answers for it rather than a tile.

Links: summary A frame hosts one presentation, a reader's document or a surface producer; a tile presents a resource, a pane answers a question; an embedded application is a presentation, not a pane.

*Latest outcome, 2026-10-28, applies: ruling 9 of the GUI catalog record (§10),
no Steward pane; the live-jobs monitor is distillery's embeddable half, a
presentation, and armillary's host actors show on the node and in apparatus.
Added the same day, outcome new, from the record's §8 (Mark, 2026-10-26,
refining its rulings 1 and 3) and its ruling 7; rulings 1, 3 and 7 apply too.*

### Surfaces 2. A catalogued surface is an emitting one

A surface in a product's catalog is one with an open and dismiss seam, an
emission site that diagnostics and the accessibility tree can hear. An
intended surface is recorded in the design record that intends it, not
declared in code before it is built, and nothing stored references it. The
state a surface's shell keeps, open, side, width, focus, context binding, has
one owner, and the surface renders what that owner holds.

Links: summary A catalogued surface has an open and dismiss seam; an intended one stays in its record until built; shell state has one owner, and the surface renders it.

*Added 2026-10-28, outcome new, from the GUI catalog record's ruling 2 (Mark,
2026-10-26). Rulings 4 and 5 apply under it: cards are kinds of content the
focus-card slot summons, and the dock contract is Cambium's shell's.*

### Surfaces 3. A contribution is one of four kinds

What a pack or mod adds to a product's GUI is a presentation (a reader or a
surface producer, which a node's resource opens into), a pane adapter (a feed
into an existing pane), a pane (a new question), or a servitor (a participant
under a grant). A manifest declares its contributions in a typed list beside
its dependency names, and activation verifies the declaration: a mod that
registers what it did not declare is refused. A presentation's declaration is
one type, owned by the reader layer and carried by the manifest: what it
presents, how it reaches the screen, what it accepts back, and which
operations it exposes. Trust stays with the manifest's capabilities and the
pack's requested scopes; leaves and clips stay graph facts that the
operations act on.

Links: draws on stack.3 · summary Four contribution kinds, declared in a typed manifest list and verified at activation; a presentation's declaration is one type the reader layer owns and the manifest carries.

*Added 2026-10-28, outcome new, from the GUI catalog record §8 and its
rulings 6 and 8 (Mark, 2026-10-26); the code landed 2026-10-27, as the
record's §9 closing says.*

### Surfaces 4. A surface names its five authorities

A surface is a projection of a node set under a scope, a ring, a keeping
level and a build profile: the node decides what is shown, the scope which
part and from where, the ring who may see or act, the keeping level whether
it is in focus, kept or ambient, and the profile whether the engine behind it
is linked. A surface is catalog-complete when each of the five has a line for
it, including its refusal, and a catalog says which profile it was read
under.

Links: draws on stack.2 · summary A surface projects a node set under scope, ring, keeping level and build profile; it is catalogued when each authority has a line for it, refusal included.

*Added 2026-10-28, outcome new. The GUI catalog record's §5 read the five
authorities from the stack on 2026-10-26 and left their home unruled; Mark
ruled on 2026-10-28 that the rules live here and the inventory stays in a
document.*
