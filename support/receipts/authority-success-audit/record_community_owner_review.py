"""One-shot portable community/source-owner checkpoint and sync qualification."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
NAME = 'community_owner_review_receipt.json'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE / n).read_text(encoding='utf-8-sig'))
def save(n, v):
    (HERE / n).write_text(json.dumps(v, indent=2) + '\n', encoding='utf-8')

assert not (HERE / NAME).exists()
old = load('retained_editor_review_receipt.json')
head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
assert head == old['basis_commit']
guards = {**old['preserved_production_and_test_hashes'], **old['source_sha256'], **old['trace_source_sha256']}
assert len(guards) == 1051
assert all(sha(ROOT / n) == h for n, h in guards.items())
assert all(sha(HERE / n) == h for n, h in old['evidence_sha256'].items())
doc = ROOT / old['document']['path']
prefix = doc.read_bytes()
assert hashlib.sha256(prefix).hexdigest() == old['document']['sha256']
assert load('pending_read.json') == [] and not (HERE / 'pending_transition.json').exists()
inv, cov = load('source_inventory.json')['files'], load('read_coverage.json')
prefixes = ('ports/graphshell/src/capsule_applet', 'ports/graphshell/src/moot_resources',
    'ports/graphshell/tests/moot_resources', 'ports/graphshell/examples/support/moot_resources',
    'ports/moot/src/resource', 'ports/moot/src/coop.rs', 'ports/moot/Cargo.toml',
    'ports/moot/examples/moot_resource', 'ports/moot/docs/receipts/resource_',
    'ports/moot/tests/resource_owner.rs')
whole = sorted(n for n, v in inv.items() if n.startswith(prefixes) and n not in guards
    and cov.get(n, {}).get('sha256') == v['sha256'] and cov[n]['through'] == v['chars'])
assert len(whole) == 28 and sum(inv[n]['chars'] for n in whole) == 320785
assert all(sha(ROOT / n) == inv[n]['sha256'] for n in whole)
stats = json.loads(subprocess.check_output(['python', str(HERE / 'read_coverage.py'), 'status'], cwd=ROOT))
assert stats['complete_files'] == 1549 and stats['total_files'] == 3727
assert stats['read_chars'] == 21079058 and not stats['changed_since_inventory'] and not stats['pending_chunks']
portable = load('community_portable_probe_receipt.json')
assert portable['test_exit'] == 0 and portable['tests'] == 28
assert portable['new_counterexample_and_controls'] == 3 and portable['original_portable_tests'] == 25
assert portable['generated_output_removed'] and portable['fixture']['exit'] != 0
harness = load('community_portable_harness_compile_receipt.json')
assert harness['exit_code'] != 0 and not harness['generated_output_present']
terminal, sync = load('personal_sync_build_terminal.json'), load('personal_sync_probe_receipt.json')
assert terminal['session_id'] == 60676 and terminal['exit_code'] == 0 and terminal['build_finished']['success']
assert sync['test_exit'] == 0 and sync['tests'] == 5 and sync['generated_output_removed']
assert all(sha(ROOT / n) == h for n, h in {**portable['source_sha256'], **sync['source_sha256']}.items())
assert all(sha(HERE / n) == info['sha256'] for n, info in terminal['logs'].items())
for stem in ['authority-success-community-identity-probe', 'authority-success-community-portable-probe', 'authority-success-personal-sync-probe']:
    assert not list(Path('C:/t/cargo-targets/mere').glob(stem + '.*'))
assert not Path('C:/t/cargo-targets/mere/authority-success-community-original-fixture.rmeta').exists()
assert not Path('C:/t/cargo-targets/mere/authority-success-community-portable-harness.rmeta').exists()
notes, authority = load('review_notes.json'), load('authority_map.json')
assert notes['findings'][-1]['id'] == 'Q258' and authority['entries'][-1]['id'] == 'A643'
rows = [
    ('Q259', 'A resource reading forgets sharing-root immutability across an undisclosed refresh',
     'ports/moot/src/resource_coop.rs:Pane::refresh; resource_coop/tests.rs:a_disclosed_sharing_root_is_immutable',
     'The refresh guard compares sharing roots only when both old and new shared_by are Some. A current public Pane accepts Some(A) -> None -> Some(B) for the same contribution/manifest and keeps selection, although direct Some(A) -> Some(B) correctly returns ChangedContribution. Three native checks reproduce that gap plus direct-refusal and first-disclosure controls; the original 25 portable tests still pass. This is an unsigned disclosed-reading identity invariant, not a forged Gemot sharing operation or a demonstrated owner authorization bypass.',
     ['community_portable_probe_receipt.json'],
     'Retain enough immutable identity knowledge to distinguish withheld attribution from a newly unknown contribution, or explicitly define forgetting semantics and qualify the immutability claim. Do not treat a temporary undisclosed view as proof that a conflicting stable sharing root is valid.'),
    ('Q260', 'Graphshell community unit fixture omits a mandatory shared read field',
     'ports/graphshell/src/moot_resources.rs:tests::read; ports/moot/src/resource_coop.rs:ResourceRead',
     'The Graphshell unit fixture initializes ResourceRead without shared_by. The current shared struct requires that Rust field; serde(default) does not supply missing Rust literal fields. Compiling the original fixture function verbatim against the current portable type returns E0063 missing shared_by. This proves the fixture/type mismatch, not execution of the full Graphshell moot-resources feature suite.',
     ['community_portable_probe_receipt.json'],
     'Update the fixture to disclose or explicitly omit the sharing root, then compile and run the real feature tests. Keep synthetic unknown attribution explicit rather than inventing an author for the fixture.'),
    ('Q261', 'The checked-in portable community harness no longer resolves its child module',
     'ports/moot/docs/receipts/resource_coop_draft_20261011/harness/src/lib.rs; ports/moot/src/resource_coop.rs:mod control',
     'Direct compilation of the unchanged historical harness root against current source returns E0583: its path-attributed resource_coop inclusion resolves control beside resource_coop.rs rather than in resource_coop/control.rs. The first audit diagnostic hit the same module-plumbing failure and executed zero tests. Both failures are retained. A new distinct diagnostic rebases only module path attributes, then runs the actual portable core and original tests successfully. This is current harness reproducibility drift; it does not invalidate an older dated receipt at its original source revision.',
     ['community_portable_harness_compile_receipt.json', 'community_identity_preliminary_receipt.json', 'community_portable_probe_receipt.json'],
     'Repair current verification plumbing or clearly pin the harness to its historical source. Prefer an ordinary bounded portable package/test target that inherits canonical module layout over another brittle source-path inclusion.'),
]
for ident, title, source, evidence, diagnostics, repair in rows:
    notes['findings'].append({'id': ident, 'title': title, 'source': source,
        'status': 'native portable model reproduction' if ident == 'Q259' else 'verified current compiler diagnostic',
        'evidence': evidence, 'verification': NAME, 'diagnostics': diagnostics,
        'scope': 'Actual current portable source with only module path plumbing rebased for the passing diagnostic. Full signed owner, browser, custody and Graphshell feature suites were read where listed but not rerun.',
        'repair_triage': {'disposition': 'documented; no production changes under active goal', 'reason': repair}})
qualifications = {
    'Q250': ('native public-replica tag-reader mismatch reproduced', 'The synchronized raw Surface tag is present while current node_content_tags is empty on the same actual public replica.'),
    'Q251': ('native access-record collision reproduced; blob observation remains source-only', 'A causally later changed access fact under the same ID is accepted, discarded and not reported as a conflict; identical duplicate replay is the passing idempotence control. No blob-observation runtime counterexample ran.'),
    'Q252': ('native admitted missing-endpoint assertion failure reproduced', 'An accepted/stored missing-endpoint assertion prevents materialization, later node arrivals do not recover it, and a prior-endpoint assertion is the passing control.'),
}
for f in notes['findings']:
    if f['id'] in qualifications:
        f['initial_source_checkpoint_status'] = f['status']
        f['status'], detail = qualifications[f['id']]
        f['runtime_qualification'] = {'verification': NAME, 'receipt': 'personal_sync_probe_receipt.json',
            'basis_commit': head, 'detail': detail,
            'scope': 'Fresh successful personal-sync-feature build from original session60676. Public PersonalGraphReplica, signed synthetic operations and in-memory stores; no installed profile, network, encryption distribution or physical-disk trial.'}
        f['diagnostics'].append('personal_sync_probe_receipt.json')
notes['ruled_out'].extend([
    {'title': 'A capsule signature does not itself grant local execution or guest authority',
     'source': 'ports/graphshell/src/capsule_applet.rs:review/approve/accept/lower/open',
     'note': 'Pack verification delegates to Eidetic; exact component/inventory and disclosed capsule checks are local policy. Separate reviewed grants, instance/execution identity and rechecks guard lowering/body arrival. Unsigned disclosure is explicitly not community history.', 'verification': NAME},
    {'title': 'Portable collection-fork validation is not the final destination authority',
     'source': 'ports/moot/src/resource_owner.rs:dispatch/fork_offer',
     'note': 'The real owner compares the reviewed destination, current source version, current capability and destination history before authoring. The simpler validate_request helper alone does not establish an owner bypass.', 'verification': NAME},
    {'title': 'Community UI acceptance does not itself invent custody, hosting or participation',
     'source': 'ports/moot/src/resource_coop.rs:Pane::complete; resource_coop/view.rs; resource_view.rs',
     'note': 'Accepted outcomes change feedback only. Fresh snapshots supply state; UI labels distinguish offers, promises, observations and unknown reachability. The passing portable checks do not prove the signed owner or browser feature suites.', 'verification': NAME},
])
notes['candidates'].append({'title': 'Three local content adapters overlap but intentionally expose different policies',
    'source': 'ports/moot/src/resource_coop/owners.rs:local_content; resource_coop/handlers.rs:Owner::content; resource_owner.rs:public_local_bytes',
    'verification': NAME, 'note': 'All delegate byte verification to Eidetic but repeat local source/manifest/budget admission. The generic disclosed read assumes caller permission, the admitted public bridge checks member/current fauna and fixed 4 MiB limits, and the handlers adapter supports sealed custody plus configured powers and hosting. Track drift or common pure preflight candidates; do not merge their authorization/privacy policies or assert interchangeable consumers.'})
arows = [
    ('Capsule verification', 'CapsuleMount delegates signature checks to Eidetic and validates exact local inventory.', ['verify_pack', 'CapsuleMount::review'], 'Unsigned host disclosure is explicitly not a signed Gemot history or execution grant.'),
    ('Applet execution grants', 'Local reviewed grants and mount/execution identities gate proposals.', ['approve', 'accept', 'lower', 'open'], 'Queued actions and late bodies are rechecked. Public mutable descriptive fields still rely on supplying-host discipline.'),
    ('Capsule readings', 'Many view IDs share one already verified revision/body.', ['Readings::open/kept/close/clear'], 'View IDs do not repeat; retention feedback binds exact still-live reading/revision. Closing a view is not deletion.'),
    ('Resource read model', 'ResourceRef keeps contribution identity separate from identical payload bytes.', ['ResourceRef', 'Snapshot', 'Pane'], 'The snapshot is disclosed view data, never a credential or publication.'),
    ('Coop lifecycle reports', 'One serde report and conformance harness carry consumer-owned lifecycle facts.', ['LifecycleReport', 'Capabilities', 'conform'], 'Verdicts and declarations are checked; no peer presence, actual key rotation or durable activity authority is established by this data-only suite.'),
    ('Local request identity', 'Pane completion uses private Arc identity plus exact target and reader.', ['Request', 'is_pending', 'complete'], 'Cloned/relabelled or foreign-pane completions do not attach; completed UI feedback does not mutate disclosed owner facts.'),
    ('Sharing-root continuity', 'Refresh compares optional attribution only when both snapshots disclose it.', ['Pane::refresh', 'Option::zip'], 'Q259: an undisclosed intermediate snapshot loses the immutability guard; native counterexample and controls pass.'),
    ('Controller/view composition', 'Controller and Reading have separate outbox adapters over the same Pane.', ['resource_coop::control::Controller', 'resource_view::Reading'], 'Request preparation identity is genuinely shared. UI realization and queue plumbing still have two maintained compositions.'),
    ('Graphshell resource hosts', 'The browser token host and admitted native Mount are distinct adapters.', ['Host::prepare_dispatch/complete', 'Mount::poll/reopen'], 'Browser host binds reading/token/snapshot; native mount additionally checks retained service principal, signer, ledger and caller clock.'),
    ('Metadata read adapter', 'Generic owners::snapshot delegates action offers and content facts to Disclosures.', ['MootRoster', 'Disclosures', 'owners::snapshot'], 'It requires prior reader permission by its caller; fauna writer authorization is explicitly not reader admission.'),
    ('Public read/fork owner', 'resource_owner uses current Gemot membership, versions and command gates.', ['snapshot', 'dispatch', 'fork_offer'], 'Public local bytes only, bounded 4 MiB; unsupported custody/hosting/activity actions refuse. Fork references do not copy bytes.'),
    ('Custody/hosting handler', 'handlers::Owner delegates retention and signed hosting to established owners.', ['retention::retain/read', 'prepare_hosting', 'commit_hosting', 'check_hosting'], 'Configured powers, terms, public export admission, ceilings and sealing are distinct from the public-only bridge.'),
    ('Content admission overlap', 'Three adapters repeat local manifest/source/budget preflight around shared Eidetic verification.', ['local_content', 'public_local_bytes', 'Owner::content'], 'Different caller permissions, privacy/key support and budgets are real policies; single uniform reader authority is not established.'),
    ('Resource views', 'Two Cambium realizations share Pane but disclose different details.', ['resource_coop::view', 'resource_view'], 'Controller view exposes retention/terms; admitted public-only Reading view currently serves the narrower installed owner. Avoid claiming all richer operations are adopted by both.'),
    ('Original tests', 'Signed owner and durable-reopen tests were fully read without rerunning.', ['ports/moot/tests/resource_owner.rs', 'graphshell/tests/moot_resources*', 'moot_resources_fixture'], 'The fixtures use actual signed/sans-I/O admission and bounded backends. Reading them is not fresh headed, Redb, carrier or private-custody evidence.'),
    ('Verification drift', 'Graphshell unit fixture and portable source harness have current compile failures.', ['tests::read', 'resource_coop_draft harness'], 'Q260 E0063 and Q261 E0583 are separately reproduced; dated historical results are not rewritten.'),
    ('Portable diagnostic scope', 'The working diagnostic rebases only module paths and runs current portable tests.', ['community_portable_probe', 'community_identity_preliminary_receipt'], 'Initial E0583 ran zero tests and remains preserved; repaired diagnostic passes 25 original tests and 3 new public checks.'),
    ('Personal sync qualification', 'Fresh public-replica checks resolve the pending source findings.', ['personal_sync_build_terminal', 'personal_sync_probe_receipt'], 'Original build60676 succeeded; Q250-Q252 have 5 passing checks. Blob-observation collision remains source-only.'),
]
entries = [{'id': f'A{644+i:03d}', 'domain': d, 'claim': c, 'implementations': impl,
    'assessment': a, 'state': 'current complete source reading; runtime scope qualified per entry', 'verification': NAME}
    for i, (d, c, impl, a) in enumerate(arows)]
assert len(entries) == 18
authority['entries'].extend(entries)
next_read = {'path': 'ports/graphshell/src/bin/moot_resources_host.rs', 'start': 0,
    'reason': 'Read remaining real host composition and applet/owner consumers, then every other unread Mere source area.'}
notes['current_area'] = 'Portable capsule/community owners, resource adapters/views/tests and native identity/sync qualification'
notes['next_area'] = next_read['reason']
notes['checkpoint'] = {'classification': 'progress', 'classification_of_previous_turn': 'progress',
    'goal_status': 'active; full-source objective incomplete', 'coverage': stats,
    'finding_count': len(notes['findings']), 'authority_entries': len(authority['entries']),
    'new_receipts': [NAME, 'community_portable_probe_receipt.json', 'community_portable_harness_compile_receipt.json', 'personal_sync_build_terminal.json', 'personal_sync_probe_receipt.json'],
    'production_edits_this_turn': False, 'next_read': next_read, 'pending_verification': None,
    'diagnostic_outcomes': '33 passing native tests: 25 original portable checks, 3 new public Pane checks, 5 personal-sync checks. Two expected compiler failures reproduce current fixture/harness drift.'}
addition = f'''

### Portable community owners and completed sync diagnostics, 2026-10-17

At {head}, this pass completely reads 28 files / 320,785 characters: CapsuleMount and Readings, both Graphshell resource hosts, the portable resource model/controller and both UI realizations, three content/owner adapters, lifecycle contract, original owner/reopen tests and fixtures, examples, feature manifest and historical source harnesses. Full coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters, with current hashes and no pending chunks. Full Mere review remains incomplete.

Q259 is natively reproduced: Pane::refresh guards shared_by only when old and new are both Some. Some(A) -> None -> Some(B) succeeds for the same contribution/manifest and preserves attention, while direct A -> B correctly refuses. Three public checks cover the gap, direct-refusal control and legitimate first disclosure. The 25 original portable tests also pass, including the existing direct immutability check. This is disclosed-reading identity loss, not forged signed community authority. Withholding attribution must not silently prove a conflicting stable sharing root valid; forgetting/retention semantics need an explicit owner decision.

Q260 is a verified fixture/type mismatch: the Graphshell resource unit fixture's original ResourceRead literal lacks required shared_by. Its verbatim function compiled against the current type fails E0063. Q261 separately verifies current harness drift: direct compilation of the unchanged checked-in portable source-contract harness fails E0583 resolving control. The first audit attempt hit the same module-plumbing error and ran zero tests; its source, runner, log and preliminary receipt remain preserved. A new distinct diagnostic rebases only the control/test module path attributes and successfully runs current behavior and original tests. Neither failure is a claim that an older dated receipt failed at its original source revision. The complete Graphshell feature suite was not compiled or run here.

The original personal-sync library build60676 has now returned terminal exit0 with build-finished success. Final log hashes are bound by personal_sync_build_terminal.json. Its preserved prepared runner then completed all 5 public checks. Q250 now has native evidence that the raw synchronized Surface tag is absent from current Resource content-tag reads. Q251 reproduces causally ordered conflicting access facts being accepted and silently discarded, with identical duplicate replay as a passing control; blob-observation collision remains source-only. Q252 reproduces admitted missing-endpoint assertions failing materialization even after later nodes arrive, with valid prior endpoints as the passing control. These are signed synthetic operations and in-memory stores at current source, not installed-profile, transport, encryption-distribution or physical-disk acceptance.

The 33 passing native tests comprise 25 original portable checks, 3 new Pane checks and 5 personal-sync checks. There are also two deliberately expected compile failures for Q260/Q261, not two passed product test suites. The portable diagnostic imports current source with only module path plumbing rebased; actual signed owner/Redb/browser feature tests were fully read but not rerun. Native executable/PDB outputs were marker-verified and removed; compile-failure metadata outputs do not exist.

A644-A661 distinguish actual shared owners from adapter duplication. Capsule pack signatures and exact body/inventory checks delegate to Eidetic, while local review/grants and per-instance/execution proposal binding are separate. Multiple reading IDs share one verified body; retention feedback requires the exact still-live reading/revision. The unsigned catalogue is explicitly not community history. LifecycleReport is consumer-owned view data; the harness cannot itself qualify peer presence, key rotation or an actual activity implementation.

Community Pane centralizes private request identity and feedback. Controller and Reading retain separate UI/outbox compositions over that real shared core. Browser Host tokens and native admitted Mount have distinct responsibilities; the native mount verifies service, signer, retained authority, ledger and caller clock, and saves attention rather than credentials/payloads. Accepted feedback does not synthesize custody, hosting or participation. The real collection-fork owner rechecks destination/version/current capabilities; the simpler validate_request helper is not its final authority.

Three local content preflights coexist: generic owners::local_content assumes prior caller disclosure permission; resource_owner is an admitted current-member/public-only 4 MiB Open/fork bridge; handlers::Owner uses configured powers, terms, sealed custody and signed hosting policies/ceilings. All reuse Eidetic verification, but their manifest/source/budget admission remains duplicated. They are not interchangeable policies or one uniform reader owner. Both Cambium resource views share Pane but expose different details and currently consume different owner compositions. Track shared pure preflight opportunities without merging authorization/privacy decisions or claiming richer operations are adopted everywhere.

All 1,051 prior protected/traced production/test hashes, prior cited evidence and the canonical document prefix remain preserved. No production/original-test code changed. No isolated target, Cargo home or worktree was created. The stable Mere target remains shared with other active builds, so fetched upstream commits remain unmerged at this audit basis. The personal-sync wait is resolved; no audit-owned build or test remains pending. Next is moot_resources_host.rs and remaining host/applet consumers, followed by all other unread source. The [community owner checkpoint](../../../support/receipts/authority-success-audit/{NAME}) binds the full scope, findings and receipts. Full-source goal stays active.
'''
diff = subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True, text=True)
assert diff.returncode == 0, diff.stdout + diff.stderr
(HERE / 'community_owner_diff_check.txt').write_text(diff.stdout + diff.stderr, encoding='utf-8')
evidence = ['record_community_owner_review.py', 'community_owner_diff_check.txt',
    'community_identity_probe.rs', 'run_community_identity_probe.py', 'community_identity_probe_compile.log',
    'community_identity_preliminary_receipt.json', 'community_portable_probe.rs', 'run_community_portable_probe.py',
    'community_portable_resource.rs', 'community_portable_probe_compile.log', 'community_portable_probe_output.log',
    'community_portable_probe_receipt.json', 'community_current_fixture_compile.rs', 'community_current_fixture_compile.log',
    'community_portable_harness_compile.log', 'community_portable_harness_compile_receipt.json',
    'personal_sync_build_terminal.json', 'personal_sync_build.jsonl', 'personal_sync_build_stderr.log',
    'personal_sync_probe.rs', 'run_personal_sync_probe.py', 'personal_sync_probe_compile.log',
    'personal_sync_probe_output.log', 'personal_sync_probe_receipt.json']
evidence_hashes = {n: sha(HERE / n) for n in evidence}
budget = sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file())
assert budget < 100_000_000
doc.write_bytes(prefix + addition.encode('utf-8'))
save('review_notes.json', notes)
save('authority_map.json', authority)
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': head,
    'prior_receipt': {'path': 'retained_editor_review_receipt.json', 'sha256': sha(HERE / 'retained_editor_review_receipt.json')},
    'kind': 'Portable/community owner review; 33 passing tests and 2 verified compile defects; full Mere goal incomplete',
    'coverage': stats, 'whole_current_files': whole, 'whole_current_chars': 320785,
    'source_sha256': {n: sha(ROOT / n) for n in whole}, 'preserved_production_and_test_hashes': guards,
    'new_findings': ['Q259', 'Q260', 'Q261'], 'qualified_previous_findings': ['Q250', 'Q251', 'Q252'],
    'authority_entries': [e['id'] for e in entries], 'new_native_tests_run': 33,
    'native_test_breakdown': {'original_portable_tests': 25, 'new_portable_counterexample_controls': 3, 'personal_sync_checks': 5},
    'verified_expected_compile_failures': ['Q260 E0063', 'Q261 E0583'],
    'production_edits_this_pass': False, 'pending_verification': None, 'next_read': next_read,
    'document': {'path': old['document']['path'], 'sha256': sha(doc), 'preserved_prefix_bytes': len(prefix),
        'preserved_prefix_sha256': hashlib.sha256(prefix).hexdigest()},
    'evidence_sha256': evidence_hashes, 'retained_isolation': [],
    'receipt_budget': {'cap_bytes': 100_000_000, 'retained_bytes_before_receipt': budget},
    'limits': 'Portable source behavior uses only rebased module path plumbing. Original signed owner, Redb and Graphshell feature tests read but not rerun. Personal sync is public synthetic signed operations/in-memory evidence; blob collision remains source-only. No browser, live carrier, physical disk or custody trial.'}
save(NAME, receipt)
print(json.dumps({'receipt': NAME, 'coverage': stats, 'whole_files': len(whole), 'whole_chars': 320785,
    'new_findings': receipt['new_findings'], 'qualified_previous_findings': receipt['qualified_previous_findings'],
    'native_tests': 33, 'expected_compile_failures': 2, 'protected_paths': len(guards),
    'pending_verification': None, 'retained_bytes': budget}, indent=2))
