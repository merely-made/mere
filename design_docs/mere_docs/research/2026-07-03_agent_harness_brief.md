# The agent harness (the run loop for models/agents)

**Date**: 2026-07-03
**Status**: Design brief, revised 2026-09-20. The proposed shared loop now lives
in Servitor under the [resident/run redesign](../implementation_strategy/2026-08-13_graph_behaviors_plan.md#8-servitor-resident-and-run-redesign-2026-09-20).
That plan owns the current boundaries and R1-R3 sequence; this brief retains
the model-guided loop rationale. R1a admission, the R1b reducer and the Turnstone adapter have source implementations with static review; executable receipts remain pending under the Cargo hold.
The original brief binds the layer between two seams that already
exist: the model runtime seam (the [local models harness brief](2026-06-24_local_models_harness_brief.md))
and the typed actuation seam (the D7 agent harness in
[`meerkat/src/agent_harness.rs`](../../../crates/meerkat/src/agent_harness.rs) *(historical citation)* <!-- doc-audit: historical-link -->). Neither owns
the loop that puts a model behind the actions. This doc is that loop.

**Related (the agentic lane):**

- [local models harness brief](2026-06-24_local_models_harness_brief.md) — `InferenceProvider` /
  `AdapterLoader`, backends by target, the inference actor. The brain seam. This doc consumes it.
- [system diagnostics plan, D7](../implementation_strategy/2026-06-08_system_diagnostics_and_accessibility_plan.md) —
  the landed typed harness: `AgentObservation` (surfaces, enabled actions, diagnostics, a11y) +
  `AgentAction` incl. the by-id `Invoke(String)` registry seam. The hands seam. This doc consumes it.
- [command registry plan](../../archive_docs/2026-09-02_retired_plans/2026-06-21_command_registry_configurable_menus_plan.md) —
  the one action id-space (palette, omnibar, a11y, harness). The tool vocabulary.
- [mcp_native_graph plan](../implementation_strategy/2026-06-26_mcp_native_graph_plan.md) — the
  external boundary, both directions. Expose rides the registry; consume is a tool ring here.
- [knot plan, agent nodes](../../archive_docs/2026-08-06_completed_plans/2026-06-24_djot_editor_knot_nodes_plan.md) —
  the durable home of a standing agent: a node whose body is its policy, whose edges are its
  materialized results.
- [geist models brief](2026-05-10_geist_models_brief.md) — the model architecture (base + adapter
  engrams). A geist is what sits behind the provider; this doc does not re-derive it.
- [capability gate catalogue](2026-05-14_capability_gate_catalogue_brief.md) +
  [persona model brief](2026-05-14_persona_model_brief.md) — the gate spine and the scope axis.
- [operator presence overlay plan](../implementation_strategy/2026-06-25_operator_presence_overlay_plan.md) —
  the agent as one of the three operator sources; a run renders presence ("watch your AI think").

---

## 1. The gap

The lane has a brain seam and a hands seam, and both are the right shape:

- **Model execution**: ESP's `InferenceProvider` supplies generation, streaming
  and deterministic stub providers today. Model execution is separate from
  deciding what a resident does next.
- **Host actions**: Turnstone's script/component lanes evaluate participant
  bodies and lower accepted actions through the host. The D7 harness cited
  above is historical context; it is not a current runtime receipt for this redesign.

What neither owns: the loop that assembles context, calls the model, parses a tool call, gates it,
dispatches it, records what happened, and goes around again. The MCP plan calls this "the agent
runtime" and defers it; the D7 plan stops at "let a Gemma-class model sit behind that typed loop"
without saying how. This doc says how.

---

## 2. The loop

A run is a Servitor state machine driven by host-supplied observations and
results. An Armillary actor is one possible execution adapter; deterministic
drivers use the same transitions without allocating a thread per run. Hosts
expose progress and cancellation through their operation surfaces. Per turn:

1. **Assemble context**: the current `AgentObservation`, RAG retrieval over capability-scoped
   engrams (persona/moot scope, per the persona brief), the agent's own policy body (for a
   standing agent node), and the transcript so far.
2. **Infer**: hand the assembled prompt to the `InferenceProvider`; stream tokens back as actor
   messages. Steward shows real status (turn count, current action, budget remaining), never a
   placebo spinner.
