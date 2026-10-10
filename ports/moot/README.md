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

## License

MPL-2.0
