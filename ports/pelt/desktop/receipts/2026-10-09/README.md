# Pelt Tabard adoption receipts

These runs launch the actual `pelt --workspace` application through the shared
macOS LaunchServices runner. The ordinary Appearance and Edit themes controls
open the shared workshop on Pelt's existing render core. Scratch settings and
definitions live under `/Users/markik/Code/tabard-app-receipts/2026-10-09/pelt`.
No application profile files are used.

## Preserved unsuccessful attempts

`fresh-01` used binary SHA-256
`4261b8579e726b533876e836159dbc02ca4179ca92c39e41528fdd5c3963c225`.
It installed the browser accessibility tree, logged three surface acquisitions
skipped as `Occluded`, and timed out without an editor receipt or capture.

`fresh-02` used binary SHA-256
`71ae4043b21d1076e314b66ddd7a45accc0168aed35f6ee9a08d3df2fa76c038`.
The product logged its primary window as visible and focused, eventually opened
the real editor, and produced initial, light and dark captures. It did not finish
the scenario, save a definition, or produce the final browser capture. The
native log reports the application's 120-second workflow deadline at stage 2.
The helper observed two PNGs when it wrote `process.json`; the third arrived
after that observation. Keep that original process record unchanged.

The included sample excerpt shows the main thread waiting in the ordinary
NSApplication/CFRunLoop. Source inspection then found that the native acceptance
driver waited for the first successful browser presentation but did not owe
its next browser frame. The retry requests continuation only for browser stages,
yielding to the editor's shared host while that tool window owns input.
The multiword scenario name was also an unsupported common-parser form; the
retry uses one token and explicitly asserts the edited name.

The shared runner's later canonical executable comparison handles the observed
`/var/folders` versus `/private/var/folders` alias when terminating its own timed
out process. These failures remain evidence of incomplete acceptance; their
partial PNGs are not a successful end-to-end receipt.