3. **Act**: parse the model's output into the internal call representation (§3), gate-check it,
   dispatch it, observe the result (the D7 `AgentStep` already returns result + fresh
   observation), append both to the transcript.
4. **Stop** when the model declares done, an action is blocked and the policy says defer to the
   user, or the budget (turns/tokens) runs out. Every terminal state is recorded in the run
   engram (§4) with its reason.

Servitor owns sequencing, transcript, budgets, retry eligibility and pause/resume
semantics. Armillary carries messages; ESP implements provider calls and streaming.
Capability checks remain independently callable, and the host registry supplies
the tool catalog. In particular, an actor retry must never silently repeat an
external effect whose outcome is unknown. The redesign plan specifies reconciliation.

---

## 3. One tool vocabulary

The commitment: **the model's tool list is the observation's `enabled_actions`, rendered as
schemas.** Palette, menus, a11y, the MCP server, and the model all address the same registry ids.
There is never a parallel "agent tools" catalog to drift out of sync.

Three rings, one representation:

- **Registry actions** (in-app): `Invoke(id)` plus the typed D7 actions. Enablement, labels, and
  ids come from the observation, so the model only sees actions that are currently invokable.
- **Graph reads** (context): scoped node/subgraph/query reads, the same read surface the MCP
  expose half plans to offer as resources. Read scope is a capability, not a prompt convention.
- **Outbound MCP** (external): the consume half of the MCP plan, surfaced to the model as more
  tools in the same list, gated per server.

Internally a tool call is one enum shaped like `AgentAction` plus the read and MCP variants.
Per-provider adapters translate: an API model's native structured tool-calling maps directly; a
small local model gets a strict JSON grammar over the same ids, validated, with a rejected parse
returned to the model as a turn result rather than crashing the run (illustrative shape only; no
signatures proposed here).

---

## 4. Runs are graph material

A run is an engram: transcript, actions taken, results, terminal state, provider/adapter identity
(which geist, which backing model). Every mutation the run performs asserts a provenance edge
pointing at the run, per the borrowed-ideas rule the MCP plan already adopts
(assert-on-every-agent-mutation). Records support attribution and reconstruction;
undo requires operation-specific compensation or reversible commits. Recording
an external effect cannot make it reversible or safe to repeat.

Two mutation postures:

- **Direct, gated**: small actions dispatch through the registry with per-action gating, exactly
  as if the user had invoked them, plus the provenance edge.
- **Proposal/apply**: batch or heavy mutations follow the Athanor split
  ([alembic plan](../implementation_strategy/2026-06-24_alembic_implementation_plan.md)): the run
  emits a changeset engram; adoption is a separate, human, gated act.

The agent node is the durable policy; runs are events under it. An interactive session's run
hangs off the session instead. Either way the transcript is queryable graph history, not app
state.

---

## 5. Two duty cycles, one loop

- **Interactive session**: summoned (omnibar `>` verb or a command), user present, chat-shaped.
  User presence is the consent context: gated actions can prompt per-action. Lean: the session is
  a node from its first turn (consistent with representations carrying node identity), and its
  transcript renders wherever the host puts conversation surfaces; the exact surface (comms
  thread, gloss card, dedicated pane) is a host choice and a user setting, not bound here.
- **Standing agent**: an agent node whose policy body defines trigger (graph signal, schedule,
  manual poke) and scope. No user present, so the posture tightens: pre-granted scoped
  capabilities only, mutations default to proposal/apply, results materialize as the node's
  edges with provenance. This is the Tinderbox-agent-made-spatial from the knot plan, with the
  loop behind it.

Same loop, same transcript shape, same gates; only trigger and consent posture differ. A run in
either cycle emits operator presence (the agent is one of the three operator sources), so watching
an agent work is the presence overlay, not a new UI.

---

## 6. Remote models are a provider, not an architecture

The local models brief binds local runtimes. One extension, stated here so it lands in the seam
and not around it: a remote API model (Claude-class, or any hosted endpoint) is just another
`InferenceProvider` backend over HTTPS, reachable from wasm and native alike. Capability
descriptors already carry context window and loader identity; a remote backend adds an endpoint +
credential, gated like any other outbound capability. Local vs. remote becomes provider choice
per agent, a user setting, with the same loop, transcript, gates, and provenance either way.

