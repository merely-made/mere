# chrome

Host-neutral browser chrome state for the [mere](https://crates.io/crates/mere)
family: location-bar classification and suggestions, the omnibar search
session, toolbar draft and load state, focus authorities and overlay return
targets. Rendering and effects stay host-side.

Package name is `mere-chrome`; the library is `chrome`. Its dependencies are
`backstep`, `kernel`, `serde` and `url`, so every module is WASM-clean and
testable without booting a host.

## Modules

| Module | Public items |
| --- | --- |
| `authorities` | `GraphSearchAuthorityMut`, `FocusAuthorityMut` |
| `nav` | `NavTarget`, `classify`, `classify_with`, `resolve_href`, `History` (re-exported from `backstep`), `DEFAULT_COMMAND_SIGIL` |
| `omnibar` | `OmnibarSearchSession`, `OmnibarMatch`, `OmnibarSearchMode`, `OmnibarSessionKind`, `SearchProviderKind`, `HistoricalNodeMatch`, `ProviderSuggestionMailbox`, `ProviderSuggestionStatus`, `ProviderSuggestionError`, `ProviderSuggestionFetchOutcome` |
| `routing` | `ToolSurfaceReturnTarget` |
| `suggest` | `suggestions`, `match_label`, `resolve_match` |
| `toolbar` | `ToolbarState`, `ToolbarEditable` (alias `ToolbarDraft`) |

The crate root also exports `VERSION` and `STAGE`.

The frame model (`FrameViewModel`, `FrameHostInput`, `FrameStep`), its
`effects` and the `host_intent` shape were retired on 2026-11-08 with the
pre-pivot shell they served (engine-split D30); git keeps them.

## Where it is used

`ux-events` depends on this crate for `routing::ToolSurfaceReturnTarget` alone.
Turnstone uses `nav::History`, which now lives in `backstep`.
Command menus and their session state live in Hest.
