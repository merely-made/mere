# Gemot

The community surface containing **murmurs** (secret conversations), **moots**
(spaces of agreement), and **coop** (shared activities among peers).

Use the hardware you have. Share resources with people you trust, contribute
work voluntarily, and make contributions visible. Platform participation does
not require a subscription; communities decide their own contributions and
how to meet actual operating costs.

Conversation presentation must mount independently for consumers such as
Signalman. A shared presentation preserves each protocol's identity, delivery
meaning, and privacy limits. Public radio traffic does not become secret merely
by appearing alongside a murmur.

The `gemot` library owns governance and membership; `murm` owns conversation
exchange; Commons and application domains own their shared state; Stickleback
owns replication machinery. Coop exposes activity lifecycle without moving
application state into a second store. Turnstone composes these surfaces.

The technical package remains `mere-moot`, library `moot`, at `ports/moot`.
The optional `captured-web` module and portable conversation view are bounded
implementations. Product conversation rendering and live coop remain open. The active
[Gemot implementation lanes](../../design_docs/2026-08-22_turnstone_suite_composition_and_capability_census.md#gemot-murmurs-moots-and-coop-2026-09-13)
define the next receipts.

## The coop lifecycle contract

`moot::coop` is a view contract, not a state owner. A `LifecycleReport` is one
consumer's own view of itself at one clock reading: a verdict (joined, left,
expired, revoked, not joined, not admitted), the consumer's named reason,
membership and grant facts, revocation facts, whether reading continues, and
the clock. It is never peer truth and nothing here is durable.

Its two consumers are Turnstone's place worker and the Commons practice peer
fixture. They differ in recorded ways — the fixture treats one grant as all
its authority, only Turnstone cuts reading on revoke — so `conform` walks the
invite/join/leave/reconnect/expire/revoke sequence against a consumer's own
`LifecycleDriver` and takes those differences as declared `Capabilities`
rather than failures. Domain state, history, merge and authorization stay with
Gemot, Commons, the place worker and the fixture's store. See the
[coop lifecycle parity plan](../../design_docs/archive_docs/2026-10-06_completed_plans/2026-09-16_coop_lifecycle_parity_plan.md).

## The Commons conversation view

`moot::conversation` composes the existing Comms pane model and a consumer's
coop lifecycle report. Each pane keeps its own attention and drafts. Switching
or clearing selection preserves edits; an explicit confirmed send clears only
the submitted draft if its content still matches. Thread-load requests carry
local identities so stale completions cannot attach after a switch or reload.

The optional `commons-chat` feature supplies `conversation::snapshot` over a
real encrypted `ChatReplica` and the owner's current `CommonsAuthority`.
It preserves stable Personae authors, original operation identities, reply and
edit facts, and causal message order. Withheld authority, revocation, causal
and retraction facts contribute counts. Message authors do not establish a
membership roster or presence. The default view builds on Wasm without Commons,
Iroh, a store or a transport runtime.

