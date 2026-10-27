# chrome

The application's outer interface for the [mere](https://crates.io/crates/mere)
browser: toolbar, address bar / omnibar, focus authorities,
window frame. View-models and their state machines; rendering stays host-side.

Package name is `mere-chrome`; the library is `chrome`. Its four dependencies
are `forme`, `kernel`, `serde`, and `url`, so every module is WASM-clean and
testable without booting a host.

## Modules

| Module | Public items |
| --- | --- |
| `authorities` | `GraphSearchAuthorityMut`, `FocusAuthorityMut` |
| `frame_model` | `FrameViewModel`, `FrameHostInput`, `FocusViewModel`, `FocusRingSpec`, `FocusRingCurve`, `ToolbarViewModel`, `OmnibarViewModel`, `GraphSearchViewModel`, `DialogsViewModel`, `ToastSpec`, `ToastSeverity`, `DegradedReceiptSpec`, plus `settings`: `SettingsViewModel`, `FocusRingSettingsView`, `ThumbnailSettingsView`, `AccessibilityViewModel` |
| `host_intent` | `HostIntent`, the host to runtime intent shape |
| `nav` | `NavTarget`, `classify`, `classify_with`, `resolve_href`, `History`, `DEFAULT_COMMAND_SIGIL` |
| `omnibar` | `OmnibarSearchSession`, `OmnibarMatch`, `OmnibarSearchMode`, `OmnibarSessionKind`, `SearchProviderKind`, `HistoricalNodeMatch`, `ProviderSuggestionMailbox`, `ProviderSuggestionStatus`, `ProviderSuggestionError`, `ProviderSuggestionFetchOutcome` |
| `routing` | `ToolSurfaceReturnTarget` |
| `suggest` | `suggestions`, `match_label`, `resolve_match` |
| `toolbar` | `ToolbarState`, `ToolbarEditable` (alias `ToolbarDraft`) |

The crate root also exports `VERSION` and `STAGE`.

## Where it is used

Turnstone consumes Chrome directly for navigation. `ux-events` depends on this
crate for `routing::ToolSurfaceReturnTarget`. The unused `shell-state`
compatibility re-exports were removed; consumers import `chrome::*` directly.
Command menus and their session state live in Hest.
