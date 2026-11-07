# Native retained page text, headed receipt (2026-11-07)

Slice S6, Mere half. `run_native.py` drives the ordinary `graphshell_mere` window
against an isolated, passphrase-held Djinn. A local fixture server (which logs every
request) serves three pages. The pages are kept through the explicit one-address fetch with
the vault unlocked, the vault is then locked through its custody door, the resident
is cold-reopened, and a body-only goal is run through the door.

Result: ok (`qualification.json`). Raw captures, receipts and resident logs:
`/Users/markik/Code/testing/eidetic/native_retained_20261106/run6/evidence/`
(`captures.sha256` here). Host: Apple M4, macOS, headed, 1280x1180 logical, scale 2.

## Captures (each read by eye)
1. `01_off_nothing_kept.png`: the "Find in what I kept" panel, "Keep page text: off. Nothing is being kept."
2. `02_before_keeping_goal_finds_nothing.png`: goal "fog signal blasts" typed; "Nothing is retained to start from."
3. `03_three_pages_kept.png`: "on. 3 pages kept."; page chips for /harbour, /shoreline, /tides; "Kept the text of .../shoreline."
4. `04_after_keeping_goal_cites_body.png`: one cited result, /harbour, version dfaee5ce78d7, matched blasts, fog, signal, coverage 100%, with the words highlighted in the body passage. The goal is in no title or address.
5. `05_vault_locked_keeping_paused.png`: after the lock, a fresh viewer shows, in the amber state line, "Keep page text: paused. The vault is locked; page text is not being kept or read. Nothing is written until this clears." No pages are listed.
6. `06_locked_keep_refused.png`: a keep of /harbour attempted while locked is refused: "Keeping page text is paused. The vault is locked...". Nothing is written.
7. `07_cold_reopen_goal_cites_body.png`: a new resident process and a new viewer; "on. 3 pages kept." and the same cited /harbour body result, so the sealed text survived the cold reopen.

## Checks in `qualification.json`
The fixture server saw /harbour, /tides, /shoreline and then /harbour again (the keep refused
while locked: the viewer fetches before the resident refuses). Nothing else was requested, and
no link was followed. `retained.redb` holds no plaintext of the page text. Two custodian launches.

## Harness notes
- Door admission is by declaration plus the owner's yes; the harness writes those grants
  (`retained_receipt_tool grants`) into the isolated profile's settings.
- The owner stages the retained persona's wallet and key epoch (`retained_wallet`) and names
  it in `content.retained_persona`. Without that, the resident composes no sealing.
- The custodian answers "locked" for a persona with no staged key. The resident now reports that
  as "unlocked, but no sealing key is staged" (test:
  `an_unlocked_vault_without_a_sealing_key_does_not_read_as_locked`).
- An `await` that never matches now gives up after 20 s and reports `await-failed`; the harness fails on it.

## Not qualified
Physical keyboard (text is entered through the lane); other platforms; real user content;
replay protection of sealed values; Turnstone's side of the shared lane; keeping with no staged key
is refused at write time, not shown as a pause beforehand.
