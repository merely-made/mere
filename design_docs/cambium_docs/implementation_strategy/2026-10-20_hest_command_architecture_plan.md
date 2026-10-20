# Hest: one command architecture

**Date:** 2026-10-20.

**Status:** plan; nothing moved. `hest` 0.0.1 is published as a name reservation
(`crates/cambium/hest`); `command-menu` 0.1.0 is unchanged and published.

**Owner direction (Mark, 2026-10-20):** subordinate `command-menu` to `hest` and
centralize some form of command architecture under it. The name `hest` was picked
because the alternatives are taken on crates.io (`beck`, `beckon`, `behest`, `bid`);
`bidding` was free but generic.

Citations are `file:line` at Mere `2fead1143`; Turnstone's `action.rs` (in its `src` directory) was read from a clone of its `main`.

## Why: the pieces exist and live apart

| What | Where | Role |
|---|---|---|
| `Command`, `CommandChoices`, `CommandSet`, `MenuSession`, the shared id catalogue | `crates/cambium/command-menu/src/lib.rs:26`, `:70`, `:81`; `session.rs:13`; `catalogue.rs` | Declare a command and offer it: one order every surface shares, the person's kept and recent choices, a palette's open state. Serde is its only dependency. |
| `ActionId`, the typed form of the catalogue ids | `crates/graph/graph-kernel/src/actions.rs:21` | Graph hosts match on it instead of strings. |
| Key bindings keyed by catalogue ids | `crates/system/registry/src/input.rs:7` | Bind keys to shared commands without knowing the host. |
| The destructive-action gate probe | `crates/system/ux-events/src/ux_probes/destructive_action_gate.rs` | Reads a caller-supplied list of destructive `ActionId`s: the "destructive" fact lives in the probe's argument, not on the command. |
| Cambium's drawn commands | `crates/cambium/cambium/src/command_surface.rs:16` (`CommandItem`), `:84`, `:122`, `:131`; `command_menu_bar.rs:38` | Render and route a command's events. |
| Window verbs | `crates/cambium/cambium-rootstock/src/window_verbs.rs:51`, `:88` (`WindowCommand`, `WindowCommands`) | A queue the host drains after each dispatch. |
| The intent contract | `crates/chirograph/src/lib.rs:118` (`IntentReference`), `:123` (`IntentEffect`), `:1032` (`IntentInvocation`), `:1043` (`IntentResult`); `titulus.rs:277` (`ActionFormV1`) | An advertised action with an effect class, a payload schema, and a result that can be refused or stale. |
| Portable host intents | `crates/shell/chrome/src/host_intent.rs:47` (`HostIntent`) | What a host's chrome can express to the runtime. |
| Typed app intents | Turnstone `action.rs:116` (`Action`) | Keys, a later palette and automation produce them. Host-owned. |

The ids are already shared (`graph-kernel`, `registry`, Cambium and the graphshell port
all depend on `command-menu`). What is missing is the middle: nothing says that this
id is declared once, offered here, invoked with these arguments and this effect class,
and recorded as this outcome. The invocation edge exists in three shapes today: a typed
enum, a string plus payload bytes, and a queue of window verbs.

## Shape

One crate, `hest`, serde-only and kernel-free like `command-menu`. Arrows point into it:
Chirograph, Cambium, `graph-kernel` and hosts adapt to it, never the reverse.

- **`hest::menu`** is `command-menu`'s content, moved unchanged: `Command`,
  `CommandSet`, `CommandChoices`, `MenuSession`.
- **`hest::catalogue`** holds the shared ids. `ActionId` stays in `graph-kernel` as the
  typed form.
- **Declare (H2)** adds to `Command` an effect class (curation, domain truth, external
  effect, mirroring `IntentEffect`, duplicated rather than imported) and a `destructive`
  flag, so the gate probe reads the set instead of a list.
- **Invoke (H3)** adds `Invocation` (id, opaque arguments, schema id) and `Outcome`
  (done, refused with a reason, stale), shaped to what Chirograph already says.
  Chirograph converts to and from `IntentInvocation` and `IntentResult`; `hest` does not
  depend on it.
- **Stays out:** drawing (Cambium), binding storage (`registry`), enactment (a host's
  reducer or `Action` enum), permissions (Chirograph, Dramatis).

## Phases, each gated on a second host

The port law applies: a piece moves only when a second host exercises it. A pure move
(H1) needs no new behavior, only that every existing receipt still passes.

| Phase | Move | Gate (second host and receipt) |
|---|---|---|
| H0 | Reserve the name. | Done: `hest` 0.0.1 published 2026-10-20. |
| H1 | `command-menu`'s code into `hest::menu` and `hest::catalogue`. `hest` 0.1.0 publishes; `command-menu` 0.1.1 becomes a re-export of it (no yank; `cambium` 0.4.0 keeps resolving). Dependents repoint: `graph-kernel`, `registry`, `pandect`, Cambium, the graphshell port. | The existing tests pass unchanged: `command_menu_compact` in `cambium-genet-winit-host`, `graph-kernel`'s action tests, `registry`'s input tests. |
| H2 | Effect class and `destructive` on `Command`. | `ux-events`' gate probe reads them from the `CommandSet`; Turnstone registers its `Action`s with them. |
| H3 | `Invocation` and `Outcome`; window verbs and Turnstone's `Action` get ids. | Two hosts invoke through it: Turnstone's keys and Knot's window verbs; a Chirograph round trip proves the remote path. |
| H4 | Audit and script reuse. | Mesquite's scripted verbs and the Rhai block evaluator name the same ids. |

## Rulings for the owner

1. **Which ids does the catalogue hold?** It carries graph vocabulary (`node:new`,
   `relation:add`, `identity:*`, `physics:*`) beside generic ids (`view:*`,
   `session:*`, `workbench:*`). Recommendation: keep them together in
   `hest::catalogue` for H1, and move the graph-specific ids to `graph-kernel` only if a
   non-graph host objects; SE49's rule already says an id enters the catalogue when a
   second host offers the verb.
2. **Effect class: duplicate or depend?** Recommendation: duplicate the three-variant
   enum in `hest` and convert in Chirograph, to keep `hest` free of Chirograph's
   dependencies.
3. **Version at H1:** `hest` 0.1.0, as the first release with content.

## Risks

- **Scope creep.** Each phase has its second-host gate; without one, a piece does not
  move.
- **Import paths.** `command_menu::` appears in five dependent manifests and their
  sources; the re-export crate keeps them compiling during the move.
- **The Scenograph editor plan** (SE45 to SE50) names `command-menu` and the catalogue;
  those references update at H1, not before.

## Not now

No code moves in this plan's first step. `command-menu` and the cambium 0.4.0 release
are untouched.
