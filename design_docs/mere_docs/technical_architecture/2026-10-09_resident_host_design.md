# Initial resident host design

**Date:** 2026-10-09
**Status:** Proposed for user review. This is not an approved implementation plan.

This proposal belongs to the [graph semantics follow-ons](../implementation_strategy/2026-10-09_graph_semantics_followons_plan.md).
It preserves the [bounded P1–P5 lane](../implementation_strategy/2026-10-04_graph_semantics_plan.md)
and its accepted semantic rulings, qualification and publication receipts.

## First milestone and authority

Graphshell/Djinn keep their existing owned complete GraphSession and add a
ResidentGraph for presentation. Commands from that partial display go to the
same writer; ResidentGraph remains immutable. This is the provisional first
milestone, subject to user steering. The resident reader can retain fewer payloads,
but the host process still holds the complete writer, baseline and history.
Candidate/replay cloning can increase the peak. Report process memory separately
from reader-record savings; this milestone makes no bounded-process-memory claim.

`ports/graphshell/src/mere_host.rs` already owns GraphSession and staged storage.
`ports/djinn/src/resident_mere.rs` shares hosts by session and sends revision
notices. Reuse that ownership and event path. Refuse an independently opened
writer without proven ownership. Do not add a general multiwriter lock service,
authority epochs or persistent request-receipt log for this milestone.

Turnstone's custom snapshot/facet persistence is a subsequent explicit authority
boundary. A dependency repin does not activate residency or migrate authority.
Its later design must preserve legacy open/save/refusal behavior and prevent old
snapshot persistence and GraphSession from writing the same graph concurrently.

## Resident source, roots and settings

Full remains the compatibility setting. Neighborhood is configured per graph
with hops, selected projection and manual or host-driven refresh. Use P5's typed
Surface/Resource addresses, zero-hop shown bindings and one-hop relations.
Equal UUID values across strata stay distinct.

The host retains the union of all open projection roots, explicit graph pins
and every pane's transient focus. Pane focus does not create a durable pin.
Keep remains its existing per-view control, separate from graph pinning.
An authored roster persists when members are unloaded or missing. Exact empty
scope remains empty; it must not fall through a helper that clears to Show all.
Identity Forme rooting every Surface cannot demonstrate neighborhood savings.
Choose an authored scoped projection when qualifying partial behavior.

Closing a pane releases only roots no longer needed by any remaining pane.
Demand loads inspected data; it is not proof that an edit's dependencies are
complete. Editing and global validation still use the complete writer.

## Typed live commands and observation checks

Adapt UI intent to typed stable-ID live commands, then resolve NodeKeys inside
the complete candidate through GraphSession::edit_now. Never carry resident
NodeKeys into the writer. Let existing capture produce exact replay records;
keep Author, product validation, replay comparison, journal and undo authority.

`crates/graph/graph-kernel/src/graph/apply.rs` distinguishes live and replay
operations. Replay tag edits use legacy Surface setters; live tag edits affect
the shown Resource under current Author. ReplayAssertRelationByIds targets
Surface assertions; live AssertRelation uses current placement. ReplayAddNode
also differs in Resource refresh. CapturedDelta is not the interactive vocabulary.
Imported replay stays on its existing Complete path. Validate external UUID
strings before replay, whose existing parser assumes valid captured IDs.

Proposed commands: CreateAddress, EditNode, SetFacet, SetPinned, SetMimeHint and
AssertRelation, carrying UUIDs, decoded JSON and existing EdgeAssertion values.
Pandect does not depend on Graphshell's product types. The initial adapter
preserves these existing operations from `ports/graphshell/src/product.rs`:

