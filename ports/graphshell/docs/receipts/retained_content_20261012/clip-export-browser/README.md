# Selected clip browser qualification

The final scoped build was also qualified on fresh origin
`http://127.0.0.1:8883`; see `final/artifact-source.json`,
`final/ui-confirmation.json`, `final/trusted-handoff.jpg` and
`final/trusted-handoff.txt`. Wasm SHA256:
`e59e423f0dea91432d01650e29833f281c40e5cae43cb6fd2048ddb6ee4b71f7`.
The native aggregate now passes all 13 retained-content tests, including the
new short-panel allocation/scroll/click test, as reported by the coordinator.

At the normal 1037 by 639 viewport the final Pelt body was 127 pixels tall.
Real reader scrolling reached Export. A page-sized wheel gesture also chained
into the outer application layout after reaching the reader boundary, placing
its accessibility mirror off-screen and producing a transient black frame.
One upward scroll outside the reader restored the outer geometry while keeping
the selected content at its end. A trusted browser force click then activated
the visible Export control and produced the handoff status in the final
screenshot. The leaf currently prevents host wheel defaults only when its
document scroll actually changes; consuming gestures at a reader boundary is
a remaining input behavior decision. No source change was made in this pass.

The final pass used a bounded six-second download-event observation. No path
was returned, and the existing disk-capture gate remains open. No long download
investigation or internal-browser navigation was repeated. The final QA tab
was closed; the coordinator's query tab and user tabs were preserved.

Fresh full Wasm package on `http://127.0.0.1:8882`, including the reader sizing
repair, was tested in the existing Chrome browser 1 QA tab at its normal
1037 by 639 viewport. Root owns the package and server receipt in
`../../query_views_20261013/browser-artifact.json`.
The exact tested Wasm SHA256 was
`4e934145b217ad4edf638b6c66c743427c2eed00814c91c299e42425cb726a8e`
(490583667 bytes), and its JS SHA256 was
`85d28d21ddbf8f205975f3a61fdb42b621e1ab147e4504b5cc0e670c73740179`.
These identities are fixed for this receipt even if the root manifest changes.

`clip_export_ready.scn` opened the retained Coffee clip from the actual owner
store. The fixture is the Fleece selected package in `../text.mereclip`.

The original fixed 300-pixel reader placed Export below the visible short
tile. `short-viewport-before.jpg` and its DOM snapshot record that condition.
The repaired flex allocation gives the Pelt body 127 pixels after the header
and status. A trusted click in the live reader, followed by End on the real
`#gs-graphshell-canvas` keyboard receiver, scrolled Export to visible y=505.
Focusing the accessibility mirror before pressing End does not send the key
to the live canvas and is not the correct keyboard test route.

The visible Export target was activated with a trusted Playwright browser
mouse click (`force:true` because the accessibility mirror has no pointer
events; the underlying live canvas receives the input). The status changed to
`Selected clip download handed to the browser`, preserved in
`trusted-export-handoff.jpg` and its DOM snapshot. No console errors or
warnings were returned. No viewport override was applied.

The browser download event was armed before this click, but timed out after
20 seconds. A second trusted click with the event armed for 60 seconds also
timed out. No matching file appeared in `C:/Users/mark_/Downloads`.
Browser security policy rejected inspection of `chrome://downloads/` because
only HTTP and HTTPS navigation is allowed. No workaround was attempted.
The handoff status does not prove disk completion.
The documented in-app-browser qualification was attempted, but the browser
was unavailable. Native OS dialogs are unavailable in this session.

Expected download filename:
`selected-clip-b40c579d5234ed1f14d7512474a18c958c5e07f1a16f27afc42d337eeabd07ba.mereclip`.
The compiled browser adapter creates its Blob with
`application/vnd.mere.clip-package+json`.

The pending disk gate is deliberately separate. Run `verify_download.py`
with the actual completed download path when available. It verifies exact
selected body bytes, payload and retained-record BLAKE3, source provenance,
original source-tag evidence, exact UTF-8 selection and source hash, and
exclusion of private backing/custody/seals. It preserves the actual package
and writes `download-verification.json` only after those assertions pass.
`core-fixture-verification.json` separately verifies the existing Fleece
fixture's body and retained-record BLAKE3, exact source selection and original
tag evidence. It explicitly records that it is not an actual browser download.

After this browser pass, the coordinator requested a further stylesheet scope
repair: the bounded flex/hidden overflow now applies only to the
`retained-reading-pane` class, selected when a Pelt appearance is present.
Other reading panes retain their outer scrolling. The root will build that
change again; this receipt's screenshot proves the previous allocation repair
and is not relabelled as evidence for a later artifact.

The existing shared tests qualify sealed-access refusal, wire/body budgets,
missing identities, and exact package serialization. Native scope tests
qualify stale persona, graph session, access, content session and clip identity,
plus hidden producer refusal. These are automated guard receipts; this browser
fixture is unsealed and does not establish a protected-persona browser export.
Object URL release on owner/source/access retirement and expiry is implemented;
this browser pass does not claim an observed URL revocation event.
