# Knot titlebar bounds repair

- Portable locked/offline `command_chrome`: 12 passed, 0 failed.
- Collapse preference migration/default: 1 passed, 0 failed. The default menu
  threshold is 1150px; a saved threshold remains user-controlled.
- Native rectangle diagnosis at 1000x700 logical: before the app-local
  reservation, caption controls painted at `(918,3,90,30)`, ending at 1008.
  After the 12px right-side reservation, they painted at `(906,3,90,30)`,
  ending at 996. Both caption and controls containment assertions passed.
  Headless and native action metrics differed by 12px; the reserved margin
  keeps the native group inside the actual viewport.
- Native full-menu / following-Preview no-image probe: passed in 9 frames;
  Preview opened and caption containment remained true.
- Native contextual-navigation baseline on the final binary: passed in 31
  frames with zero captures. Fixture SHA-256 unchanged:
  `eeb62129c64b381047b1fd12dfb57f215a0ff9baea2f1cb523437f17fe960b43`.
- Final visual capture:
  `../knot-native/portable-baseline-final/capture/nested-context-full-menu.png`.
  Parent visually accepted the Close outline containment. Its scenario receipt
  is `RESULT fail` because the isolated profile used the wrong JSON spelling
  for `menu-below`, and the Preview click used a selector unavailable in that
  compact mode. This image is visual evidence only, not a scenario pass.
- Final binary SHA-256:
  `395320bd8a6a99166eb0b29f35e77047621e512a5edab8bf3a700b5f084bf31cf`.
