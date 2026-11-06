# Native retained page text, headed receipt (2026-11-06) - NOT RUN

Slice S6, Mere half. `run_native.py` drives the ordinary `graphshell_mere`
window against an isolated, passphrase-held Djinn: it serves three fixture pages
from a local server that logs every request, keeps them through the explicit
fetch with the vault unlocked, locks the vault through the custody door and
shows the pause (including a refused keep), cold-reopens the resident, and runs
a body-only goal through the door. Seven captures are expected (`01` to `07`).

## Status
Not qualified. The machine's console session was locked (`CGSSessionScreenIsLocked`
= Yes, `screencapture` could not read the display), so the native window never
presented a frame ("native presentation stalled for 60000 ms, 0 presented
frames") and no screenshot exists. The raw failed attempt is in
`/Users/markik/Code/testing/eidetic/native_retained_20261106/run1`. Run again
with the console unlocked:

    T=target/debug
    python3 run_native.py --binary $T/graphshell_mere --djinn $T/djinn \
      --prepare $T/examples/coop_existing_graph --lock $T/examples/retained_receipt_tool \
      --output /Users/markik/Code/testing/eidetic/native_retained_20261106/runN

Build first: `cargo +1.97.1 build -p mere-resident --bin djinn --features passphrase-fd
--example coop_existing_graph --example retained_receipt_tool` and
`cargo +1.97.1 build -p graphshell-mere`.

## Harness notes
Door admission is by declaration with the owner's grant, so the harness writes the
owner's yes (Graphshell's declaration and a lock-only helper's) into the isolated
profile's settings, as printed by `retained_receipt_tool grants`.
