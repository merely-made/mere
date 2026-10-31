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
second host's adoption is that host's receipt.

Links: draws on isocosm:600, isocosm:organs.5, house-style D1, house-style D2, house-style D12 · summary Build on the stack we have: unneeded crates stay out, stale ones leave; components are modules, reserved homes receive their capability; seams serve named consumers; extraction waits for a planned second host, not an arrived one.

*Refined 2026-10-30 (house-style D1, D2): "Named futures only"; "Fold into
Stack 1".*

*Refined 2026-10-31 (house-style D12): "second host is planned", because
"shows up doesn't happen, you know, until it's planned."*

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

Links: draws on isocosm:592, isocosm:method.3, house-style D10, house-style D11 · summary No source file over 600 lines without a listed reason; splits follow existing responsibilities; one shared check at commit and in each local gate.

*Refined 2026-10-30 (house-style D11): "Local only".*
