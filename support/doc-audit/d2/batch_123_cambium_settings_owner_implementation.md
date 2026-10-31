# Batch 123: Cambium composition settings owner implementation

## cambium_docs/implementation_strategy/2026-10-29_cambium_composition_plan.md

- disposition: current
- status line: "2026-10-29: P0 and P1 locally qualified." - accurate: yes
- claims checked: 10 - holds: 10, stale: 0, unverifiable: 0

### Stale claims

The prior P1 writer inventory remains a dated baseline at Turnstone source
`571c2f3c`. The current finding describes the later composition worktree and
does not rewrite that historical observation. P1's consumer implementation is
present and locally qualified by the retained real-provider Settings and
Arrange filters. P2–P6 remain open.

### Contradictions

None found within the documented scope. Shell startup creates the shared
in-process owner; the Settings pane Apply route uses its revisioned interface
and binding. The owner declares four domains: `appearance-choice`,
`shell-chrome`, `behaviors`, and `theme-catalog`. Theme library refresh and
explicit choice Apply remain separate. The implementation does not provide
cross-process compare-and-swap. The raw `setting_row` draft still resets when
the same setting's incoming value or control changes, so full dirty/conflicted
reconciliation remains open for P3.

### Recommended action

Keep P1 marked locally qualified within the stated offline source/consumer
scope. Preserve the four-domain scope, the library-versus-choice boundary, and
the explicit P3 draft-reconciliation limitation. Do not infer publication,
pin, cross-process, or headed acceptance from these results.

### Notes

Ten checked claims: the shared owner is installed at shell startup and passed
to Settings panes; Apply uses the revisioned binding; request admission is
bounded to eight pending tickets with client identity, generation and opaque
single-use tickets; the owner uses four named revision domains; successful
commits return canonical stored values; same-value successful explicit theme
Apply advances the choice revision while catalog refresh is separate; failed
writes do not publish the attempted value or revision; the owner guarantee is
limited to participating in-process callers; successful own Apply may adopt
same-value/control sibling guards only when each sibling still holds that
request's exact expected guard; and the actual guarded Settings and Arrange
qualification passed with zero failures. Source review covered the
Turnstone owner, live-settings handle, command route, shell startup, pane
construction and appearance editor. Evidence is retained under
`/Users/markik/Code/output/cambium-composition-p1-owner`: 61 guarded offline
Settings-filter tests and nine frozen-binary Arrange-filter tests passed, with
zero failures; source-size and diff checks passed. The frozen
`turnstone-lib-tests` SHA-256 is
`2aef06464467c3b79ebb55c3380b3a49e62713207d3b58359a556c6fa820e89c`.
No Meristem API changes, dependency publication/pin updates, cross-process CAS,
or headed qualification are claimed.
