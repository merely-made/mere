# alembic

**Alembic** is the recall and workshop component of
Distillery, the Mere platform's works. Ruled a port of its own 2026-08-22 and
re-ruled into Distillery 2026-09-02; the distinction survives as a component
boundary.

**Status (2026-10-25):** recall and the workshop’s shared records/read model
are implemented. Enable `workshop` for versioned passage, proposal, review,
acceptance and receipt records plus typed controls; this feature requires
Serde only. Athanor supplies the deterministic domain and optional worker. The [Alembic plan](../../../design_docs/mere_docs/implementation_strategy/2026-06-24_alembic_implementation_plan.md#5-workshop-reconciliation-and-first-workflow-2026-10-24)
owns its first workflow: assemble an evidence brief from retained passages in
Turnstone and Graphshell, review it, and apply it through each destination's
owner. Servitor remains in Eidetic and supplies admission/run semantics;
Armillary supplies execution. No two-host workshop acceptance is claimed.

An alembic is the still that sits on the athanor's constant low heat: what
distils is what the furnace has been holding. This port is that pair for your
own work — the memory it accretes, and the bounded actors that run over it.

It splits in the [castellan](https://crates.io/crates/castellan) mold:

- **the embeddable half** (feature `recall`): the memory surface — three
  levels (short-term, long-term, codicil), promotion and eviction, the codicil
  browser, and lexical and embedding recall over a mere's traces. A host that
  wants memory and no agents takes this alone.
- **the authority half**: the workshop — agent identity and purpose, granted
  reads, writes, actions and watches, model and tool selection, run history,
  pending petitions, refusals and costs, pause, revoke, retry, and dissolve,
  with exact attribution into the target application's history. It lives in
  [Athanor](https://crates.io/crates/mere-athanor), the sibling crate, in the
  domain that owns it; Djinn schedules it as one resident service and invents
  nothing.

**Athanor was always an agent.** The distillation furnace is a bounded actor
under a grant, so the workshop generalizes the furnace rather than standing
beside it.

The boundaries are the point: inside
[distillery](https://crates.io/crates/distillery) but not the model works
(Distillery runs models, its Alembic component runs work over them), not the
store (codicils and retention are eidetic's), and not the grant algebra (that
is servitor's, over personae's identity).

The package is `mere-alembic` because crates.io `alembic` is the Linux
Foundation's VFX-format binding; the library keeps the product name.

Lives in the [mere](https://github.com/merely-made/mere) workspace at
`crates/distillery/alembic`. Its shared recall implementation moved out of
the port directory on 2026-10-07 so Pandect can consume it without depending
on a port. Athanor remains at `ports/distillery/athanor`; Distillery continues
to own the workshop domain.

The distillation plan intends to accept a Fleece `Article` rather than raw page
bytes, but the reservation has no Article-consuming code and deliberately has
no Fleece dependency. Re-add it only with the Athanor consumer.

## License

MPL-2.0
