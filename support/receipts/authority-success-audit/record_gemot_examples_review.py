"""Record complete Gemot example reads and the bounded late-return diagnostic."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
NAME = 'gemot_examples_review_receipt.json'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE / n).read_text(encoding='utf-8-sig'))

def save(name, value):
    (HERE / name).write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')

def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()

assert not (HERE / NAME).exists()
old = load('commons_complete_review_receipt.json')
head = git('rev-parse', 'HEAD')
assert head == old['basis_commit']
guards = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}
aux, runtime_inputs = old['auxiliary_read_sha256'], old['runtime_input_sha256']
assert len(guards) == 1962
assert all(sha(ROOT / p) == h for p, h in {**guards, **aux, **runtime_inputs}.items())
assert all(sha(HERE / p) == h for p, h in old['evidence_sha256'].items())
shared = old['shared_working_file_sha256']
doc = ROOT / old['document']['path']
prefix = doc.read_bytes()
assert sha(doc) == old['document']['sha256']
assert all(sha(ROOT / p) == h for p, h in shared.items() if p != old['document']['path'])
index = Path(git('rev-parse', '--git-path', 'index'))
index = index if index.is_absolute() else ROOT / index
index_hash = sha(index)
assert index_hash == old['git_index_sha256']
inv, cov = load('source_inventory.json'), load('read_coverage.json')
assert inv['commit'] == head and load('pending_read.json') == []
stats = json.loads(subprocess.check_output(['python', str(HERE / 'read_coverage.py'), 'status'], cwd=ROOT))
assert stats['complete_files'] == 2453 and stats['read_chars'] == 29249148
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
area = 'crates/moot/gemot/'
files = [p for p in inv['files'] if p.startswith(area)]
fresh = [p for p in files if cov.get(p, {}).get('through') == inv['files'][p]['chars']]
assert len(files) == 51 and len(fresh) == 21
assert sum(inv['files'][p]['chars'] for p in fresh) == 243811
assert all(cov[p]['sha256'] == sha(ROOT / p) == inv['files'][p]['sha256'] for p in fresh)
probe = load('capsule_late_return_probe_receipt.json')
assert probe['checks'] == 3 and probe['runner_sha256'] == sha(HERE / 'capsule_late_return_probe.mjs')
assert all(sha(ROOT / p) == h for p, h in probe['source_sha256'].items())
assert [c['case'] for c in probe['cases']] == ['control', 'revoke', 'stop']
assert [c['body_published_after_boundary'] for c in probe['cases']] == [False, True, True]
assert probe['cases'][2]['closed_workers'] == 1

notes, authority = load('review_notes.json'), load('authority_map.json')
assert notes['findings'][-1]['id'] == 'Q353'
assert authority['entries'][-1]['id'] == 'A999'
notes['findings'].append({
    'id': 'Q354',
    'title': 'Standalone capsule page publishes delayed navigation after grant revocation or completed Stop',
    'source': area + 'examples/capsule-library/browser.mjs:46,95,110,137,145',
    'status': 'reproduced unchanged complete page and shared Runtime in Node VM with explicit boundary doubles',
    'evidence': 'apply checks the current navigate grant before calling openAddress. That host continuation awaits storage, pack verification and body fetch, but its final check only compares an open token. Revoking the grant and completing Stop do not advance that token. A deferred valid body then sets current state, reader text, last_opened and success notice in both cases. Stop actually calls the unchanged shared Runtime.close and terminates the Worker double; the host fetch continuation is already outside its pending-call map.',
    'scope': probe['qualification'],
    'verification': NAME,
    'diagnostics': ['capsule_late_return_probe_receipt.json'],
    'repair_triage': {
        'disposition': 'documented under active read-only source audit',
        'reason': 'Bind host navigation to the applet instance and current grants through the final publication boundary; invalidate outstanding requests on Stop, replacement and revocation. Keep the existing new-open token check. Reuse the maintained Graphshell host lifetime contract rather than making shared worker cancellation stand in for host continuation cancellation. Delayed-body controls should exercise the standalone page too.'
    }
})

rows = [
    ('Gemot source assembly and package dependency ownership differ',
     'The manifest now states its transitive p2panda-net edge through Stickleback, while moot-peer still says Gemot owns no p2panda-net.',
     [area+'Cargo.toml', area+'examples/moot-peer.rs'],
     'Host-composed JoinedSpace is actual source-level assembly sharing. The older unqualified comment is misleading if read as dependency absence; do not erase the package graph distinction. No new cargo tree or dependency build ran.'),
    ('Offline publication proof keeps its candidate grammar explicit',
     'PublicationRevisionV1 and HostingCommitmentV1 are proof-local signed records with pinned fixture roots.',
     [area+'examples/author-offline-publication.rs'],
     'The example uses real authorized Moot commands, aggregate import/reopen and scoped blob authorization, while documenting candidate grammar, loopback, missing historical authority frontier and non-atomic aggregate import. These are bounded rehearsal owners, not a demonstrated replacement for the constitutional or storage owner.'),
    ('Community contribution and local applet permission are separate authorities',
     'Capsule contribution is governed in Gemot; applet execution uses separately reviewed local Servitor grants.',
     [area+'examples/capsule-library.rs', area+'examples/capsule-library/native.rs', area+'examples/capsule-library/guest/src/lib.rs'],
     'The guest only filters disclosed catalogue entries and requests actions. The host binds applet code identity and action scope independently of capsule authorship. Index-only community metadata excludes selected payload hashes; full host Standing and contribution authority are separately checked. This source pass does not rerun native or network proofs.'),
    ('Capsule builds reuse the maintained Graphshell applet runtime',
     'build.mjs copies worker, runtime, verification and WASI modules from ports/graphshell/web/applets.',
     [area+'examples/capsule-library/build.mjs', 'ports/graphshell/web/applets/runtime.mjs'],
     'This is genuine shared source ownership; there are no example-local copies of those four modules in the current Gemot inventory. Native and browser host adapters remain distinct implementations of their respective execution environments, rather than being proven equivalent by one catalogue comparison.'),
    ('Shared worker cancellation does not own host fetch continuation',
     'Runtime.close rejects pending guest turns, but the standalone page separately owns awaited openAddress work.',
     [area+'examples/capsule-library/browser.mjs', 'ports/graphshell/web/applets/runtime.mjs'],
     'Q354 reproduces a valid late body published after revocation and after completed Stop. The source comment that an emitted action cannot outlive a revoked local grant is broader than the final publication guard actually enforces.'),
    ('Browser and Graphshell revocation checks cover different cases',
     'browser-check revokes after opening has finished; graphshell-check explicitly submits late turns and bodies against revoked or replaced generations.',
     [area+'examples/capsule-library/browser-check.mjs', area+'examples/capsule-library/graphshell-check.mjs'],
     'A grant_revocation_refused result from the standalone checker does not establish delayed body rejection. The separate Graphshell checker has source cases for queued-action/body revocation and stale generations. Neither full Playwright checker ran in this audit pass; Q354 does not assert a current Graphshell host failure.'),
    ('Capsule exact-byte and contributor verification use existing pack owners',
     'model.rs delegates signature verdicts to Eidetic pack verification and checks contributor/part hashes and lengths.',
     [area+'examples/capsule-library/model.rs', area+'examples/capsule-library/browser.mjs', area+'examples/capsule-library/build.mjs'],
     'The browser imports the shared verify module and rechecks retained pack addresses and body parts. The three late-return diagnostics stub cryptographic and storage suppliers, so they provide host-lifetime evidence only, not independent signature or persistence acceptance.'),
    ('Native interruption success field accepts any returned error',
     'native.rs derives runaway_interrupted from on_event_blocking("spin", "").is_err().',
     [area+'examples/capsule-library/native.rs'],
     'The boolean alone does not discriminate watchdog/epoch interruption from another error. The browser spin handler instead checks the time-limit text. Record the native proof limitation and require a typed interruption cause before treating every error as that specific success; no native run or injected alternate failure occurred here.'),
    ('Prebuilt Graphshell artifact is not proven built from observed source hashes',
     'graphshell-build accepts supplied Wasm, runs wasm-bindgen, and hashes current source files into its receipt.',
     [area+'examples/capsule-library/graphshell-build.mjs'],
     'Produced binding hashes and observation-time source snapshots are useful identities, but this script does not establish compiler provenance from those source bytes to the supplied Wasm. Preserve that qualification when interpreting the copied standalone lock and sources map.'),
    ('Offline publication child cleanup has a narrower owner than capsule ManagedChild',
     'The author-offline example uses raw Child and existence-based readiness; capsule-library uses ManagedChild Drop and pending-file rename.',
     [area+'examples/author-offline-publication.rs', area+'examples/capsule-library.rs'],
     'Readiness timeout or JSON-read failure can return before stop/wait in the raw-child orchestration; unlike the capsule helper, Child Drop does not supply that cleanup. The main host-import result path explicitly cleans up the author before propagating its saved error, so do not claim every failure leaks. This pass ran no child process fixture.'),
    ('Current constitutional grant checks are distinct from host time and history',
     'CapabilityGrant checks subject, nonempty segment-bounded prefix and inclusive supplied-time interval; accepted rules supply current grants.',
     [area+'src/moot/constitution/event.rs'],
     'The event owner exposes rule commitments, quorum eligibility and optional unweighted Standing filters. It does not by itself establish current wall-clock sampling, accepted revision or a historical authority frontier. Continue the unread fold, store and service consumers before broad authority conclusions.'),
    ('Headed screenshot checks establish nonblank output within their stated limits',
     'graphshell-check combines real input/state checks with screenshots and simple pixel thresholds.',
     [area+'examples/capsule-library/graphshell-check.mjs'],
     'The source checker throws on failure and closes Chromium in finally. Its screenshot/RAF checks do not independently prove semantic pixel correctness or fresh display handoff. No headed/GPU execution or fresh screenshot was produced this pass.'),
]
entries = [{'id': f'A{1000+i}', 'domain': d, 'claim': c, 'implementations': p, 'assessment': a,
            'state': 'Gemot examples and initial event owner reviewed; three bounded Node VM audit cases', 'verification': NAME}
           for i, (d, c, p, a) in enumerate(rows)]
authority['entries'].extend(entries)
partial = {'prefix': area, 'files_read': 21, 'total_files': 51, 'chars_read': 243811, 'total_chars': 777720,
           'verification': NAME, 'basis_commit': head, 'remaining': [p for p in files if p not in fresh]}
notes.setdefault('partial_source_areas', []).append(partial)
notes['review_followups'].extend([
    {'verification': NAME, 'area': 'Native capsule interruption evidence', 'detail': 'Discriminate the actual interruption error before reporting runaway_interrupted from any error; source-only limitation, native guest not run this pass.'},
    {'verification': NAME, 'area': 'Offline publication readiness and cleanup', 'detail': 'Trace bounded readiness/error exits and reuse an actual child cleanup owner; distinguish the already-cleaned host-import result path. No child orchestration fault injection ran.'},
    {'verification': NAME, 'area': 'Graphshell artifact provenance', 'detail': 'Observed source hashes beside supplied prebuilt Wasm are not a compiler source receipt. Bind future build claims to actual compiler inputs instead of inferring from the packaging-time sources map.'},
])
next_read = {'path': area+'src/moot/constitution/fold.rs', 'start': 0,
             'reason': 'Continue unread Gemot constitutional fold, storage, delegation and aggregate service; full audit incomplete.'}
notes['current_area'] = 'Gemot examples complete; initial constitution event read; late standalone host publication reproduced'
notes['next_area'] = next_read['reason']
notes['checkpoint'] = {'classification': 'progress', 'classification_of_previous_turn': 'progress',
    'goal_status': 'active; full source incomplete', 'coverage': stats, 'finding_count': len(notes['findings']),
    'authority_entries': len(authority['entries']), 'new_receipts': [NAME, 'capsule_late_return_probe_receipt.json'],
    'audit_authored_production_edits_this_turn': False, 'next_read': next_read,
    'pending_verification': 'Q352 whole-owner key-replacement diagnostic remains unexecuted; Gemot native/headed/system proofs not rerun; no process live',
    'diagnostic_outcomes': 'Three bounded Node VM cases pass: ordinary opening control plus late publication after local navigate revocation and completed Stop. Actual unchanged standalone page and shared Runtime executed; explicit DOM/Worker/storage/network/crypto doubles.'}

diff = subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True, text=True)
assert diff.returncode == 0
with (HERE / 'gemot_examples_diff_check.txt').open('x', encoding='utf-8') as out:
    out.write(diff.stdout + diff.stderr)
addition = f'''

### Gemot examples and late standalone host returns, 2026-10-17

Read21new complete inventoried files /243,811characters at unchanged HEAD {head}. This covers the manifest, all current Gemot examples and their build/check/guest files, crate/artifact introductions and the constitutional event owner. Gemot remains partial at21/51files /243,811of777,720characters. Repository coverage is{stats['complete_files']:,}/{stats['total_files']:,}files /{stats['read_chars']:,}/{stats['total_chars']:,}characters, with no stale or pending read credit. The full audit remains active and incomplete.

Q354 reproduces a late navigation publication in the standalone capsule page. The host checks navigate permission before beginning openAddress, then awaits storage/pack/body work. Its final token check only guards newer opens. Revocation and completed Stop do not invalidate that token. A delayed body therefore updates current reader state, last_opened and the verified-success notice after either boundary. Stop really executes the unchanged shared Runtime.close and terminates its Worker double; the pending host fetch is outside that worker's pending-call owner.

Three fresh Node VM cases execute unchanged complete browser.mjs plus the shared Runtime: ordinary open control, navigate revoked while the body is deferred, and Stop completed while the body is deferred. DOM, honest Worker replies, absent-only IndexedDB, networking and signature/part verification are explicit doubles. This is host-continuation evidence, not actual Chromium/Wasm, cryptographic authenticity, real IDB, carrier or headed acceptance. It does not establish a bug in the separate Graphshell host. Its checker source explicitly covers delayed turn/body revocation and stale generations; the standalone checker revokes only after opening has completed and misses Q354.

A1000-A1011 record actual shared runtime/verification ownership, source assembly versus transitive dependencies, proof-local publication limitations, contribution versus local applet authority, the unshared host-lifetime guard, distinct checker coverage, existing signed-pack verification, overly broad native interruption evidence, supplied-Wasm provenance limits, raw-child readiness/cleanup, current constitutional grants and screenshot scope. The build actually copies shared Graphshell runtime/worker/WASI/verification source; there are no example-local fork copies of those four modules in the current inventory. The packaging-time sources map cannot alone prove that supplied prebuilt Wasm was compiled from those current sources.

Followups remain qualified: native runaway_interrupted accepts any error rather than a typed interruption cause; raw-child readiness/error exits lack the capsule example's ManagedChild cleanup; historical authority and aggregate import atomicity are explicitly open in the publication proof. No native, child, network or full Playwright proof was rerun here. Continue constitutional fold and actual authorization/storage/service consumers before claiming Gemot as a whole reviewed.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}) and [late-return diagnostic](../../../support/receipts/authority-success-audit/capsule_late_return_probe_receipt.json). No production edits, Git index changes or isolated build output. Existing source/runtime/auxiliary and shared-WIP hashes remain intact; all prior document bytes remain an exact prefix. Diff check passes. The diagnostic is terminal; retained evidence remains under100MB. Next: unread Gemot constitutional fold.
'''
doc.write_bytes(prefix + addition.encode('utf-8'))
save('review_notes.json', notes)
save('authority_map.json', authority)
evidence = ['record_gemot_examples_review.py', 'gemot_examples_diff_check.txt',
            'capsule_late_return_probe.mjs', 'capsule_late_return_probe_receipt.json']
budget = sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file())
assert budget < 100_000_000
receipt = {
    'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': head,
    'kind': 'Gemot examples and initial constitutional event source; late standalone host publication; repository incomplete',
    'prior_receipt': {'path': 'commons_complete_review_receipt.json', 'sha256': sha(HERE / 'commons_complete_review_receipt.json')},
    'coverage': stats, 'partial_source_areas': [partial], 'whole_current_files': fresh,
    'source_sha256': {p: sha(ROOT / p) for p in fresh}, 'auxiliary_read_sha256': aux,
    'runtime_input_sha256': runtime_inputs, 'preserved_production_and_test_hashes': guards,
    'shared_working_file_sha256': shared, 'new_findings': ['Q354'], 'authority_entries': [e['id'] for e in entries],
    'audit_authored_production_edits_this_pass': False,
    'diagnostics': {'capsule_late_return_probe_receipt.json': {'run_exit': 0, 'checks': 3, 'audit_cases': 3, 'original_tests': 0}},
    'next_read': next_read, 'pending_verification': notes['checkpoint']['pending_verification'],
    'document': {'path': old['document']['path'], 'sha256': sha(doc), 'preserved_prefix_bytes': len(prefix),
                 'preserved_prefix_sha256': hashlib.sha256(prefix).hexdigest()},
    'evidence_sha256': {p: sha(HERE / p) for p in evidence}, 'git_index_sha256': index_hash,
    'retained_isolation': [], 'receipt_budget': {'cap_bytes': 100_000_000, 'retained_bytes_before_receipt': budget},
    'limits': probe['qualification'] + ' Whole Gemot and repository source review remain incomplete.'
}
save(NAME, receipt)
assert sha(index) == index_hash and doc.read_bytes().startswith(prefix)
assert all(sha(ROOT / p) == h for p, h in {**guards, **receipt['source_sha256'], **aux, **runtime_inputs}.items())
assert all(sha(HERE / p) == h for p, h in receipt['evidence_sha256'].items())
assert all(sha(ROOT / p) == h for p, h in shared.items() if p != old['document']['path'])
assert subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True).returncode == 0
print(json.dumps({'receipt': NAME, 'coverage': stats, 'new_findings': receipt['new_findings'],
                  'authority_entries': receipt['authority_entries'], 'protected_sources': len({**guards, **receipt['source_sha256']}),
                  'retained_bytes': budget}, indent=2))
