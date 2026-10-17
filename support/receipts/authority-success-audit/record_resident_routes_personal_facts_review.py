"""One-shot whole resident-route, personal-facts and Dramatis reading checkpoint."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, re, subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
NAME = 'resident_routes_personal_facts_review_receipt.json'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE / n).read_text(encoding='utf-8'))
def save(n, value):
    (HERE / n).write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')

assert not (HERE / NAME).exists()
old = load('djinn_current_owner_review_receipt.json')
assert load('pending_read.json') == []
whole = ['ports/djinn/src/resident_mere.rs', 'ports/djinn/src/personal_facts.rs',
         'ports/djinn/src/sync/personal_seals.rs', 'ports/djinn/tests/common/inference_door.rs',
         'ports/djinn/tests/inference_automatic.rs']
inv, cov = load('source_inventory.json')['files'], load('read_coverage.json')
for prefix in ['crates/domain/personal-facts/', 'ports/dramatis/', 'crates/dramatis/persona-picker/']:
    whole.extend(sorted(n for n in inv if n.startswith(prefix)))
assert len(whole) == 20
assert all(cov[n]['sha256'] == inv[n]['sha256'] == sha(ROOT / n)
           and cov[n]['through'] == inv[n]['chars'] for n in whole)
assert sum(inv[n]['chars'] for n in whole) == 347072
stats = json.loads(subprocess.check_output(['python', str(HERE / 'read_coverage.py'), 'status'], cwd=ROOT))
assert stats['complete_files'] == 1250 and stats['total_files'] == 3722, stats
assert stats['read_chars'] == 16549229 and stats['total_chars'] == 39342110, stats
assert not stats['changed_since_inventory'] and stats['pending_chunks'] == 0
assert all(sha(ROOT / n) == inv[n]['sha256'] for n in cov if n in inv
           and cov[n]['sha256'] == inv[n]['sha256'])
src = [n for n in inv if n.startswith('ports/djinn/src/')]
assert len(src) == 69 and sum(inv[n]['chars'] for n in src) == 1559634
assert all(cov[n]['through'] == inv[n]['chars'] for n in src)
preserved = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}
assert all(sha(ROOT / n) == h for n, h in preserved.items())
assert subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip() == old['basis_commit']

route_log = (HERE / 'djinn_resident_mere_current_tests.log').read_text(encoding='utf-8-sig')
assert 'test result: FAILED. 22 passed; 1 failed;' in route_log
assert 'left: ["link.surface"]' in route_log
assert 'right: ["containment", "semantic", "semantic"]' in route_log
facts_log = (HERE / 'personal_facts_current_tests.log').read_text(encoding='utf-8-sig')
assert 'test result: ok. 7 passed; 0 failed;' in facts_log
reuse = load('djinn_resident_mere_reused_binary_receipt.json')
assert reuse['exit_code'] == 101

notes, authority = load('review_notes.json'), load('authority_map.json')
assert notes['findings'][-1]['id'] == 'Q215'
assert authority['entries'][-1]['id'] == 'A452'
finding = {
    'id': 'Q216',
    'title': 'The resident route lifecycle acceptance test still requires assertion-family rows after the host moved to pair-link projection',
    'source': 'ports/djinn/src/resident_mere.rs:the_route_adapter_takes_the_lifecycle_and_shows_the_route; ports/graphshell/src/mere_host.rs:projection; crates/cambium/mere-overmap/src/route.rs:graph_model',
    'status': 'current Cargo test failure reproduced; documented, not repaired',
    'evidence': 'The current offline default-feature resident_mere test run reports 22 passed and 1 failed. The lifecycle test inserts two semantic assertions and one containment assertion on one pair, then requires [containment, semantic, semantic]. The host deliberately emits link.surface/link.resource keyed by the link stratum; the route model forwards that disclosed kind. The assertion instead observes [link.surface] and aborts before the rest of that test body. A separately retained earlier-binary execution produced the same failure; it is not counted as a second independent test suite.',
    'scope': 'A stale acceptance assertion under the current source and shared WIP, not evidence that underlying assertion records were deleted. The link-model plan explicitly derives pair links while preserving separately attributed assertions. Other lifecycle, grant, archive and tamper tests passed; this failure does not invalidate their named checks or prove a headed rendering regression.',
    'repair_triage': {'disposition': 'documented in read-only audit', 'size': 'bounded test-contract refresh', 'reason': 'Align the route test with the current pair-link contract, and check independently addressable assertion preservation through the shared reading/material owner. Do not restore obsolete per-family scene rows or weaken the design merely to make this assertion pass. Keep the later lifecycle checks reachable.'},
    'verification': NAME,
    'diagnostic': 'djinn_resident_mere_current_tests.log',
}
notes['findings'].append(finding)
rows = [
    ('Resident source completion', 'Current Djinn source shares sessions through the resident.', ['ports/djinn/src/:69 files1559634 chars', 'MereShared shared host map'], 'All current Djinn src files are read, including the complete resident_mere test module. This is a source milestone, not full Djinn package or repository acceptance; two inference integration test files remain unread.'),
    ('Media binding refusal', 'The common route controller binds disclosed material to its admitted application and acknowledged position.', ['resident_mere media-fault tests', 'mere-overmap route_session media adoption'], 'Current original tests pass for wrong schema, app, attached session, epoch, revision, projection namespace, requested hash and byte address. Their fixtures rebind content hashes for envelope faults, so these are binding checks after valid hashing.'),
    ('Revocation and application isolation', 'Admission remains live beyond initial attachment.', ['AccessEndpoint live grants', 'ResidentMereProxy notice/resume', 'app-scoped saved views'], 'Current tests pass for idle notice revocation, refused later resume/snapshot, cross-app writes/material, ambient restrictions and inability of a product app to clear denials. No physical multi-device or headed run is inferred.'),
    ('Pair links versus old family-row oracle', 'Projection follows the current link owner while an older acceptance assertion retains another contract.', ['mere_host link.surface/link.resource', 'mere-overmap graph_model', 'resident route lifecycle assertion'], 'Q216 records the real current suite failure. The plan deliberately derives pair links; the stale assertion must not become authority to undo that design or a claim of lost durable assertions.'),
    ('Personal fact storage ownership', 'The domain crate transforms candidates; Djinn owns files, grants and the journal.', ['mere-personal-facts validate/validate_for_persona', 'PersonalFactsClient edit', 'resident-mere domain validator'], 'No second disk-backed personal-facts store is created. Seven fresh domain tests and the shared two-app resident test pass. Content-addressed corrections retain prior records, exact subject associations and replayable deltas.'),
    ('Repeated scoped graph decoder', 'PersonalFactsClient and the common route controller each decode session-card graph media.', ['ports/djinn/src/personal_facts.rs:refresh/resource', 'mere-overmap route_session Task::Media'], 'Both check schema/application/session/epoch/revision and use the common wire type, but independently traverse cards and adopt envelopes. This is a concrete consolidation seam despite shared durable authority. No forged-client acceptance or namespace bypass was reproduced.'),
    ('Borrowed personal graph protection', 'Sibling lanes borrow the sync host keyring rather than owning key epochs.', ['sync/personal_seals.rs', 'PersonalGraphSeals current_reader/admits/seal/open'], 'The host changes membership/epochs; the public handle exposes protection and current-seat checks. Historical readability and current compute eligibility remain distinct. The original current-reader unit test is read here, not newly executed by this checkpoint.'),
    ('Persona row sharing and product surfaces', 'The shared picker owns row labels/identity while Dramatis composes its own management surface.', ['persona-picker roster_items', 'Dramatis views persona rows', 'CustodyClient execute_at'], 'Dramatis reuses roster_items and forwards custody requests. Its independent buttons are a product surface, not another vault writer. The picker owns no vault mutation; source review does not establish suite-wide consumer adoption.'),
    ('Dramatis worker and evidence gates', 'The UI worker serializes requests and refreshes resident state.', ['worker serial request loop', 'State queue/adopt', 'ignored wire/native/package tests'], 'Failed refresh purges the snapshot, and request outcomes retain errors. The wire/native/package tests require explicit fixture gates and remain unread-as-execution: this pass reads them but runs none. The package helper delegates signed manifest generation/validation to Luggage; no package was published or installed.'),
    ('Automatic inference fixture scope', 'The fixture uses real resident routes, graph key agreement and supervisor loops on software devices.', ['tests/common/inference_door.rs', 'tests/inference_automatic.rs'], 'The ordinary path uses synthetic weights and three independent roots on one computer; the GPU case is ignored and explicitly requires external qualified libraries. This checkpoint reads the fixture without executing staged inference, key agreement or physical/GPU gates.'),
]
entries = []
for num, (domain, claim, implementations, assessment) in enumerate(rows, 453):
    entries.append({'id': f'A{num:03}', 'domain': domain, 'claim': claim,
                    'implementations': implementations, 'assessment': assessment,
                    'state': 'current source review; executed scope named in assessment',
                    'verification': NAME})
authority['entries'].extend(entries)
notes['candidates'].append({
    'title': 'Personal-facts refresh independently treats absence of a session-item source as an empty catalog',
    'source': 'ports/djinn/src/personal_facts.rs:PersonalFactsClient::refresh/catalog',
    'note': 'The client clears its envelope then returns success if no session-item source is found; catalog defaults to an empty graph. Trace legitimate no-session descriptors and malformed projection refusal before classifying a misleading success. It also independently decodes scoped graph media instead of the common route controller. No malformed-resident probe was run.'})
next_read = {'path': 'ports/djinn/tests/inference_route.rs', 'start': 0,
             'reason': 'Read all64543 characters and the remaining inference_gpu_reclaim.rs11749 characters; continue the wider source inventory afterwards.'}
notes['current_area'] = 'Whole current resident-mere, personal-facts domain/client, Dramatis app/picker and personal seals; automatic inference fixture source'
notes['next_area'] = 'Remaining Djinn inference integration tests, then unread Mere families'
notes['checkpoint'] = {'classification': 'progress', 'classification_of_previous_turn': 'progress',
                       'goal_status': 'active; full-source objective incomplete', 'coverage': stats,
                       'finding_count': 216, 'authority_entries': 462, 'new_receipts': [NAME],
                       'next_read': next_read, 'production_edits_this_turn': False,
                       'diagnostic_outcomes': 'Current Cargo resident-mere:22 pass1 fail; personal-facts:7 pass. Reused earlier test binary also22 pass1 fail, retained separately without double-counting.',
                       'coverage_note': '20 whole files347072 chars represented;251283 additional chars beyond the previous partial resident-mere checkpoint. All69 current Djinn src files complete; substantial repository source remains unread.'}
notes['source_qualifications'].append({'area': 'Whole resident routes and personal facts', 'note': 'The current Cargo route gate fails one stale per-family scene assertion after the intentional pair-link change. No durable assertion loss is inferred. Personal-facts domain tests pass. Dramatis/picker/automatic-inference fixtures were read without headed, package, staged runtime or GPU execution.'})

doc = ROOT / old['document']['path']
original = doc.read_bytes()
assert hashlib.sha256(original).hexdigest() == old['document']['sha256']
addition = '''

### Whole resident-route and personal-facts review, 2026-10-17

This checkpoint completes the current resident_mere.rs reread, all three personal-facts domain files, the Djinn personal-facts client and personal-seals adapter, all ten Dramatis app files, both persona-picker files, and the inference-door/automatic-inference fixtures. The 20 whole files contain 347,072 characters; 251,283 characters were added beyond the preceding partial resident-mere checkpoint. Current coverage is 1,250 of 3,722 source files and 16,549,229 of 39,342,110 characters. Every one of the 69 files under ports/djinn/src is now read at its current hash, totalling 1,559,634 characters. This does not complete the Djinn package or Mere audit. No stale acknowledged hash or pending chunk remains.

Q216: the original route lifecycle test retains an obsolete per-family projection assertion. A current offline Cargo run reports 22 passed and one failed: the test expects containment/semantic/semantic on one pair and observes link.surface. The host deliberately emits link.surface/link.resource from the link stratum, and the common route model forwards that disclosed kind. The link-model plan explicitly derives pair links while preserving separately attributed assertions. This is a test-contract integration defect, not evidence that three durable assertions disappeared. The failure aborts that test before its later lifecycle checks. Refresh the oracle around the current link contract and separately verify source assertion preservation, rather than restoring obsolete scene rows or declaring the entire gate green. Production and original tests were left unchanged.

The 22 passing route tests exercise actual memory-carrier admission and local broker paths, live grant revocation on idle notice intake, application-scoped views/material, ambient restrictions, archive/read-only behavior, merge choices and domain validation. Hash-valid wrong-role material and eight media-binding fault classes are refused. These are the exact named local automated checks; no physical multi-device or headed claim follows. An earlier compiled default test binary was also run while the fresh Cargo command waited; it gives the same 22/1 result and has its own artifact hash/source guard receipt. It is not counted as another independent acceptance suite.

Seven fresh personal-facts domain tests pass for revision preservation, exact subject links, replay, persona scope and tamper refusal. The domain crate transforms candidate graphs; the admitted client sends captured edits through the shared resident journal. It opens no second personal-facts file. There is still a duplicated implementation seam: the client separately traverses portable cards and validates scoped graph envelopes instead of consuming the common route decoder. Its absent-session-item path defaults the catalog to an empty graph; malformed versus legitimate empty projection behavior remains a qualified candidate, without a fabricated transport probe.

Dramatis reuses the shared roster-row builder and sends management requests through CustodyClient. Its worker serializes operations and its state drops a failed snapshot. The picker contains no vault writer. The read wire/native/package tests retain explicit ignored fixture gates; none ran here. The packaging helper delegates signed feed generation to Luggage; no install, publication or signing was performed. PersonalGraphSeals borrows current/historical keyring state from the personal-sync host, keeping current compute eligibility distinct from retained-history readability. The automatic-inference fixture is read, including synthetic CPU weights, three independent software roots on one computer, and an ignored GPU case. Reading it is not a new staged execution receipt.

Authority entries A453-A462 record these ownership and verification distinctions. The next unread Djinn fixtures are inference_route.rs (64,543 characters) and inference_gpu_reclaim.rs (11,749). Continue the wider inventory afterwards. Shared WIP and the entire prior document prefix were preserved. Stable shared Cargo output was reused; no isolated target, Cargo home, worktree or standalone generated fixture remains. Compact evidence stays within the 100 MB cap. The [checkpoint receipt](../../../support/receipts/authority-success-audit/resident_routes_personal_facts_review_receipt.json) binds coverage, sources, failed/passing gates and remaining scope.
'''
doc.write_bytes(original + addition.encode('utf-8'))
save('review_notes.json', notes)
save('authority_map.json', authority)
diff = subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True, text=True)
(HERE / 'resident_routes_personal_facts_diff_check.txt').write_text(diff.stdout + diff.stderr, encoding='utf-8')
assert diff.returncode == 0
evidence = ['djinn_resident_mere_current_tests.log', 'personal_facts_current_tests.log',
            'djinn_resident_mere_reused_binary.log', 'djinn_resident_mere_reused_binary_receipt.json',
            'record_resident_routes_personal_facts_review.py', 'resident_routes_personal_facts_diff_check.txt']
trace = ['ports/graphshell/src/mere_host.rs', 'crates/cambium/mere-overmap/src/route.rs',
         'crates/cambium/mere-overmap/src/route_session.rs',
         'design_docs/mere_docs/implementation_strategy/2026-10-15_link_model_plan.md']
budget = sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file())
assert budget < 100_000_000
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': old['basis_commit'],
           'kind': 'Whole resident routes and personal-facts source review; full Mere audit incomplete',
           'production_edits_this_pass': False, 'coverage': stats, 'whole_current_files': whole,
           'whole_current_chars': 347072, 'additional_read_chars': 251283,
           'djinn_src': {'whole_files': 69, 'chars': 1559634},
           'new_findings': ['Q216'], 'authority_entries': [v['id'] for v in entries],
           'source_sha256': {n: sha(ROOT / n) for n in whole},
           'trace_sha256': {n: sha(ROOT / n) for n in trace},
           'preserved_production_and_test_hashes': preserved,
           'prior_receipt': {'path': 'djinn_current_owner_review_receipt.json', 'sha256': sha(HERE / 'djinn_current_owner_review_receipt.json')},
           'next_read': next_read,
           'tests': {'resident_mere': {'passed': 22, 'failed': 1, 'exit': 101},
                     'personal_facts': {'passed': 7, 'failed': 0, 'exit': 0},
                     'reused_binary': {'passed': 22, 'failed': 1, 'exit': 101, 'counted_as_independent_suite': False}},
           'commands': ["cargo --config 'build.build-dir=\"C:/t/cargo-targets/mere/build\"' test -p djinn --lib resident_mere::tests --locked --offline --target-dir C:/t/cargo-targets/mere -- --test-threads=1",
                        "cargo --config 'build.build-dir=\"C:/t/cargo-targets/mere/build\"' test -p mere-personal-facts --lib --locked --offline --target-dir C:/t/cargo-targets/mere"],
           'document': {'path': old['document']['path'], 'sha256': sha(doc),
                        'preserved_prefix_bytes': len(original), 'preserved_prefix_sha256': hashlib.sha256(original).hexdigest()},
           'evidence_sha256': {n: sha(HERE / n) for n in evidence},
           'retained_isolation': [], 'receipt_budget': {'cap_bytes': 100_000_000, 'retained_bytes_before_receipt': budget},
           'limits': 'Full repository incomplete. No new headed, package/signing, GPU, staged-inference or physical multi-device run. One stale original-test contract documented; no production or original-test repairs.'}
save(NAME, receipt)
print(json.dumps({'receipt': NAME, 'coverage': stats, 'new_findings': ['Q216'],
                  'authority_entries': receipt['authority_entries'], 'retained_bytes': budget}, indent=2))
