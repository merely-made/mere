# Data formats across the stack

**Status, 2026-10-06:** ruled (F1 to F5). The rule is in force for new files.
Migrating the outliers (§4) and the postcard version-header audit (F4) are
separate lanes.

The question came from Isocosm. Mark asked whether the TOML used for generated
datasheets suits the wing's entity rosters and rulesets. Then he widened it:
"Should we consider the rest of the stack, too? Like mods, configs, ambient
strata, generated tables…? Which format is preferable, and is a split like in
1 enforceable throughout similar situations in the stack, or would 2 be?" And
then: "What do you think of the binary side?" On "ambient strata": "More like
background/foreground in projection grammar". That gave no separate format
question.

## 1. The rule

**Text: split by who writes the file.**
- **A person authors or reviews it → TOML.** Configs, mod and pack manifests,
  rosters, catalogues, rule and material tables, and generated datasheets
  people review in diffs.
- **A program writes it, it is interchange, or the outside world dictates the
  format → JSON.** Saves, receipts, expectation maps, imports (the SRD,
  Datasworn), and web-mandated manifests.
- **Datasheets take Livery's shape** (`genet/components/livery/properties.toml`):
  a `schema` version, owner and consumer, a `status` saying how the sheet is
  generated, and `[sources.*]` provenance.
- **One serde type per record loads either format** during a migration.
- **Identity is never the source text.** Any content address or state hash is
  taken over the parsed form or its canonical encoding, so changing the format
  never changes a world's identity.

