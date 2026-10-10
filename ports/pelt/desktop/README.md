# Pelt desktop appearance

Run the browser workspace with `cargo run -p pelt -- --workspace PATH_OR_URL`.
The Appearance button opens Pelt's saved theme and mode controls. **Edit themes**
opens the shared Tabard workshop in a tool window using Pelt's existing render
core and device. Saving the workshop publishes a definition to the shared
library. Select that saved definition and a mode in Pelt's Appearance drawer to
apply it to browser Chrome. Previewing or saving a definition never changes
Pelt's selection implicitly; document content keeps its engine-owned theme.
An already applied definition stays visible for this browser session when the
editor saves changes under the same theme identity. Explicitly select the saved
theme or mode to adopt those changes. A new process resolves the latest saved
definition for its restored choice.

Pelt's application choice defaults to the platform configuration directory at
`mere/pelt/appearance.json`. Authored definitions use the independent shared
application-data library at `mere/tabard/themes.json`, also used by standalone
Tabard. `--appearance-store PATH` and `--theme-library PATH` override those
independently and imply the workspace. Existing valid Pelt choice files remain
valid. Missing definitions resolve visibly through the shared fallback without
rewriting the requested choice. Malformed choice/library files are preserved;
the Appearance notice explains the failure, and a corrupt library disables
editing until repaired. Canonical modes use Tabard's shared palette math;
authored mode sheets enter the Chrome cascade as authored CSS.

Workshop exports protect Pelt's active appearance file as well as the shared
library and workshop preferences. Choosing an alias of those files, including
symlinks or normalized paths, reports the protected destination and preserves
its bytes. Arbitrary other export destinations retain normal create and replace
behavior. Embedders supplying a durable store use
`WorkspaceViewerConfig::with_appearance_store_at` to identify its owned path.

Closing a dirty editor offers Save and close, Discard and close, or Cancel.
Closing the primary browser window while that editor holds work routes through
the same decision. Cancel preserves both browser and editor. A failed save keeps
the editor and primary window open with the failure visible.

## Native acceptance

Build with `cargo build --locked -p pelt --bin pelt`. On macOS, use the shared
LaunchServices runner so the product's own native windows receive normal
activation. Choose a fresh scratch directory; do not use application stores:

```sh
python3 scripts/run_macos_scenario.py \
  --binary target/debug/pelt --prefix PELT_APPEARANCE \
  --scenario ports/pelt/desktop/scenarios/appearance.scn \
  --output /tmp/pelt-appearance/fresh --timeout 120 \
  --env PELT_APPEARANCE_BROWSER_CAPTURE=/tmp/pelt-appearance/fresh/browser.png \
  -- --workspace --size 1180x800 \
  --appearance-store /tmp/pelt-appearance/settings.json \
  --theme-library /tmp/pelt-appearance/themes.json \
  ports/pelt/examples/workspace/p6-appearance/index.html
```

The acceptance driver clicks the ordinary Appearance and Edit themes controls,
then Mesquite drives the actual shared editor. Its scenario checks all four
canonical modes, edits a stylesheet, saves, resizes the same editor window to
640×780, and reveals the specimens through the shared scroll-into-view pointer
route. Windowless tests separately cover real wheel delivery. After closing the
editor, the driver clicks Pelt's saved definition and exact mode controls and
captures the browser through its existing compositor. It checks that preview
never applied implicitly and that the held controller identity, history, focused
tile and content aperture remain unchanged.

Run `appearance_reopen.scn` in a second process against those same scratch files
with a new output directory. Supply `PELT_APPEARANCE_EXPECT_CHOICE` with the exact
JSON contents of the scratch settings file to assert that the browser restored
its choice before opening the editor. Preserve every run's `scenario.done`,
`process.json`, `native.log` and PNGs. Acceptance requires `RESULT ok`, successful
presentation/nonblank evidence, and the final Pelt workspace assertion and frame
digest in the native log; the launcher exit alone is insufficient.
