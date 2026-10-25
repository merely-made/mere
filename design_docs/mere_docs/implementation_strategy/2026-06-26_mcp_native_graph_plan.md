# MCP-native graph (future)

**Date**: 2026-06-26
**Status**: Scoped, future, not yet. Captured from the
[borrowed-ideas brief](../research/2026-06-25_borrowed_ideas_brief.md) at Mark's
direction (scope both directions now, build later). Mere is agentic, so MCP (the Model
Context Protocol) is the natural agent boundary: it makes external agents first-class
citizens of the space and gives internal agents a standard way to reach out, both
without bespoke glue. No work scheduled.

## Two directions

**Expose: Mere as an MCP server.** The graph (nodes, edges, queries, crawl, clip)
offered as MCP tools and resources, so an external agent (Claude or any MCP client)
operates on the user's space. The clean substrate is the existing
[command registry](../../archive_docs/2026-09-02_retired_plans/2026-06-21_command_registry_configurable_menus_plan.md): the agent's
API is already the same `ActionRegistry` the user drives (the agent harness already runs
over `Command::ALL`), so the MCP tool set is that registry exposed over MCP, plus
read-only resources for nodes and subgraphs. Mutating tools route through the same
command path, so they inherit its gating.

**Corrected 2026-10-06 (S14 pass):** that substrate is gone. `ActionRegistry` survives
only as a string literal
(`crates/system/registry/src/mod_loader/loader/registry/registry_default.rs:71`),
`Command::ALL` has no hits (the agent harness went with meerkat), and the command
registry plan is retired. Two live plans now schedule MCP as an optional adapter: the
[Graphshell reference host plan](2026-07-27_graphshell_reference_host_plan.md)'s H9, over
Graphshell's projection, query and intent grammar, and the
[djinn family resident services plan](2026-08-22_djinn_family_resident_services_plan.md)'s
F2, over djinn's authenticated agent door.

**Open, raised by the S14 pass (2026-10-06):** which substrate does the expose half ride
now? Options: Graphshell's intent grammar (H9's adapter); djinn's authenticated agent
door (F2's adapter).
**Answered 2026-10-25:** both, as one adapter. [`graphshell-mcp`](../../../crates/graphshell/graphshell-mcp/README.md)
serves any admitted Graphshell route as stdio MCP and dials djinn's owner-only
application door, so the grant, the admission and the attribution stay djinn's.

**Consume: internal agents reach external MCP servers.** Agent nodes and the harness
gain an MCP client, so an agent node's policy can call out to external MCP tools. The
outbound half of the same boundary.

## What it must ride

- **Capability gating.** Exposed mutating tools sit behind the in-app permission spine
  (the [capability-gate catalogue](../research/2026-05-14_capability_gate_catalogue_brief.md)),
  with consent. Read scope can reuse the federation capability work
  ([federation_interop](2026-06-26_federation_interop_plan.md)) when an external agent's
  reach should be a scoped subgraph rather than the whole space.
- **Provenance.** Every external-agent mutation asserts a `ProvenanceSubKind` edge (the
  borrowed-ideas brief's assert-on-every-agent-mutation rule), so external action stays
  auditable and reversible.

## Decisions to settle (when picked up)

- Transport (stdio / SSE / streamable HTTP) and auth for the exposed server.
- Which commands are exposed, and the read-only vs mutating split (mutating off by
  default, behind consent plus capability).
- Whether expose or consume leads. Expose is the higher-value, more-contained half: it
  rides the command registry and needs no new agent runtime.

## Cross-references

- [borrowed-ideas brief](../research/2026-06-25_borrowed_ideas_brief.md): the source.
- [command registry plan](../../archive_docs/2026-09-02_retired_plans/2026-06-21_command_registry_configurable_menus_plan.md): the
  `ActionRegistry` that becomes the MCP tool set.
- [capability-gate catalogue brief](../research/2026-05-14_capability_gate_catalogue_brief.md):
  the gating the exposed tools sit behind.
- [federation_interop plan](2026-06-26_federation_interop_plan.md): capability-scoped read
  for external agents.
- [document_script_substrate plan](../../archive_docs/2026-07-03_completed_plans/2026-06-21_document_script_substrate_plan.md),
  [local_models_harness brief](../research/2026-06-24_local_models_harness_brief.md): the
  agentic lanes this complements.

## Progress

- **2026-06-26, scoped (future).** Captured from the borrowed-ideas brief at Mark's
  direction, both directions, marked future. The expose half rides the existing command
  registry as its tool surface and the capability spine as its gate; the consume half
  adds an MCP client to the agent path. No work scheduled, no code.
- **2026-10-06 (S14 pass).** Claims corrected against the tree at mere 535bca11, from the
  D2 record in support/doc-audit/d2/batch_48_s14_phase_b10.md (the status line held): the
  retired command-registry substrate annotated, the Graphshell H9 and djinn F2 adapters
  named, and the expose half's substrate written in as an open question.
- **2026-10-25, expose half built (stdio, local).** `crates/graphshell/graphshell-mcp`: a
  stdio MCP server, MCP 2026-07-28 plus the handshake revisions, over one admitted
  Graphshell route. Projections are resources; the route's advertised actions are listed by
  `mere_actions` and invoked through one envelope tool, `mere_invoke`, carrying the
  observed epoch and revision. The server holds no key: `mere-mcp --app <id> --route
  mere/<domain>` is admitted by djinn's door, which attributes accepted edits to the persona
  via that app. 17 tests against a route that behaves like djinn's; `claude mcp list`
  connects. Prompts, subscriptions and HTTP are not
  built. The consume half is untouched.
- **2026-10-25, served over djinn's real mere route.** `resident_mere::tests::mcp` (3 tests)
  runs the adapter over the reservoir, route grants and session item: an unadmitted app
  reaches no mere; an admitted agent applies an edit through `mere_invoke`, reads it back and
  undoes it, with the stored change naming the app the door admitted; a malformed edit is
  refused and changes nothing; the ambient route advertises no edit. The socket path
  (`AppRouteCarrier` to a running djinn) and a model-driven session are still unrun.
