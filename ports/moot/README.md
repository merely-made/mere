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
The existing optional `captured-web` module is a bounded implementation;
the composed conversation and coop experiences remain planned. The active
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
[coop lifecycle parity plan](../../design_docs/mere_docs/implementation_strategy/2026-09-16_coop_lifecycle_parity_plan.md).

## License

MPL-2.0
