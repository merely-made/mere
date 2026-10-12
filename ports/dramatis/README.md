# Dramatis

The first native window for persona selection and quarantine review, on
Mere's shared Cambium/Genet host. Djinn owns the vault and every review decision.

Run your Djinn resident first, then from the Mere workspace:

```powershell
cargo run --locked -p dramatis-app --bin dramatis-ui
```

`GRAPHSHELL_APP_ENDPOINT` selects the existing owner-local Djinn app door when
it differs from the default. The app creates no identity or credential files.
An absent resident leaves identity pending. A locked resident still supplies
its public roster, while choosing a persona and reading quarantine require
unlocking through Djinn's native surface.

The app selects existing personae, lists their public quarantined cards, and
asks Djinn to accept or permanently delete one exact item. Djinn displays the
native consent prompt, rechecks ownership, lock state and the item's revision,
and commits through Castellan. Cancellation leaves the item quarantined.
Unknown and malformed credentials, unsupported OTP digit counts, passkeys and
files remain held; their acceptance controls explain the reason.

The CXF importer records each explicitly chosen profile/item-namespace
association. Older unbound namespaces appear under “Older imports awaiting a persona”.
Choose their persona and request assignment; Djinn confirms the whole namespace
and rechecks its revision. Assignment does not accept its quarantined items. Persona creation, contacts,
credential fill, CXF export and distribution packaging are follow-on slices.

`python support/dramatis_review_receipts.py` reproduces synthetic owner-local
wire tests, a deliberately bypassed consent control followed by byte-exact
restoration, and native captures. Its fixture host is available only with
Djinn's `custody-receipt-host` feature, requires a new synthetic directory,
and never opens your actual vault. Capture environment variables are receipt
hooks, not custody or consent bypasses. See the canonical app plan for qualified
platform evidence and remaining gates.
