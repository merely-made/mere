# Batch 113: GUI catalog design record

## mere_docs/design/2026-10-26_gui_catalog_design.md

- disposition: current
- status line: "design record, 2026-10-26. Cues only: what the stack's existing catalogs cover, what a complete catalog would have to add, and the loops a catalog has to close. No ruling, no code, no work assigned. Amended the same day with §6, where the catalogs disagree with each other and with the tree, §7, Mark's rulings 1 to 5 on them, §8, his refinement of rulings 1 and 3, and §9, his rulings 6 to 8 on the cues §8 left." - accurate: yes
- claims checked: 58 - holds: 58, stale: 0, unverifiable: 0

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