| Operation | Complete-writer adaptation and checks |
|---|---|
| Create address | Do the existing complete URL lookup first; return its UUID if found. Otherwise require the explicit Surface UUID absent and use live AddNode with Some(id), plus SetNodeTitle when nonempty. Preserve Resource/visit capture; explicit IDs work on wasm. |
| Edit title/tags | Require Surface existence and unchanged shown binding. For observed replacement, compare Surface/shown-Resource records and current content tags. Trim input and compute tag differences from complete node_content_tags; use live SetNodeTitle, RemoveNodeTag and InsertNodeTag. Title retains existing Surface behavior; tag withdrawal keeps other authors' assertions. |
| Set product facet | Require Surface existence and unchanged observed Surface record. Trim the facet name, parse JSON fallibly, then use live SetNodeFacet and existing validation. |
| Assert product relation | Require both Surface IDs. Cites, Hyperlink and UserGrouped require unchanged shown bindings and observed Resource records; CollectionMember and FrameMember use Surface preconditions. Use live AssertRelation with current Author's asserter IRI and writer-resolved keys. Preserve exact captured handles and placement. |
| Set pin | Require Surface existence and unchanged observed Surface record; use live SetNodePinned with the desired boolean. |
| Create file metadata | Keep existing content-reference validation/address creation, then live SetNodeMimeHint and SetNodeFacet. Do not infer stored bytes from a resident view or silently regroup existing host changes. |

Proposed preconditions: Exists/Absent(GraphAddress), RecordUnchanged(address,
digest), ShownUnchanged(surface, resource) and ContentTagsUnchanged(surface, tags).
Expose the resident catalog's record digest through a planned accessor, and reuse
P5 record encoding to compare current complete records. Stored records include
complete incident copies even when their endpoints are not resident. Do not
hash the partial Graph's filtered incident list. Shown binding needs its separate
check because it lives in the catalog row. These checks are conservative and
may refuse an unrelated incident change; unrelated source progress alone need
not refuse if all command observations still match.

Require applicable checks before mutation; missing targets cannot silently become
successful no-ops. Validate all commands in the batch, including references to
items created earlier in it, before applying. Rejected commands change neither
writer nor reader. Invalid JSON/IDs, stale observations, codicil read-only state,
unqualified placement and product validation retain their specific errors.

Initial partial commands exclude raw complete-record/pair replacement, deletion,
withdrawal, history manipulation, fields/couplings, imports and arbitrary closures.
Return NeedsCompleteOperation. Existing global operations and undo/redo remain on
the same owned Full writer, with their existing conflict/kept-part semantics.
Settings may refuse Complete operations rather than silently opening a writer.
Exact-statement withdrawal is a later explicit command, not a family-selector
substitute for an inspected handle. A genuinely bounded writer is separately
deferred: it needs an operation/dependency/validation/history contract.

## Prepare, write, acknowledge and refresh

`crates/system/pandect/src/graph_session.rs` applies before storage: apply calls
apply_now and then store; pending describes unstored writes and stored acknowledges
a committed batch without reapplying. Preserve that contract. A store error is
not evidence that an edit never happened.

Recommend preparing addressable generation/record/catalog/head WriteOps separately
in `crates/system/pandect/src/addressable_graph/write.rs`, then combining them
with existing Pending forced-checkpoint writes. The prepared object carries source
cursor/revision and generation. Pause further writer edits for this first staged
commit; await without holding the browser host borrow, acknowledge only confirmed
storage, then refresh the resident display. Source/session changes reject stale
acknowledgments. `ports/graphshell/src/web_session.rs` supplies the existing
prepare/write/install pattern; planned helpers extend it without another journal.

Keep outcomes distinct: Rejected changes nothing; AppliedButUnstored means the
owned writer already changed but preparation/storage remains pending; Durable
means storage is confirmed; PendingPublication means the edit is durable while
addressable publication or reader refresh remains pending. The last coherent
reader remains usable throughout. Do not present pending publication as an edit
failure that should be submitted again.

If the smaller two-commit path is chosen, store the edit first and publish next;
retry publication only. The recommended combined batch avoids that intermediate
durability split on supported atomic backends. Same-batch retry uses fixed journal
positions and existing Applied {first,end,revision}/Pending state, not another edit.
After uncertain storage, compare intended journal/change/checkpoint/head writes
under ownership or reopen/recover. Matching writes acknowledge; absent writes
retry the same batch; mixed state refuses. After host loss, reopen authoritative
history and show current truth rather than resubmitting old UI commands.

Memory/redb/IndexedDB support atomic apply; Directory uses exclusive lifetime
ownership, a shared write mutex and redo recovery, with reopen after poisoned
failure. Fjall's sequential apply cannot claim this atomic contract. Unsupported
backends refuse the combined operation or retain their explicitly qualified
Complete persistence path. Zip does not establish cross-process ownership.