**Binary: two formats, by job.**
- **postcard for bytes that stay inside the stack:** storage, snapshots,
  hashes, our own wire. Always behind a version header checked before
  positional decoding, as in `isometry/shared/wing-formats` ("versioned
  interchange records… the raw header checked before positional decoding").
  postcard carries no field names, so an unversioned record cannot change
  shape without breaking stored bytes.
- **CBOR where bytes cross a trust or ownership boundary, or must be readable
  without our schema:** p2panda's protocol, signed grants, open project files.
  Encoding is deterministic, and decoding is strict wherever bytes are signed,
  content-addressed or validated on the wire. That is ruling 24's line in the
  [device pairing plan](mere_docs/implementation_strategy/2026-10-02_device_pairing_by_key_plan.md).
- **No third binary format in production.** Codec races stay
  dev-dependencies.

**Enforcement.** The rule is one sentence per side, plus one check: no
hand-edited JSON in a pack or config directory. The check is a lane of its
own (§4). TOML everywhere would not be enforceable:
- outside mandates keep JSON (browser-extension manifests, the SRD and
  Datasworn imports, JS and web interop);
- TOML has no null and no top-level array;
- deeply nested data (rule-element trees) reads badly in it.

JSON everywhere would lose comments in authored files, and would contradict
Cargo and Mere's `.wasm.toml` mod manifests.

## 2. Evidence: text, 2026-10-06

Tracked data files, excluding tooling, tests, fixtures and receipts:

| Repo | json | toml | lua |
|---|---:|---:|---:|
| genet | 40 | 3 | 0 |
| retinue | 35 | 11 | 0 |
| isometry | 12 | 0 | 6 |
| mere | 11 | 0 | 0 |
| mere-verify | 11 | 0 | 0 |
| mer3ly | 1 | 9 | 0 |

No RON, KDL or YAML is used as data. `serde_json` is declared in 16 repos;
`toml` in 5 (genet, mer3ly, mere, mere-verify, retinue).

What each format holds today:
- **Tooling config** is TOML (Cargo, rustfmt, rust-toolchain, CubeCL's runtime
  config).
- **Mod manifests disagree.** Mere's mod loader reads a `<mod>.wasm.toml`
  sidecar (`crates/system/registry/src/mod_loader/loader/free_fns.rs:65`,
  `:110`). The wing's packs use `isometry-pack.json` and
  `mesocosm-pack.json`.
- **Datasheets are mostly TOML.** Livery's `properties.toml` and
  `consumed_longhands.toml`, mer3ly's `content/*.toml`, and Retinue's firmware
  package index.
- **Authored content records are JSON.** Mesocosm's
  `packs/mesocosm/processes/*.json`, five small flat records.
- **Machine-written records are JSON.** genet's 32 WPT expectation maps,
  receipts, Isocosm saves.
- **Imports are JSON.** The VTT's SRD `data/{monsters,items,spells}.json`.
  Datasworn's JSON is cited in the wing record's ruling 539.
- **Themes.** Mere's theme plan (2026-06-22, archived) recommended TOML for
  theme files.

## 3. Evidence: binary, 2026-10-06

| Format | Production use | Non-test call sites |
|---|---|---:|
| postcard | Mere: `graph-kernel` 30, `notochord` 11, `gaz` 6, `content-contract` 4, `muniment` 4, `chatelaine` 4, `insigne` 3, `mesquite` 1. isometry: `isonetry`'s session wire, Isocosm, isometer, `wing-formats`. netrender and genet capture corpora. knot. | Mere 63, isometry 61, netrender 9, genet 5, knot 3 |
| CBOR (`ciborium`) | p2panda's protocol, which Mere decodes at about 70 sites through `p2panda_core::cbor`. `pandect`'s wallet grants (`src/wallet_grant/mod.rs:47`). hocket's open `.hock` file, a zip of `manifest.cbor`, chosen "to align with Moothold" (`hocket/design_docs/2026-05-18_initial_plan.md:1406`). | `pandect` 4, hocket 5 |
| MessagePack (`rmp-serde`), `cbor4ii`, `minicbor-serde` | none: dev-dependencies of `graph-kernel`'s dissolution-gate codec race ("candidates must be self-describing so unknown-forward foreign facets survive") | 0 |

**Version headers.** A quick grep for version or header names beside the
postcard code found mentions in `graph-kernel` (7), `notochord` (11),
`muniment` (12) and `insigne` (4), and none in `gaz`, `content-contract` and
`chatelaine`. That is a hint, not a finding. F4's audit settles it.

## 4. Outliers and follow-on lanes

- **The wing's pack manifests and Mesocosm's process records** are authored
  JSON. Under F1 they become TOML, loaded by the same serde types. Isocosm's
  roster move (wing design record ruling 623) starts as TOML datasheets.
- **The VTT's SRD data** stays JSON as an import, converted on the way in if
  a datasheet form is wanted.
- **The check:** no hand-edited JSON in pack or config directories. It
  belongs in each repo's pre-commit or doc audit, once the outliers move.
- **F4's audit:** every postcard record that is stored or sent gets a
  version header if it lacks one, with a test that old bytes still load.

## 5. Rulings (2026-10-06)

- **F1. The text rule.** Asked "Which text-format rule should the stack
  follow?" The options were: split by who writes it; TOML wherever possible;
  JSON everywhere. Mark: "Split by who writes it (Recommended)".
- **F2. Where the rule lives.** Asked where the rule should live. The options
  were: Mere's docs plus one line in `Code/CLAUDE.md`; Mere's docs only; the
  wing only for now. Mark: "Mere's docs plus one line in Code/CLAUDE.md
  (Recommended)". So this brief is the record, and `Code/CLAUDE.md` gains one
  line pointing here.
- **F3. The binary rule.** Asked after "What do you think of the binary
  side?": "Which binary-format rule should the stack follow?" The options
  were: two formats, by job; CBOR everywhere; no rule. Mark: "Two formats, by
  job (Recommended)".
- **F4. The version-header audit.** Asked whether to audit `gaz`,
  `content-contract` and `chatelaine`, which showed no version header near
  their postcard records. The options were: audit through a lane; note only.
  Mark: "Audit through a lane (Recommended)".
- **F6. One header helper.** Asked when the lanes were planned: "How should
  postcard records get their version header?" The options were: one shared
  helper in Mere (wing-formats' 8-byte magic and `u16` version, with
  `frame`, `unframe` and `peek`, in one small Mere crate that Mere's crates
  and wing-formats both use, old unheadered bytes loading as version 0);
  per-crate headers. Mark: "One shared helper in Mere (Recommended)". F4's
  lane is
  [`2026-10-06_postcard_framing_plan.md`](mere_docs/implementation_strategy/2026-10-06_postcard_framing_plan.md).
- **F7. The helper's name.** Asked at the framing plan's H1 checkpoint:
  "What is mere's postcard framing helper crate called?" It must be
  published, since muniment 0.1.2 is on crates.io and H4 moves it onto the
  helper, and publishable Mere infrastructure takes `mere-` plus one plain
  word (`mere-transport` is on crates.io). The options, all free on
  crates.io and unused in the tree on 2026-10-06, were: `mere-framing`;
  `mere-header`; `mere-frame`. Mark: "mere-framing (Recommended)". The name
  is claimed by a real publish once H1 lands, and that publish waits on his
  word like any other.
  *Claimed 2026-10-06:* Mark said "Publish 0.0.1 now (Recommended)", and
  `mere-framing` 0.0.1 is on crates.io, published from mere `13a6b49d`.
- **F5. Isocosm's rosters.** Asked whether rosters (`Founding::SpacedRoster`,
  `SEEDED_KINDS`) move from Rust into datasheets. The options were: yes,
  through a plan; only new ones; not now. Mark: "Yes, through a plan
  (Recommended)". This is recorded as ruling 623 of the wing design record
  (`isometry/mesocosm/design_docs/2026-09-18_wing_design_plan.md`), where
  the plan lives.
