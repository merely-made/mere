# Xilem provenance and patch ledger

> **Historical note (2026-09-05):** The recorded revisions and replay describe
> extraction provenance at the audit point. They are not a current upstream
> alignment claim; re-check live forks and patches before acting on this ledger.

## Recorded bases

- Serval extraction source: `6b955ff96ed8b2912d04f7a36a85a36b401bb780`
- `mark-ik/xilem` main at audit: `5d72ad41eb660fa620110e045d332fd95684ebae`
- `linebender/xilem` main at audit: `c5950bcb03d4f3d187a20d1159f6aa276fd056bf`

Meristem began as Linebender's Apache-2.0 `xilem_core`, vendored into Serval at
`10b557c3d27003288bd54b86bb5225b4d8127e82`. The extraction repository replays
the four Serval commits that touched that subtree, preserving their authors,
dates, and messages before moving the files to `crates/cambium/meristem`.

The upstream side is a path-only replay of the 113 commits that touched
`xilem_core` through the recorded `mark-ik/xilem` base. The filtered tip is
`7f61f0537c2d911498cf0e7c940b377cb7673a76`; merge
`51a8a6a72fc1021ffcaa4c4d7a7ca5dbebddb7bf` joins that lineage to the
Serval-derived extraction without replacing Cambium's live tree. Patch replay
rewrites commit hashes but retains original authors, dates, and messages.

## Semantic patches over the vendored core

The initial Cambium patch set adds three defaulted `ElementSplice` operations:

- `hoist_pending`: preserve a backing node during a same-parent reorder.
- `extract_pending`: park a backing node without destroying it.
- `adopt_pending`: place a parked node under a new parent.

These operations support keyed and portable views in the Serval backend. Their
default implementations preserve compatibility for other Meristem backends.

### 2026-08-12 scope cut

Removed vendored surfaces with zero consumers across the family (cambium,
sprigging, hosts, turnstone, woodshed, mere, hocket, isometry, retinue):

- `views/fork.rs`, `views/run_once.rs` and `view_sequences/without_elements.rs`
  (side-effect views; the family runs side effects imperatively through
  app-level effect queues instead)
- `views/one_of.rs` with `OneOfCtx` and `PhantomElementCtx`, and its
  integration test
- `views/memoize.rs` (`Memoize` and `Frozen`), which had no family consumers
- `message_proxy.rs` (`MessageProxy`, `RawProxy`, `ProxyError`)
- `environment.rs` (`Environment`, `Provides`, `WithContext`, `Resource` and
  friends), including the `ViewPathTracker::environment()` hook upstream
  marked temporary, the `Environment` parameter of
  `MessageCtx::new`/`finish`, and Cambium's take/restore threading (G2.2)
- the now-orphaned `hashbrown` dependency (the `Environment` slot map was its
  only consumer)
- `docs.rs` (`DocsView`, `DocsViewSequence`, `Fake`, and `Nothing`); its only
  external reference was one lens doctest, now an explicitly illustrative
  example

Public trait count fell from 16 to 10. Reconciling these files against a
future upstream pull means re-evaluating, not re-vendoring: see
`docs/2026-08-12_meristem_scope_cut_and_component_contract_brief.md` at the
repository root for the rationale and receipts. The workspace now versions
`meristem` as 0.2.0 for the removed public API; registry publication remains
a separate release operation.

## xilem_web derivatives in Cambium

Cambium's Genet backend was templated on Linebender's Apache-2.0 `xilem_web`
(then `xilem_serval`). Audit base: `linebender/xilem`
`5d72ad41eb660fa620110e045d332fd95684ebae`, compared file by file on
2026-10-16. Files that share structure, names or logic with it keep the Xilem
Authors' copyright line above the MPL header, as Meristem's files do (Apache-2.0
section 4). This is notice-keeping, not legal advice.

| Cambium file (`crates/cambium/cambium/src/`) | Upstream (`xilem_web/src/`) | Class | Notice |
|---|---|---|---|
| `splice.rs` | `vec_splice.rs` (2023); `DomChildrenSplice` in `elements.rs` (2024) | Port: `VecSplice` keeps the struct, fields and the `new`/`skip`/`delete_next`/`insert`/`mutate`/`clear_tail` bodies; `GenetChildrenSplice` keeps `DomChildrenSplice`'s fields and `with_scratch` shape | 2023, 2024 |
| `optional_action.rs` | `optional_action.rs` | Port: near-verbatim `Action`, sealed `OptionalAction` and its three impls | 2023 |
| `text.rs` | `text.rs` | Port: same `impl_string_view!` / `impl_to_string_view!` macros, type list and `OrphanView` methods; `Pod`/`web_sys::Text` swapped for `GenetElement` | 2024 |
| `pod.rs` | `pod.rs` | Adapted: `SuperElement`/`AnyElement` impls and `replace_inner` follow `AnyPod`'s; `parent` field and `reborrow_mut` mirror `PodMut` | 2024 |
| `element.rs` | `elements.rs` | Adapted: `ElementState` fields and the build/rebuild/teardown/message splice flow follow `build_element`, `rebuild_element`, `teardown_element` and `message_element` | 2024 |
| `event.rs` | `events.rs` | Adapted: `OnClick` is a trimmed `OnEvent` (child, handler, `capture` and its builder, `with_id` marker id, `OptionalAction` message result) | 2023 |

Design influence only, with no upstream code carried over, so no Xilem
notice: `tags.rs` (a tag-name list over `el`; upstream uses `define_element!`
per tag), `context.rs` (`GenetCtx` is original; the `ViewPathTracker` impl is
dictated by the trait), `runner.rs` (original; upstream's `App` differs),
`pointer.rs` (original; upstream's `Pointer` view is a different design), and
the `HTML_NS` constant in `lib.rs`.

`key.rs`, `value.rs`, `hover.rs` and `wheel.rs` follow `event.rs`'s listener
shape and were not compared separately; they carry no notice. If a later
review finds shared code with `events.rs`, add them to this table.

## Update policy

Reconcile against a recorded Xilem release or commit. Compare the retained core
surface, update this ledger, and run the keyed and portable-move tests before
accepting an upstream change.

The `upstream-xilem` remote points to `linebender/xilem`. The `mark-ik/xilem`
fork is archived; its recorded base `5d72ad41eb660fa620110e045d332fd95684ebae`
is in upstream main's history. Fetching the remote does not merge the wider
Xilem workspace; updates are filtered to the core path first.
