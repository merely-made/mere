# Dramatis

The native window for persona selection, local authentication and quarantine review, on
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
credential fill and CXF export are follow-on slices. Packaging follows the
Hocket/Luggage release path; see [RELEASING.md](RELEASING.md) for the bounded
native-host commands, signing and qualification gates.

`python support/dramatis_review_receipts.py` reproduces synthetic owner-local
wire tests, a deliberately bypassed consent control followed by byte-exact
restoration, and native captures. Its fixture host is available only with
Djinn's `custody-receipt-host` feature, requires a new synthetic directory,
and never opens your actual vault. Capture environment variables are receipt
hooks, not custody or consent bypasses. See the canonical app plan for qualified
platform evidence and remaining gates.

## Local authentication

Under **Authentication**, **Change passphrase…** asks Djinn to open its own
masked prompts for the current passphrase, the replacement and confirmation.
The action applies to this computer's existing vault. It preserves the vault
root, personae, protocol keys and stored credentials. The app receives only a
public completion receipt and retains the acknowledgement through background
refreshes. A disconnected request is unconfirmed; it is not shown as success.

Both wrapped-root storage and the portable encrypted-profile file support the
operation. The portable file authenticates and re-encrypts every profile in one
replacement, including legacy version-one files. Cancellation, a rejected
current passphrase, empty replacement and mismatched confirmation leave the
credential unchanged. A vault locked while its prompts are open refuses the
commit. This operation requires the current passphrase; forgotten-passphrase
recovery and updates to other computers remain separate capabilities.

The [macOS qualification receipt](../../support/receipts/dramatis-authentication-20261021/README.md)
records both formats, native prompts, failure controls and independent cold reopens.

Linux password prompts require an available masked graphical provider: Zenity,
Yad or KDialog in the resident's desktop session. Console, unmasked and unknown
providers are refused. Djinn invokes the selected Linux helper directly and
preserves the entered spaces, punctuation and Unicode through replacement and
reopen.

The Linux qualification receipts cover [Fedora with Zenity](../../support/receipts/dramatis-authentication-fedora-20261021/README.md)
and [Kubuntu with KDialog](../../support/receipts/dramatis-authentication-kubuntu-20261021/README.md).
Both record exact input preservation, native cancellation and retry, failure
controls and cold reopens for both vault formats. The Kubuntu run reuses the
verified Fedora-built Linux executables; build provenance is recorded separately
from runtime qualification.

The [Windows qualification receipt](../../support/receipts/dramatis-authentication-windows-20261022/README.md)
records masked Win32 prompts, exact input and cold reopens for both formats,
the shared password-window reuse repair, and separate current-stack integration
checks against the native-changed fixture vaults.
