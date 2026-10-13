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

## Shared resources and coop draft

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
The first source harness qualifies these view contracts. Production owner
bindings, Cambium controls, Graphshell rendering and actual peer activity retain
their separate gates in the
[Moot plan](../../design_docs/moothold_docs/implementation_strategy/2026-06-12_moot_object_m1_plan.md#shared-resources-and-coop-interaction-draft-2026-10-11).

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

Graphshell's opt-in `moot-resources` host binds a mount to an existing verified
projection session. It rechecks retained session authority and the actual
resource owners before dispatch, refreshes current owner reads afterward, and
keeps opened bytes and feedback local to each reading. The separate
`moot-resource-proof` binary exercises two mounts on the native Cambium host
using signed, public in-memory fixtures. It installs no activity owner. See
the [host continuation](../../design_docs/moothold_docs/implementation_strategy/2026-06-12_moot_object_m1_plan.md#admitted-graphshell-resource-mount-2026-10-12).

## License

MPL-2.0
