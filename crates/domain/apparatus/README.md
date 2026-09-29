# apparatus

Shared diagnostics home for [mere](https://crates.io/crates/mere).
Package `mere-apparatus`, library `apparatus`. The September 29 ruling assigns
bounded observations and inspection to this crate; the current implementation
is still only the peripheral system-inspector skeleton described below.

See the [diagnostics plan](../../../design_docs/mere_docs/implementation_strategy/2026-06-08_system_diagnostics_and_accessibility_plan.md)
for the accepted ownership boundaries and pending two-consumer implementation.

## API

| Item | Role |
| --- | --- |
| `project_skeleton() -> uxtree::UxTree` | Emits the v0 skeleton subtree. Takes no input. |
| `VERSION`, `STAGE` | Crate version string and lifecycle marker (`"pre-alpha"`). |

## Node shape

```text
apparatus (Role::Group, label "Apparatus")
  ├─ tracing events               (Role::Group, empty)
  ├─ register-diagnostics channels (Role::Group, empty)
  ├─ uxtree                       (Role::Group, empty)
  └─ accesskit                    (Role::Group, empty)
```

Node ids come from `uxtree::node_id_for_path`: `apparatus` for the root,
`apparatus/section/{label}` for each section. Ids are stable across runs.

## Dependencies

`accesskit`, `uxtree`, `tracing`.

## Status

Pre-1.0. Sections are empty placeholders. The bounded store, producer adapters
and correlated Mesquite receipts remain planned; this API does not implement
them or establish their acceptance gates.
