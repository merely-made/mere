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
abstraction earns one.

Links: draws on isocosm:600, isocosm:organs.5 · summary Build on the stack we have: unneeded crates stay out, stale ones leave, own crates when the abstraction earns one.

*Moved 2026-10-22 from isocosm's doctrine, its Stack 1, worded for any product
("Reword to 'a product'").*

### Stack 2. The stack supplies identity and trust

Protected identity and admitted connections come from the stack; a product
maps an authenticated peer to its own participant. A product never
manufactures keys, grants or trust roots to fill a missing stack
configuration.

Links: draws on isocosm:962 · summary Identity and admitted connections come from the stack; a product maps peers to its participants and never manufactures keys or trust roots.

*Moved 2026-10-22 from isocosm's doctrine, its Stack 2, worded for any product
("Reword to 'a product'").*

### Stack 3. Seams and formats

Seams between components stay clean even at some cost in speed: a missing
boundary bites later, and hardware improves. A renderer sits behind a scene
contract and can be swapped. Formats follow open standards; a product makes
its own only where none holds the thing.

Links: draws on isocosm:27, isocosm:78 · summary Seams stay clean at some cost in speed; renderers swap behind a scene contract; open formats first.

*Moved 2026-10-22 from isocosm's doctrine, the stack-wide half of its Stack 3;
the wing's placements stay there ("Split as drafted").*

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

### Stack 5. Extension rungs

Extension rides four rungs: data (open sets a product defines), scripts
(authored rules and packs, lowered to definitions by digest), components
(outside code, on Wasmtime where a platform allows) and native crates (the
first-party cores). Total componentisation is not worth it applied totally: by
call frequency times data volume, cold and narrow goes behind a component when
it wants to be pluggable, and hot or wide stays native.

Links: draws on isocosm:77, isocosm:78, isocosm:1000, stack.3 · summary Data, scripts, components and native crates; cold and narrow goes behind a component when pluggable, hot or wide stays native.

*Moved 2026-10-22 from isocosm's doctrine, the rungs and their rule from its
Stack 14; the wing's instances stay there ("Split as drafted").*

### Stack 6. Storage follows the death condition

Data is classified by what makes it dead: a boundary (scratch, task, frame,
device epoch), an event (one entity removed from a living world) or lost
reachability. Storage and ownership follow that and the access pattern, not
allegiance to an allocator. Identity is stable; representations are
projections with their own lifetimes. Data crossing into a longer-lived domain
is copied, promoted or re-acquired through that domain's handle, never
borrowed across, and persisted state keys by durable identity, never by a
reusable slot. The domains, death kinds and crossing rule are argued in the
[data-oriented doctrine brief](2026-07-02_data_oriented_doctrine_brief.md) §7.

Links: draws on node-body-face-model D1 · summary Storage follows the death condition; cross domains by copy or handle; persist by durable identity.

*Added 2026-10-25, outcome new ("Write stack.6 now"), the first statement from
this repository's own rulings.*

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

Links: draws on stack.5 · summary Four contribution kinds, declared in a typed manifest list and verified at activation; a presentation's declaration is one type the reader layer owns and the manifest carries.

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
