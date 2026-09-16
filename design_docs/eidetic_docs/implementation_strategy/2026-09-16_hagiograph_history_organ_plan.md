# Hagiograph, the history organ

**Date:** 2026-09-16

**Status, 2026-09-16:** plan. H1 (this rescope) landed with this document;
H2 to H4 are dispatched. No consumer yet.

**Owns:** turning the 26-line `hagiograph` reservation into the history organ
Mark ruled on 2026-09-16: the record of standing marks and the feat rule over
it, the deep-time seam that gives a generated world a past, and later
promotion, retelling and memorials.

**Does not own:** storage of what happened, which stays in
`muniment::Journal` and each product's own log; any product's simulation or
its vocabulary of feats; associations, which are the wing's impresa; the
divinity organ, which is the wing's hagioglyph and consumes this one; descent
across worlds, which is `fili`.

**The design record** lives in the games wing, since the rulings were made
there: `isometry/mesocosm/design_docs/2026-09-16_isoscape_family_plan.md`
(rulings 1 to 14, §2 deep time, §3.0 the three generation buckets, §4 Phase D).
This plan carries only what this crate builds.

---

## 0. Rulings (Mark, 2026-09-16)

1. **The hagiograph is the history organ.** It judges significance, through
   the record of standing marks and the feat rule; it generates the past that
   significance is judged against, through deep time; and later it promotes,
   retells and memorializes. The reservation's boundary "not ordinary event
   history" holds for **storage**.
2. **It stays in mere's portable eidetic core.** This answers the eidetic
   review brief's question 7 (`research/2026-09-04_eidetic_review_brief.md:428-431`:
   keep, move out, or retire).
3. **The record's mechanism moves here and its vocabulary stays with each
   product.** Axes and holders are the product's types.
4. **A feat beats a mark that stood before the reckoning.** A first mark on an
   empty axis is never a feat, and two lineages that beat one older mark at
   the same boundary are both feats.
5. **Deep time runs the product's own simulation** for a span fixed in the
   product's world rules, stops on an epoch boundary, and hands the world
   over with its clock continued. The product keeps the baseline, the world
   and its history, rather than re-running deep time on load.

## 1. What exists

- **The reservation**: `crates/eidetic/hagiograph`, `src/lib.rs` 26 lines of
  doc comment, no dependencies, no code consumers
  (`research/2026-09-04_eidetic_review_brief.md:69-72,342`).
- **The mechanism to move**: Mesocosm's `WorldRecord`
  (`isometry/mesocosm/crates/mesocosm-core/src/record.rs`, 405 lines), high-water
  marks per `(Feat, Scale)` with their holders, joined by maximum so records
  merge without a protocol, and `is_unprecedented`, `untouched`, `note`,
  `merge`, `axes`, `filled`. Its docs already name this crate's role: "the
  journal holds everything, tulpa holds what is retold" (`:43-44`; tulpa was
  this organ's earlier name).
- **Its serialized form is pinned.** `WorldRecord` is part of Mesocosm's
  world snapshot, whose state hash is byte-pinned. The moved type must encode
  identically under postcard.

**Fixture bytes, generated 2026-09-16 from the real type.** A `WorldRecord`
noted with `(Growth, Local, 130, species 4)`, `(Growth, Local, 130,
species 2)`, `(Predation, Worldwide, 2082, species 2)` and `(Spread,
Regional, -3, species 300)` encodes as:

```text
[3, 0, 0, 132, 2, 2, 2, 4, 1, 2, 196, 32, 1, 2, 4, 1, 5, 1, 172, 2]
```

and an empty record as `[0]`. Variant order: `Feat` is Growth, Predation,
Symbiosis, Endurance, Spread, Construction; `Scale` is Local, Regional,
Worldwide; a holder is `SpeciesId(pub u32)`.

## 2. The shape

Illustrative, not compile-ready.

```rust
pub struct Mark<H: Ord> { pub high: i64, pub holders: BTreeSet<H> }
pub struct Record<A: Ord, H: Ord> { marks: BTreeMap<A, Mark<H>> }

pub struct Entry<A, H> { pub axis: A, pub value: i64, pub holder: H }
pub struct Judgement { pub took: bool, pub feat: bool }

impl<A: Ord + Clone, H: Ord + Clone> Record<A, H> {
    /// Notes every entry. `took` is the running answer a product's screen
    /// already shows; `feat` is judged against the record before the first
    /// entry was noted, so it does not depend on entry order.
    pub fn reckon(&mut self, entries: &[Entry<A, H>]) -> Vec<Judgement>;
}

pub trait Epochal {
    /// One tick of the world's own rules, with no hand on anything.
    fn advance(&mut self);
    fn epochs(&self) -> u64;
    fn tick(&self) -> u64;
}

pub struct DeepTime { pub epochs: u32 }
pub struct Handover { pub span: DeepTime, pub from_epoch: u64, pub to_epoch: u64, pub from_tick: u64, pub to_tick: u64 }

pub fn run<S: Epochal>(world: &mut S, span: DeepTime, max_ticks: u64) -> Result<Handover, DeepTimeError>;
```

The tick ceiling exists because a product's epoch rule may never close an
epoch (Mesocosm's `Gated` and `PlayerTriggered` rules do not), and deep time
must refuse rather than spin.

## 3. Steps

| Step | Builds | Done when |
| --- | --- | --- |
| **H1** | Rescope the README, crate doc and manifest description; this plan | The reservation's text names the history organ and keeps the storage boundary |
| **H2** | `Mark`, `Record`, merge and the queries, generic over axis and holder | Merge is commutative, associative and idempotent; a higher mark replaces, a tie shares; the fixture bytes above are reproduced exactly by a mirror of Mesocosm's types |
| **H3** | `Entry`, `Judgement`, `Record::reckon` | A first mark is never a feat; a beat of an older mark is; two beats of one older mark at one reckoning are both feats in either order; `took` equals sequential `note` exactly |
| **H4** | `Epochal`, `DeepTime`, `Handover`, `run` | A toy simulation under a three-epoch span stops on its third boundary; a zero span runs no ticks; a simulation that never closes an epoch is refused at the ceiling |
| **H5** | Consumers pin it | Mesocosm pins this crate alone at its own mere revision, adopts `Record` for `WorldRecord` with its six byte pins holding, and implements `Epochal` (wing plan D6, D7) |

Later, not planned here: promotion and retelling, the attention that makes a
legend nobody tells fade, and memorials handed to the stack's voxel lane.

## 4. Constraints

- Portable eidetic core: no I/O, no threads, no clock, no product types, and
  nothing that stops it building for `wasm32-unknown-unknown`.
- Dependencies: `serde` with `derive`; `postcard` only as a dev-dependency for
  the byte fixture.
- MPL-2.0, the workspace license.

## Findings

- **2026-09-16.** The fixture bytes in §1 come from running Mesocosm's
  `WorldRecord` under postcard, not from reading the encoding rules.

## Progress

- **2026-09-16.** Plan written; H1 landed with it.
