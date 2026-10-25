# athanor

**Athanor** is the distillation furnace of Distillery,
the Mere platform's works.

An athanor is the furnace that holds a constant low heat for as long as the
work takes. This crate is the authority half of the
[alembic](https://crates.io/crates/mere-alembic) workshop: what a furnace pass
is, what it may propose — distillation, cleanup, facet proposals — and the
grants, runs, pending petitions, refusals, costs, pause, revoke, retry, and
dissolve over the bounded actors the workshop admits, of which the furnace
was always the first.

Two rules it keeps:

- **It proposes; it never owns truth.** A furnace pass emits proposals. The
  authority that owns the affected store decides. Athanor is not, and does
  not become, a graph-truth authority.
- **It is scheduled, not resident.** Djinn contains the scheduler and runs
  Athanor as one resident service, composing this crate's authority the way
  it composes Distillery's works: putting owners together and inventing
  nothing. Lifetime is Djinn's; the domain is Distillery's.

Ruled 2026-09-02: Athanor's authority lives with Distillery, the domain,
rather than with Djinn, the resident, by the same precedent Djinn set for the
works themselves. It sits beside Alembic as `ports/distillery/athanor`; the
package is `mere-athanor` and the library keeps the name.

Lives in the [mere](https://github.com/merely-made/mere) workspace at
`ports/distillery/athanor`.

**Status (2026-10-24):** forgetting, consolidation, image-reference cleanup
and retirement proposal logic are implemented. The general workshop service,
its driver and two-host controls remain unbuilt. The [Alembic plan](../../../design_docs/mere_docs/implementation_strategy/2026-06-24_alembic_implementation_plan.md#5-workshop-reconciliation-and-first-workflow-2026-10-24)
owns their integration and the first retained-passage evidence-brief workflow.
Athanor persists domain decisions and drives Servitor's existing reducer;
an Armillary adapter performs admitted work and reports execution completion.
Each destination owner separately acknowledges its accepted effects.

## License

MPL-2.0
