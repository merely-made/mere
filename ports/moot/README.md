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

## License

MPL-2.0