## Host-driven refresh and portable coverage

Provide manual refresh and configurable host-driven checks: after confirmed local
publication, on foreground/attachment and at a minimum host check interval.
Use existing idle/event pumps and Djinn notices, not a mandatory thread/runtime.
Suspended browser tabs catch up on resume. Publication cadence is separate:
whole materialization is expensive, so batch at existing store/idle boundaries
and flush on explicit save/detach. Reader polling cannot expose unpublished edits.

Add a planned refresh-if-changed head fast path before catalog/record loading.
Allow one refresh in flight and coalesce newer notices. Each completion checks
host lifecycle token, session/graph, roots revision, policy revision and request
sequence. A session/root/policy switch discards stale completion. Publication
during a coherent load queues another check; acknowledging it cannot clear a
newer notice. Carry latest host coverage into the installed view, including
context changes that arrived during await. Failures retain the previous reader.

Add product-neutral coverage to the portable snapshot boundary in
`crates/chirograph/src/lib.rs` and Graphshell's snapshot producer. A proposed
defaultable Option<PortableCoverageV1> carries limits with layer, reason, separate
Surface/Resource ID lists and optional count. Preserve Possession, Residency,
Disclosure, Synchronization and Projection meanings. Absent old-wire metadata
means unreported coverage; an empty supplied note means no known limit within
that supplied graph, not universal completeness. Do not encode coverage as truth.

Present unloaded/missing distinctions and demand/refresh actions in the host
shell; carry them through the actual endpoint to its client. Context-only coverage
must still produce a client update. Compose host observations with computed
Residency using existing set_host_coverage. Pending stays separate observation
context and keeps its retention policy. Preserve P5's available-truth digest:
unloaded-only source changes and Pending/coverage-only refresh must not advance
truth revisions or rederive successful queries; resident truth changes do.

## Maintenance prerequisite and deferred cleanup spec

Do not collect generations in this milestone. Later cleanup is confined to the
owned residency namespace and requires protected in-flight loads plus atomic
reference/head/publication checks. Existing readers are unprotected across their
awaited reads; activate cleanup only after all readers/publishers adopt its
contract under a quiescent ownership transition. Loaded views own their bytes;
subsequent demand selects current head. Conservatively retain crash-left refs.
Never erase session baseline/journal/history, archives or blobs as generation GC.

`crates/eidetic/muniment/src/backend.rs` supplies default-refusing transact.
`crates/eidetic/muniment/src/indexeddb_backend.rs` currently snapshots all store
values for it. A later scoped transaction-read extension must fetch declared
values and bounded prefix key names in one transaction, abort on overflow,
retain native Send/wasm ?Send and default typed refusal for old implementors.
`crates/system/pandect/src/wallet_sealed_backend.rs` must preserve authenticated
reads/sealed writes and suppress the batch on any error. Scoped reads remove
unrelated values, not oversized selected-record peaks. Detailed GC chronology,
reachability, budgets and concurrent-publication protocol need a separate spec.

## Qualification and done conditions

Qualify scoped two-pane roots, closing one pane, exact empty/unloaded rosters,
Keep versus pins, and Full clearing Residency. Compare each allowed operation
with existing Complete product behavior, including author tags, Resource versus
Surface placement, equal UUIDs, exact statements, facets and navigation.
Exercise malformed/missing/stale targets and unchanged-source positive controls.
Reopen and undo must retain exact history. Force preparation/store/publication/
refresh failures, same-batch retries, session switches and newer notices during
await; previous views survive and no command is applied twice.

Run a real Graphshell/Djinn endpoint and portable client path, native durable
reopen and actual Firefox IndexedDB. Verify context-only notices/coverage and
query-cache retention separately from truth changes. Report full process memory
and resident record/IO ownership separately. Existing qualified P5 tests do not
qualify these new host paths automatically.

Done: the owned host exposes configurable residency and scoped roots, the named
partial-display edits, explicit persistence state, manual/host refresh and visible
coverage through its real endpoint; close/reopen preserves identity and history.
Turnstone migration, bounded writers and cleanup remain separate reviewed work.
