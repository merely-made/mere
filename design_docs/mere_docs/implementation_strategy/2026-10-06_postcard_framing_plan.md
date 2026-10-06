# Postcard framing: one version header for stored and sent records

**Status:** plan, 2026-10-06. H1 waits on a crate name (checkpoint below).

Carries out F3, F4 and F6 of the
[data formats brief](../../2026-10-06_data_formats_brief.md). postcard
carries no field names, so a stored or sent record without a version header
cannot change shape without breaking the bytes already written.

## Rulings carried

- **F3.** "Two formats, by job (Recommended)": postcard inside the stack,
  always behind a version header.
- **F4.** "Audit through a lane (Recommended)": every postcard record that is
  stored or sent gets a header if it lacks one, with a test that old bytes
  still load.
- **F6.** "One shared helper in Mere (Recommended)": wing-formats' header in
  one small Mere crate that Mere's crates and wing-formats both use.

## Findings (2026-10-06)

- **wing-formats has the pattern**
  (`isometry/shared/wing-formats/src/lib.rs`): an 8-byte magic and a `u16`
  version (`HEADER_LEN = 10`), `frame` to write it before the postcard
  payload, `unframe` to check and decode, and `peek` to read the version
  without decoding.
- **muniment has its own** `REDO_MAGIC = b"muniment-redo/1\n"`
  (`crates/eidetic/muniment/src/directory_backend.rs:69`).
- **postcard call sites outside tests:**
  - Mere 63: `graph-kernel` 30, `notochord` 11, `gaz` 6, `content-contract` 4,
    `muniment` 4, `chatelaine` 4, `insigne` 3, `mesquite` 1;
  - isometry 61;
  - knot 3;
  - genet's and netrender's are capture corpora used by examples.

  A name search found version or header mentions in `graph-kernel`,
  `notochord`, `muniment` and `insigne`, and none in `gaz`,
  `content-contract` or `chatelaine`. That is a hint for H2, not a finding.

## Phases

### H1: the helper crate

- **A small Mere crate** with `frame(magic, version, &T)`,
  `unframe(magic, bytes)` and `peek(magic, bytes)`, as wing-formats has
  them.
- **One addition: a legacy path.** Bytes that do not start with the record's
  magic decode as unheadered version 0, so records written before H3 still
  load.
- **A documented limit:** an unheadered payload whose first 8 bytes happen to
  equal the magic would be misread as framed. Each magic is chosen to make
  that impossible for its record's leading fields, and a test pins it.

**Done when:** unit tests cover:
- round-trip;
- a version mismatch refused with its number;
- legacy unheadered bytes decoding as version 0;
- a truncated header refused;
- `peek` never decoding the payload.

**Checkpoint: the crate's name.** A new crate name is a naming decision.
Plain technical candidates go to Mark, checked against crates.io and the
naming ledger, before H1 starts.

### H2: the inventory

- **A table of every postcard record that is stored or sent**, across Mere,
  isometry and knot: its crate, its type, stored or sent, its current header
  if any, and whether old bytes exist anywhere (on disk, in a peer's log, in
  a fixture).
- **Corpora and test-only encodings are listed and excluded.**

**Done when:** the table is in this plan's Findings, with each row's source
line cited.

### H3: headers where missing

- **Each record H2 finds without a header is framed by the helper**, with its
  own magic, at version 1.
- **Its test decodes bytes written before the change**, captured before the
  edit, as version 0.

**Done when:**
- every H2 row is framed or excluded with a reason;
- each framed record's old-bytes test passes;
- the owning crates' suites pass.

### H4: the existing headers join the helper

- **wing-formats and muniment's redo log move onto the helper.** Their magic
  and version bytes are kept, so their stored bytes are unchanged; a test
  shows they are byte-identical.
- **wing-formats is in isometry,** which reaches Mere through its git pins.
  So its half waits for isometry's repin onto stable (wing design record,
  ruling 621) and lands as its own isometry commit.

**Done when:** both use the helper with bytes unchanged, and their suites
pass.

## Progress

- 2026-10-06: plan written; F3, F4 and F6 recorded in the data formats
  brief.
