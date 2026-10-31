# macOS Intel chrome check, 2026-10-27

Current gate: native frame screenshots await Screen Recording approval on imac-intel. Both native builds passed. Knot app-side assertions and process exit passed at1280 and640 logical pixels (34 presented frames). Woodshed app-side assertions passed at both sizes (26 frames), but the locked/pending profile has no writable store: its normal close-preservation policy kept the window open until harness termination. See close-policy-interpretation.json before interpreting this timeout.

The viewed PNGs are app render captures; they omit the native titlebar. Native traffic lights and complete frame visuals remain unqualified. Wide native window metadata reported1280x788 (1280x760 client content plus28 frame pixels); both attempted OS screenshots were refused.

Visual findings: Knot commands wrap into two rows at1280 and collapse to Menu at640, with context visible. Woodshed's640 pixel workspace/scope row truncates the owning-place label; CSS requests ellipsis, and the current label has no title tooltip. This is a layout finding, not a new patch.

Source and binary pins: preflight.json, resolved-sources.json, and per-attempt process.json. All failures preserved. No product source changes. Private profiles remain ignored under each app's .cargo/local/context-navigation-chrome.

Managed cache retained for this check: /Users/markik/Code/targets/lanes/context-navigation-chrome. Owner: this Mac chrome qualification; reuse when the Screen Recording gate opens. No isolated Cargo home or worktree was created. Authenticated SSH sessions and a loopback-only receipt-transfer helper remain available for continuation.
