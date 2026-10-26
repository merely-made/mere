# The agent harness: decision drivers in the Alembic workshop

**Date:** 2026-07-03
**Status (2026-10-25):** model-guided design rationale reconciled with the
[Alembic workshop plan](../implementation_strategy/2026-06-24_alembic_implementation_plan.md#5-workshop-reconciliation-and-first-workflow-2026-10-24).
Alembic/Athanor own workshop composition. The [Servitor redesign](../implementation_strategy/2026-08-13_graph_behaviors_plan.md#8-servitor-resident-and-run-redesign-2026-09-20)
owns reusable admission/run semantics; Servitor remains in Eidetic.
R1a/R1b and the Turnstone adapter have executable receipts. Athanor now consumes the same reducer in the deterministic evidence-brief
core (Alembic plan section 5.7). The product adapters, two-host acceptance
and model-guided decisions remain open.

## 1. The composition

Alembic is the workshop where a person creates, grants, runs, observes,
interrupts and dissolves a helper. Athanor owns its domain logic inside
Distillery. A decision driver can be a script, deterministic algorithm or
model; its proposed work passes through independently enforced authority.

Servitor supplies resident binding, admission and recorded-event run
reduction. It performs no I/O and produces no executable requests. Athanor's
host driver persists intents, dispatches admitted work and records observed
results. Armillary supplies actors/tasks, messages, cancellation and execution
completion. A synchronous scenario can drive the same reducer without
Armillary, ESP or Djinn. ESP supplies model execution when selected.

Product owners still authorize, commit and acknowledge their own effects.
Djinn composes the workshop service and its lifetime; it supplies neither a
second run state machine nor new domain authority. Eidetic retains artifacts
and journals using the schemas their interpreting owners define.

## 2. A model decision turn

This is design for a later driver, not a receipt for an integrated tool loop.

1. **Collect authorized context.** Recheck read authority, select exact source
   versions and record the selected behavior revision, observations and
   context references. Retrieval is bounded by configured source/token limits.
2. **Obtain a decision.** Invoke the selected ESP provider with explicit
   limits. Retain provider/session identity and observed usage. Shared Alembic
   controls show actual progress; worker completion is separately observable.
3. **Validate and dispatch.** Parse against the selected domain operation
   schemas, recheck live authority and preconditions, persist intent, then
   dispatch. Persist actual results before publishing their disposition.
4. **Advance or stop.** Apply recorded events through Servitor. Check claimed
   success against observations. A durable proposal can finish generation
   while the workshop goal still awaits review or destination application.

Armillary does not choose whether an action is safe to retry. An interrupted
consequential operation remains unresolved until the owning adapter supplies
evidence. Restarting a worker cannot establish that its effect failed or
authorize redispatch.

## 3. Owner-defined semantic operations

The original brief assumed a shared ActionRegistry/Command::ALL. That
substrate was retired, as the [MCP plan](../implementation_strategy/2026-06-26_mcp_native_graph_plan.md)
records. The retained invariant is that human controls, scripts and protocol
adapters invoke the same domain validation and commit paths.

Each owner supplies versioned operation descriptors: checked arguments,
scope, current availability, preconditions, effect class, result evidence
and reconciliation behavior. Discoverability is not permission. Human menus
and model tool lists may present these operations differently.

The useful families are domain actions, capability-scoped graph/source reads,
and later external tools. The app-host action envelope is a stable ABI;
it does not supply semantic schemas or prove effect completion. Provider
adapters may translate structured calls or validate a strict JSON form.
Malformed output becomes an observed refusal/failure within the run budget.

MCP exposure/consumption remains an optional adapter with its own transport,
disclosure and authorization contract. The workshop's first workflow does
not depend on it or establish a universal action registry.

## 4. Retained work, proposals and accepted effects

Retain causally related identities rather than collapsing work into one
record: installed helper, versioned definition, goal, run, proposal,
acceptance operation, model invocation, mesh job and observation. A graph
projection can make those relationships inspectable without turning every
runtime message into graph truth.

Product mutations retain proposing resident/run attribution and the
applicable accepting authority through the product's own journal. Recording
an external effect neither reverses it nor makes it safe to repeat.

Two postures remain useful:

- **Direct, gated:** an operation has the required live authority,
  preconditions, stable identity and completion/recovery contract.
- **Proposal/apply:** the helper emits a durable proposal. Human acceptance
  is a distinct authorized operation with its own durable outcome.

The first workflow uses the second posture. Its generation run may be
complete while the goal awaits review. A queued port request or a successful
worker return must not be presented as a committed product change.

## 5. Interactive and standing helpers

Interactive work begins through an explicit invocation and can request review
through the shared workshop surface. User presence permits asking; it is not
a grant. Standing helpers additionally need explicit trigger, overlap,
backlog and unattended-action policies.

Servitor watches, ticks, cascades and deadbands have existing consumers.
Connecting them to slow asynchronous work must preserve cursor/disposition
durability and yield rather than holding a synchronous cascade drain.
The initial Alembic workflow is manual; automatic scheduling remains a
separate extension. Procedure learning and evaluated adoption remain R2/R3.

## 6. Provider and target differences

Local and remote execution can share decision-driver contracts. Provider
selection also changes disclosure, credentials, costs and continuation
capabilities. Remote API adapters remain planned; their network placement
follows the owning host's transport policy.

A different provider cannot pretend to resume a native exact-token checkpoint.
Unsupported continuation refuses or starts an explicitly new decision step.
Recovery depends on the operation/effect contract, not merely on whether the
provider is remote.

The reducer is portable sequencing and data. Each host must declare its actual
execution, storage and action capabilities. Browser execution and persistence
need their own receipts; the first workshop gate is native desktop.

## 7. First workflow and future guidance

The Alembic plan's W0–W3 specify an evidence brief from selected retained
passages in Turnstone and Graphshell: one versioned helper definition,
separate installation grants, shared controls/domain logic, exact citations,
review, destination-owned application and cold reopen. That plan is the
single detailed workflow and acceptance specification.

Existing Servitor tests cover reduction and admission, not this new effect
path. The two product adapters must prove real persistence acknowledgement,
stable acceptance identity, refusal, interruption and reconciliation.
A deterministic body is sufficient for that proof.

Later R2 compares optional procedural guidance against flat context with the
same tasks and budgets. R3 evaluates candidates on held-out tasks and checks
authorized, versioned adoption. Servitor defines reusable validation
invariants; Athanor records the owner's adoption decision and selected
revision. Procedure text cannot confer authority or certify its own success.

## Related owners and research

- [Alembic implementation plan](../implementation_strategy/2026-06-24_alembic_implementation_plan.md)
  — workshop composition, first workflow and current acceptance gates.
- [Graph behaviors plan](../implementation_strategy/2026-08-13_graph_behaviors_plan.md)
  — watches, admission, run contracts and proposed R2/R3.
- [Local models harness brief](2026-06-24_local_models_harness_brief.md)
  — model-execution and adapter rationale; current resident qualification is
  in the [inference plan](../implementation_strategy/2026-10-15_resident_inference_and_recall_plan.md).
- [Diagnostics plan](../implementation_strategy/2026-06-08_system_diagnostics_and_accessibility_plan.md)
  — current owner observations and semantic action evidence. The retired
  Meerkat D7 harness is historical prior art, not today's action catalog.
- [Persona model](2026-05-14_persona_model_brief.md), [geist models](2026-05-10_geist_models_brief.md)
  and [MCP plan](../implementation_strategy/2026-06-26_mcp_native_graph_plan.md)
  — identity/context, model research and planned protocol adapters.

## Progress

- **2026-10-24:** reconciled the workshop with Alembic/Athanor and retained
  Servitor in Eidetic. Replaced the retired registry assumption, corrected
  pending-test claims, separated proposals from accepted effects, and made
  the Alembic two-host workflow the single acceptance specification. This
  documentation change implements no workshop, model loop or protocol adapter.
- **2026-09-20:** proposed the shared Servitor resident/run contract and
  separated actor delivery from run policy, historical D7 citations, and
  audit from undo. The graph behaviors plan retains its source/evidence record.
- **2026-07-03:** drafted after Mark's harness discussion, connecting model
  execution with typed actions, attributed work and interactive/standing
  duty cycles. Later ownership and substrate corrections are applied above.
