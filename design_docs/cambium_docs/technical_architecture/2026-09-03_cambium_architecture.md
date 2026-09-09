# Architecture

Cambium is an application toolkit over Genet. Meristem produces and reconciles
view structure. Cambium translates that structure into Genet's neutral DOM,
custom-leaf, presentation, and document-engine seams.

The dependency direction is one-way:

```text
applications -> Cambium -> Genet seams -> rendering and platform

Genet engine crates -X-> Cambium
```

## Ownership

- Meristem owns reactive diffing, messages, view identity, and view sequences.
- Cambium owns application views, controls, composition, and Genet adapters.
- Sprigging owns retained custom-leaf state and arrangement helpers.
- Genet owns DOM, style, layout, paint, input, accessibility, and browser
  behavior.
- Nematic and other document engines own parsing and protocol-faithful lowering.

Genet remains independently usable without Cambium. Sprigging is an extension
of Genet's neutral custom-leaf seam, not a second layout or input engine.

The published seam crates use Genet package names. Cambium's public backend
types use `Genet*` names; deprecated `Serval*` aliases are temporary source
compatibility shims.

## Surface lifetime (2026-09-08)

Application leaf identity and renderer-issued fragment identity have different
lifetimes. The native host preserves the application leaf registry, retained
layout and state during suspension, but retires `leaf_fragments` at the same
point it drops the renderer. Resume registers the retained leaves with its new
renderer, even when their content epochs have not changed.

S8 source review found that the old cleanup in `sync_leaf_fragments` could not
run through a suspended redraw: `redraw` returns before that call when there
is no surface. Clearing at `suspended` repairs that missed invalidation. Two
independent source reviews verified the path and the focused patch passes
`git diff --check`. Native suspend/resume GPU execution and rendered-pixel
acceptance remain open; this is a source correction, not that receipt.