`Pane::prepare_send` returns addressed content for the existing application
owner. The owner rechecks current membership, Gemot authority and keys before
authoring; a snapshot or coop report cannot authorize a later write. Failure
keeps the draft. No new wire protocol, message store or key distribution lives
in this view. The CPU receipt uses two encrypted replicas and signed Gemot
delegation/revocation facts; it qualifies no network, browser UI, presence or
media path. See the [Moot implementation plan](../../design_docs/moothold_docs/implementation_strategy/2026-06-12_moot_object_m1_plan.md#moot-conversation-and-coop-continuation-2026-10-10).

## Resource owner and Cambium controls

`resource-owner` connects an admitted reader to current Gemot collections and
Eidetic local content. The first reader verifies public local blobs up to 4 MiB;
private/sealed content requires a separate access/key adapter. Reads acquire no
remote content and write no storage. A reviewed destination is bound into each
collection-fork intention and checked against current Gemot authority again at
dispatch. Reference forks retain ancestry and copy no content bytes. Unsupported
content-copy, content-fork, hosting and application actions need their owners.

`resource-view` supplies host-neutral Cambium summaries, an adjacent inspector
and coop controls. Each reading has independent attention and a scoped request
outbox. Hosts consume requests, deliver owner feedback and refresh authoritative
reads; rebuilding or arranging the UI performs no domain action. The host owns
mounting, style, rendering and application-session routing. See the
[owner and control continuation](../../design_docs/moothold_docs/implementation_strategy/2026-06-12_moot_object_m1_plan.md#resource-owners-and-cambium-controls-2026-10-11)
for acceptance boundaries.

## Shared resource controls and coop

`moot::resource_coop` is the portable interaction model for a disclosed shared
collection beside a domain-owned activity. Each reading keeps independent exact
resource and activity attention. Preparation returns an owner-routed intention;
the owner must check current authority, revision, storage, audience and quotas.
Offers and coop reports grant no rights. A collection fork preserves references
and ancestry; content copying and content forking remain separate actions.

The model distinguishes address-only/resolved content, hosting promises and
observed retrievals, and owner acceptance/refusal. Acceptance feedback does not
invent custody, availability or participation. Refreshes invalidate stale local
completions; a new contribution does not retarget an exact selected revision.
The optional `presentation` feature renders ordinary Cambium summaries and
resource/activity inspection. Unknown, withheld and unsupported actions stay
disabled with their owner's reason. `control::Controller` prepares intentions;
`control::Owner` supplies fresh reads and dispatches through the application.
Selection, refresh and close affect only that reading.

The optional `resource-owners` feature maps Gemot's authorized fauna and selected
collection's exact frontier/membership commitment. The caller supplies current
content, hosting, offers and activity disclosures separately. A missing collection
refuses rather than widening the read. The application must permit reader metadata
access first; Gemot's fauna filter checks authors' write authority.
`owners::local_content` verifies already
permitted local/embedded bytes through Eidetic with a caller-selected byte budget;
it performs no network fetch or custody write. Backend reads still materialize a
stored value before the actual-size check.

Graphshell's opt-in `moot-resources` feature mounts independent readings in its
retained tree. Its host keeps private request bindings and transient opened text.
The supplying application drains addressed commands, obtains a fresh owner read,
calls `prepare_dispatch`, then routes the operation to its actual owner. Completion
requires the same live reading and token. Refresh replaces owner facts; failures
end the request with a refusal. Closing never means leaving an activity.

The `moot_resource_owner` example is an explicit local acceptance fixture using
scoped Gemot authority and real Eidetic verification. Its identities are synthetic
and unsealed. Its first receipt covers Open; the handler continuation below adds
retention and hosting. Live signed peer revocation, production activity adapters,
and collection/content fork writes retain separate gates in the
[Moot plan](../../design_docs/moothold_docs/implementation_strategy/2026-06-12_moot_object_m1_plan.md#shared-resource-controls-and-owner-reads-2026-10-12).

`resource-handlers` adds the native composition of those actual owners.
`handlers::Owner` receives the existing Moot service, signing identity, encounter
and custody backends, optional wallet sealer, and explicit configuration. Current
Moot admission and configured read/copy powers are checked at dispatch. Retention
uses Eidetic's atomic owner-scoped copy. Hosting additionally requires explicit
content-owner approval for that exact public payload and Gemot's audience power;
an unsigned privacy tag supplies neither. The current hosting adapter serves
public-portable content only; private retention supports the wallet sealing seam.

Disclosed action terms are bound to preparation, so changed settings require
another request. Acceptance feedback changes no custody or hosting fact until a
new owner read. A hosting command can fail after a copy succeeds; the owners use
separate stores, so the copy remains and the host must refresh records before
retrying. The supplying application serializes dispatch with authority/configuration
ingestion. Automatic deletion and cancellation are not introduced.

`moot_resource_handlers` exercises this composition with synthetic identities,
real signed membership/contribution/Standing facts and durable Redb databases.
Its rendered host qualifies retain, bounded public hosting, refresh and restart;
the native serving API separately checks held bytes and current authority. Network
serving, live peer revocation, full historical-cut validation and native Turnstone
mounting remain separate gates. See the
[handler continuation](../../design_docs/moothold_docs/implementation_strategy/2026-06-12_moot_object_m1_plan.md#retention-and-hosting-handlers-2026-10-12).
Graphshell's opt-in `moot-resources` host binds a mount to an existing verified
projection session. It rechecks retained session authority and the actual
resource owners before dispatch, refreshes current owner reads afterward, and
keeps opened bytes and feedback local to each reading. The separate
`moot-resource-proof` binary exercises two mounts on the native Cambium host
using signed, public in-memory fixtures. It installs no activity owner. See
the [host continuation](../../design_docs/moothold_docs/implementation_strategy/2026-06-12_moot_object_m1_plan.md#admitted-graphshell-resource-mount-2026-10-12).

## Optional live collection interaction

The `ocapn-coop` feature adds `ocapn_coop` over a stream already secured and
admitted by Murm/Notochord. It registers no endpoint or discovery service.
The ordinary port remains usable without this runtime or Gemot governance.
The protocol is the bounded OCapN suite profile identified by
`mere/gemot-coop/ocapn-suite-31f0b806/v1`.

`MootSessionAuthority::from_admitted` takes the actual secure admission result
and expected network. Its immutable subject and admission epoch enter the
runtime through host-supplied `SessionAuthority`; Syrup arguments cannot supply
or change them. Construct a fresh `session_object` and runtime authority context
for each admission. A previous object's subject or epoch cannot acquire a new
session's authority. The service's `mere.gemot` / `/services/gemot-coop` /
`connect` grant permits adapter use only.

`CollectionOwner` owns one configured collection and serializes its actual
`Moot`, host signing identity and current admission revocation ledger. Every
view, proposal and callback emission checks retained session authority, current
Moot membership and the current signed typed collection grant/delegation.
Read members can view; proposals additionally need Write access. The host signs
an accepted membership change under its own checked Gemot authority, and the
receipt separately names `submitted_by`, `authored_by`, admission epoch and the
ordinary operation hash. Exact collection heads make a repeated or stale
proposal fail; reconnect never automatically replays a mutation.

The callable methods are `view`, `subscribe` with a callback capability,
`unsubscribe` with its returned id, and `set-membership` with one bounded JSON
`MembershipProposal`. They expose no arbitrary records, collection selectors,
clocks, subject claims, evaluator or method reflection. View and receipt
records contain bounded JSON bytes. Views reuse `coop::LifecycleReport` as the
owner's current reading, with no new lifecycle authority.

The host must ingest membership, constitution, delegation and revocation
updates through `host_update` under the same coordinator. Mutating cloned
underlying stores outside it breaks the stated serialization contract. After
an update, `emit_current` repeats current membership/typed-grant authorization
at a fresh clock reading and checks retained-session lapse immediately before
synchronous bounded callback queue insertion; it never waits for a peer reply. An emission
already admitted to that queue may finish in flight after a later revocation;
subsequent emissions fail closed. Queue refusal drops that subscription.
Callback references and disconnect watchers have per-owner and per-session
ceilings, and session closure removes the retained callback references.
Reads and proposals authorize at their captured owner-clock reading under the
coordinator. Already-authorized work may finish if time advances or the remote
disconnects during its existing domain operation. The direct `read` and
`propose` APIs are trusted host paths; remote calls use the bound session object,
which also checks runtime session lifetime.

Accepted proposals return Gemot's normal command receipt and outbound operation.
The configured nonblocking `PublicationHook` offers them to the host's existing
publication path; it creates no second durable store. A queue refusal is
reported in the receipt, while the committed operation remains recoverable
through `Moot::outbound`. The trusted hook must itself perform bounded,
nonblocking work. Submitter attribution belongs to the returned/local receipt;
Gemot's ordinary durable history records the host author. Retaining the
submitter receipt uses the host's existing audit/publication path.

`testing/moot-ocapn-coop` is an isolated source consumer for the adapter, using
real signed Gemot operations and independent replica intake. It avoids the
port's unrelated browser/rendering dev dependencies. The canonical phase and
qualification boundaries live in
`murm/design_docs/2026-10-29_secure_sessions_and_live_interaction_plan.md`.

## License

MPL-2.0