---

## 7. wasm vs native

The loop itself is portable: it is sequencing + data, no OS surface. The split it inherits is the
brain's (the local models brief §4: in-browser = RAG + small-model inference via Burn-wgpu;
heavier runtimes native) plus remote providers everywhere. The hands seam is host-provided, so
the browser lane's harness exposes whatever action surface that host has (the orrery browser lane
has a smaller registry than meerkat; the loop does not care).

---

## 8. Where it lives

Proposed home: modules in the existing Servitor crate, preserving host-free
data and transition logic and independently usable participant authority APIs.
The [redesign plan](../implementation_strategy/2026-08-13_graph_behaviors_plan.md#8-servitor-resident-and-run-redesign-2026-09-20)
owns the module boundaries. ESP supplies model execution, Chartulary/Eidetic
store records, and host adapters supply actions and effects. A separate agent
crate is not the starting assumption. Procedural guidance is one optional
decision driver alongside scripts and deterministic fixtures.

---

## 9. First slice (no model required)

R1a in the redesign plan supplies resident binding, lifecycle and run admission
with the existing Turnstone consumer; its source changes await executable
validation and the portable dependency pin. R1b supplies source for the shared
run reducer, a deterministic provider fixture and the Turnstone adapter.
Executable acceptance gates remain pending and include
cancellation, revocation, stale replies, uncertain effects and replay, alongside
call validation and attribution. A real model is unnecessary for that contract
proof. R2 measures optional procedural guidance;
R3 evaluates and adopts procedure revisions. Their done-conditions live in the
plan rather than in a second implementation checklist here.

---

## 10. Owned elsewhere

- **Model architecture, adapters, governance**: geist brief.
- **Runtime backends, training, wasm inference ceiling**: local models harness brief.
- **MCP transport/auth and the exposed tool split**: mcp_native_graph plan.
- **Agent-node authoring UX** (the `=query` promotion, editor): knot plan.
- **Presence rendering + agent-transparency grants**: operator presence overlay plan.
- **Marketplace/compute**: communal-compute + resource-coordination briefs.

---

## 11. Open questions

- **Constrained decode for small local models.** API models emit structured calls natively; a
  Gemma-class model via Burn-wgpu needs grammar-constrained decoding (logit masking) or
  validate-and-retry over the strict JSON grammar. Lean validate-and-retry first (cheap, provider-
  agnostic); constrained decode is a Burn-side follow-up once a real local model is wired.
- **Context budget policy.** How much observation + RAG + transcript fits a small model's window;
  what gets summarized vs dropped. Empirical, per-provider, rides the capability descriptor.
- **Standing-agent integration.** Servitor watches, ticks, cascades and deadbands
  already have Turnstone consumers. The redesign must connect those primitives
  to resumable runs, including overlap policy and slow model completions, without
  blocking a cascade drain or replaying an effect.
- **The interactive surface.** Comms thread vs gloss card vs pane; a user setting per the
  configurability doctrine, but the default needs picking when the session slice lands.
- **Multi-agent.** Runs spawning runs (an agent invoking another agent node) is representable
  (a run engram pointing at a child run) but deliberately unbound here; revisit once single runs
  are real.

---

## Progress

- 2026-09-20: proposed Servitor as the shared resident/run owner, replacing the
  unassigned standalone-agent-crate assumption. Reconciled actor versus run
  policy, current trigger consumers, historical D7 citations, and audit versus
  undo. The graph behaviors plan owns the design and pending R1-R3 gates.

- 2026-07-03: Brief drafted from a design conversation with Mark ("a harness for models/agents,
  how should that look"). Grounded against the landed D7 harness code (`agent_harness.rs`:
  `AgentObservation` / `AgentAction::Invoke(id)` / `AgentStep` / `StopFocusedOperation`), the
  local models harness brief, the MCP plan, and the knot plan's agent nodes. Core commitments:
  one tool vocabulary (registry ids, never a parallel catalog), runs as provenanced graph
  material with the Athanor proposal/apply split for heavy mutations, two duty cycles over one
  loop, remote models as a provider choice inside the same seam. No code proposed.
