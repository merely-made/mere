> Copied 2026-10-19 from the M4-local `~/Code/testing/retained-document-20261016/SYNTAX.md`; see the [receipts index](../README.md).

# Supported syntax and lowering policy: `probe-toml10-pack-v1`

This proof uses the unchanged taplo 0.14 lexer/grammar on Rowan 0.17, plus a
pinned independent `toml` 0.8.23 parser as a TOML 1.0 gate. This is a deliberately
bounded dialect; it is not a claim of full TOML conformance.

A document can be retained and saved as a draft even when syntax or lowering
fails. Successful typed lowering requires all of these checks:

1. Taplo has no syntax errors and its DOM semantic validation succeeds.
2. `toml` 0.8.23 accepts the text, with no float or date/time values anywhere.
3. Both parsers produce identical JSON-safe parsed values.
4. The copied Isocosm `ContentPackManifest` deserializes and passes its validator.
   The probe-local generator value ABI supports only `{ type = "text", value =
   "..." }`; other generator value variants are outside this proof.

Within those gates the dialect permits strings (basic, literal, multiline),
integers, booleans, arrays, inline tables, ordinary tables, arrays of tables,
quoted and dotted keys, comments, and LF or CRLF. Numbers are signed 64-bit
integers as admitted by the parsers; alternate integer spellings normalize to
the same parsed number. Tests cover the real pack syntax, quoted/dotted keys,
multiline strings, integer bases, booleans, and arrays. They do not exhaust
all allowed lexical combinations.

Floats, all date/time values, and TOML 1.1 additions such as `\e`, `\xHH`,
multiline inline tables, and omitted time seconds block typed lowering.
Their text can still form a retained draft with diagnostics.

## Input and edit boundaries

- Input must be UTF-8. Invalid UTF-8 and UTF-8 BOM input are explicitly rejected.
- The proof limits input to 2 MiB and lexical bracket/brace nesting to 64.
  Exceeding a limit refuses open/publication; it does not create a draft.
- No newline normalization: saved text preserves the revision's exact UTF-8 bytes.
- Edit ranges use UTF-8 byte offsets and must end at code point boundaries.
- Batch edits use base-revision coordinates, sorted by start/end. Overlaps and
  same-start edits are refused, including ambiguous same-position insertions.
  Result ranges in the receipt account for earlier edits in the batch.
- Each successful edit fully reparses before publishing. Incomplete edits are
  allowed; syntax/policy diagnostics are attached to the resulting revision.
  Schema and application diagnostics are returned by lowering with that stamp.

## Unknown content and identity

Unknown properties that do not prevent typed decoding/validation are retained
and included in the domain hash. This includes the fixture's entire
`[vendor.experimental]` table. Unknown fields inside the probe-local `GenValue`
text variant are rejected by that variant's decoder. Unsupported scalar types
block lowering even in an opaque extension.

The content hash is SHA-256 of the revision's exact authored UTF-8 bytes.
The proof domain hash is SHA-256 of:

```
UTF8("retained-pack-proof-v1\0") || canonical_json(parsed_value)
```

The prefix ends with one actual NUL byte. Canonical JSON recursively sorts
object keys, retains array order, uses integer-only JSON numbers, and encodes
UTF-8 without formatting whitespace using the pinned serde_json version.
It includes all parsed unknown properties. Comments, whitespace, integer base
spelling, and key order do not affect this domain hash. Schema defaults are
not materialized into the hash; absent and explicitly defaulted fields can
have different hashes. This is an explicitly local proof encoding, not a new
stack wire/hash format or a product identity implementation.

## Save and reload policy

Saving names an immutable revision and holds an exclusive per-path lease file
until completion/cancellation. A second cooperating resource cannot begin a
save while the lease exists. An older revision than the last saved revision
cannot start a save. Edits may publish newer revisions while a save ticket is
held; completion records the revision actually saved and computes dirty state
from content equality, without marking the newer revision saved.

The backend checks the disk hash against the last observed saved bytes at
begin, before writing, and before replacing. External changes are refused;
there is no merge or forced overwrite. The caller must explicitly reopen the
external file as a new resource to reload it. The backend writes a same-directory
create-new temporary file, syncs its contents, then renames it over the target.
Pre-replacement failures leave the saved marker and target unchanged, and
ordinary cancellation removes lease/temp files.

These guarantees apply to cooperating writers using the same path spelling in
an owned local testing directory. The hash check followed by rename is not
compare-and-swap against an uncooperative writer. Path aliases/symlinks,
network filesystems, adversarial lock replacement, permission preservation,
directory fsync/crash durability, and stale-lease recovery are not implemented
or qualified. A crash can leave a lease file, which fails closed on later saves.

## Identity and history

Document stamps contain the caller-supplied name, an in-process instance
counter, and a monotonically increasing revision number. Reopening creates a
new instance; handles are not durable or serializable identifiers. Node handles
combine the stamp with Rowan's kind/range pointer. Another instance or revision
is rejected before resolving the pointer. There is no remapping.

Undo publishes older retained content under a fresh revision and receipt;
old handles stay stale. It does not reverse the isolated consumer's admitted
state. A configurable count limit bounds history and receipts. Readers holding
an `Arc<Snapshot>` can keep evicted content alive; releasing resource history
does not revoke such snapshots. The parser's builder cache is fresh for each
parse and dropped before publication. There is no persistent cache or
incremental parse/structural-edit path.
