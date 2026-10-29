# Batch 113: GUI catalog design record

## mere_docs/design/2026-10-26_gui_catalog_design.md

- disposition: current
- status line: "design record, 2026-10-26. Cues only: what the stack's existing catalogs cover, what a complete catalog would have to add, and the loops a catalog has to close. No ruling, no code, no work assigned. Amended the same day with §6, where the catalogs disagree with each other and with the tree, §7, Mark's rulings 1 to 5 on them, §8, his refinement of rulings 1 and 3, and §9, his rulings 6 to 8 on the cues §8 left. The §9 code landed 2026-10-27, read at `10fe98b50`; §9 closes with what it leaves. Since 2026-10-28 the rules are the doctrine's Surfaces area (statements 1 to 4) and the inventory stays here. §10, his ruling 9 of 2026-10-28: no Steward pane." - accurate: yes
- claims checked: 86 - holds: 86, stale: 0, unverifiable: 0

### Stale claims

None at authoring. Every code citation was read from the tree at `498fef4bf`
on 2026-10-26: `SurfaceId`'s twenty variants, `platen`'s `forme` and
`workbench` dependencies, Graphshell's `platen::` imports in `forme_session`
and `forme_workspace`, `ScopeNavigation<P, A>`, `setting_row`,
`dramatis::view::VaultLockView` in the door's custody client, the web host's
`a11y.rs`, inker's `SurfaceProducer` and `WebSurface`, `DocumentCapabilities`
in `document-lanes`, and `backstep` as `edit-history`'s new name. Doc citations
were read from the named sections the same day. The §6 amendment adds
fourteen more: `ToolPaneState` and `PanePresentationMode` in kernel `pane.rs`,
`DockState`/`DockSide` in the comms pane, pictograph's gloss module,
`crates/domain/apparatus`, `cartography::FormFactor`, the absence of
`Scope::Orrery`, `SurfaceId` emission counts, Graphshell's card identifiers,
the `workbench` crate description, the spine's projection list, and the
cabal, verso, engram, meerkat and Gyre identifier senses. The §9 amendment
adds twenty, read at `8d69c9144`: `ModManifest::provides` and its three
readers, the seven built-in manifests, `ModExtensionRecord`'s variants and
which are produced, `DiskModManifest`, eidetic's `PackManifest` fields, the
absence of a misfin fetch path, inker's `misfin` scheme rule, the registry's
middlenet list, `MisfinAdapter` over `MailboxStore`, `AddressKind`'s
variants, `CommsPane::select` and `open`, `SurfaceEngine` and
`WebSurface::capabilities`, `EngineRoutePolicy` and `SurfaceContractMode`,
the three `SurfaceEngine` implementors, the viewer registry's lack of an
external caller, document-host's capability check, the brief's clip ruling 4,
`crates/system/source-edit`, and the registry/inker dependency direction.
The 2026-10-27 closing of §9 adds twenty-one, read at `10fe98b50`:
`PresentationDeclaration` and its four parts, `EngineRoutePolicy::default`
from the declared readers, the three engines' declarations, `contributes` on
`ModManifest` and `DiskModManifest`, `Contribution`'s four kinds, Verso's
moved ids, `ModActivationError::undeclared` and its rollback, the viewer
registry's deletion with three features, the three renamed record variants,
the registry's `inker` dependency, the absent `misfin` scheme rule,
`misfin_correspondent`, `CommsPane::misfin_conversation`,
`comms_open::open_node`, the tree page's `comms_pane` and its three Forme
call sites, graphshell's `mere-comms` dependency, one-way verification, the
built-ins' empty operations, `ViewerRegistry` and `mod:core-viewer` as
dependency names, the unloaded inbox, and pelt's absent Comms path. The
2026-10-28 ruling in §5 adds one: the doctrine's Surfaces area, statements 1
to 4, read from `doctrine.md` the same day. The §10 ruling adds six, read at
`d4d5b0c32`: the panes doc's Steward definition and the 2026-06-09 axis, the
`StewardPane` variant with its descriptor and diagnostics channels, Cambium's
two module docs naming Steward rows, distillery's embeddable-half crate doc,
armillary as a separate repository holding `apparatus-core`, and the steward
device role in distillery's fixtures.

### Contradictions

None with current rulings. The record quotes Mark's 2026-10-25 settings
direction, which no Mere document had recorded before; it is marked as a first
citation. The "built, named, or designed" counts (which panes Graphshell and
Cambium name) are a reading of the source on one day and will drift.

### Recommended action

Keep as cues. When a GUI ruling lands, record it here or in TERMINOLOGY's
aspect table and update the status line; when a pane named only by
`SurfaceId` is built, correct §1's first reading.

### Notes

Cross-repository path `woodshed/crates/woodshed-core/src/settings.rs` is cited
by path per DOC_POLICY §5, not linked.
