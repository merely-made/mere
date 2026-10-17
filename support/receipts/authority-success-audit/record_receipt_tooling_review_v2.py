"""One-shot historical Graphshell receipt-tool source review checkpoint."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
NAME = 'receipt_tooling_review_receipt.json'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE / n).read_text(encoding='utf-8-sig'))
def save(n, value):
    (HERE / n).write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')

assert not (HERE / NAME).exists()
old = load('web_scenario_review_receipt.json')
head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
assert head == old['basis_commit']
guards = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}
assert len(guards) == 1319
assert all(sha(ROOT / n) == digest for n, digest in guards.items())
assert all(sha(HERE / n) == digest for n, digest in old['evidence_sha256'].items())
doc = ROOT / old['document']['path']
prefix = doc.read_bytes()
assert hashlib.sha256(prefix).hexdigest() == old['document']['sha256']
assert load('pending_read.json') == [] and not (HERE / 'pending_transition.json').exists()
inv, cov = load('source_inventory.json')['files'], load('read_coverage.json')
area = [n for n in inv if n.startswith('ports/graphshell/docs/receipts/')]
assert len(area) == 91
assert all(cov.get(n, {}).get('sha256') == inv[n]['sha256'] and cov[n]['through'] == inv[n]['chars'] for n in area)
whole = sorted(n for n in area if n not in guards)
assert len(whole) == 89 and sum(inv[n]['chars'] for n in whole) == 417532
assert all(sha(ROOT / n) == inv[n]['sha256'] for n in whole)
stats = json.loads(subprocess.check_output(['python', str(HERE / 'read_coverage.py'), 'status'], cwd=ROOT))
assert stats['complete_files'] == 1878 and stats['total_files'] == 3727
assert stats['read_chars'] == 22199337 and not stats['changed_since_inventory'] and not stats['pending_chunks']
probe = load('receipt_packaging_probe_v2_receipt.json')
assert probe['tests'] == 15 and len(probe['checks']) == 15
assert all(sha(ROOT / n) == digest for n, digest in probe['source_sha256'].items())
assert sha(HERE / 'receipt_packaging_probe_v2.py') == probe['diagnostic_sha256']
positive = [r for r in probe['checks'] if r['condition'] == 'zero-test-success']
assert len(positive) == 5 and all(r['actual_fixture_tests'] == 0 and r['returned_normally'] and r['new_source_labels_earlier_binary'] for r in positive)
assert sorted(r['claimed_cpu_tests'] for r in positive) == [68, 72, 72, 83, 88]
assert all(r['refused_before_writes'] for r in probe['checks'] if r['condition'] != 'zero-test-success')

notes, authority = load('review_notes.json'), load('authority_map.json')
assert notes['findings'][-1]['id'] == 'Q271' and authority['entries'][-1]['id'] == 'A689'
rows = [
    ('Q272', 'Five historical packagers manufacture fixed passing-test totals from successful command records',
     'ports/graphshell/docs/receipts/host_presenter_20261012/{main-050e7d542,main-8568f42fd,command-option-name-repair,option-mirror-repair,retired-hover-repair}/package.py:10,20-22',
     'The scripts assert the expected record count and zero exit codes, then assign CPU test totals of 68, 72, 72, 83 or 88 and true continuation flags. They do not derive those totals from successful, nonempty named tests. Executing each exact unchanged script with in-memory filesystem/process substitutes and the expected number of zero-test successful records returns normally and emits those positive claims. Ten controls with a failed exit or missing record correctly refuse before writes. These are rerun verification weaknesses; the probe does not establish that the dated original test totals were false.',
     'Keep immutable historical scripts and original receipts. A maintained future packaging gate should validate named gate identity and actual test results against its declared required cases before generating acceptance totals.'),
    ('Q273', 'Historical packagers relabel existing artifacts with current source without proving their build basis',
     'ports/graphshell/docs/receipts/host_presenter_20261012/{main-050e7d542,main-8568f42fd,command-option-name-repair,option-mirror-repair,retired-hover-repair}/package.py:11-22',
     'All five scripts consume existing Wasm artifacts, run bindgen/copy, then overwrite candidate source HEAD and source-byte hashes from the current checkout while setting packaging_complete and CPU/Wasm-qualified status. No checked build-basis record connects those artifact bytes with that source. The same five in-memory executions retain an earlier-A binary while successfully emitting newer-B HEAD/source labels. Bundle hashes describe the supplied bytes accurately; they do not establish which source built them. This demonstrates an attribution gap on rerun, not a proved mismatch in the original dated packages.',
     'Retain historical evidence. Future maintained packaging should require an immutable successful build record binding exact source, feature/target/toolchain configuration and produced artifact digests; reject a mismatch rather than refreshing source labels over existing binaries.'),
]
for ident, title, source, evidence, repair in rows:
    notes['findings'].append({'id': ident, 'title': title, 'source': source,
        'status': 'verified exact unchanged scripts with in-memory counterexamples and refusal controls',
        'evidence': evidence, 'verification': NAME, 'diagnostics': ['receipt_packaging_probe_v2_receipt.json'],
        'repair_triage': {'disposition': 'documented; historical evidence preserved; no production changes', 'reason': repair}})
arows = [
    ('Historical source copies', 'Dated receipt tools preserve their executed implementation and configuration.',
     ['host_presenter_20261012/*/package.py', 'viewer_budget_20261012/drivers*', 'query_views_20261013'],
     'Many copies are immutable experiment evidence, not competing live product engines. Do not deduplicate them in place or transfer their old acceptance to the current tree. Q272/Q273 concern unsafe rerun claims.'),
    ('Packager acceptance authority', 'Command success, test coverage, source identity and artifact identity need separate evidence.',
     ['five host_presenter package.py variants', 'receipt_packaging_probe_v2_receipt.json'],
     'Five zero-test counterexamples generate fixed positive test totals; five also label older binaries with current source. Failed-exit and missing-record controls refuse. No original historical result is disproved by these mocked counterexamples.'),
    ('Toolkit qualification ownership', 'qualify_artifacts.py checks retained gate output, image pairs and source records.',
     ['caller_owned_toolkits_20261013/final/qualify_artifacts.py', 'query_views_20261013/final_native_gates.ps1'],
     'The former parses actual named test totals for eight test gates and separately handles bench/boundary commands. Its supplier identity and same-device facts are read from supplied qualification records, not an independent GPU handle measurement by Python. Pixel comparison booleans are reported rather than universally asserted.'),
    ('Source freeze boundaries', 'Browser packaging scripts differ in how much of their source and drivers they freeze.',
     ['query_views_20261013/final_browser_build.ps1', 'viewer_budget_20261012/*/shipping*.py'],
     'The PowerShell build compares selected tracked extensions before/after; it does not establish all untracked or omitted extensions. Later shipping scripts add driver/auxiliary/frozen/served hash checks absent from earlier copies. Do not flatten their distinct coverage into one generic successful gate.'),
    ('Browser input evidence', 'Headed Playwright scripts drive browser mouse and keyboard APIs and inspect live DOM/AX state.',
     ['verify-physical*.cjs', 'verify-commands*.cjs', 'verify-presenter*.cjs'],
     'Physical-labelled scripts are automated browser input, not a new human or OS-device qualification. A separately requested navigator adapter proves adapter availability, not that the production renderer shares that device. External project-page routes are fulfilled by the test, not fetched from the live service.'),
    ('Diagnostic and refusal scope', 'Expected failure and diagnostic output have different acceptance meanings.',
     ['verify-keyboard-diagnostic.cjs', 'keyboard-diagnostic/*.scn', 'verify-budget*.cjs', 'verify-commands*.cjs'],
     'The keyboard diagnostic explicitly marks diagnosisOnly and need not refuse on a failing scenario. Later twelve-Tab input corrects an eleven-Tab fixture. Wheel planted controls require a particular failed assertion, while some unsupported-route tests accept fail without proving the refusal reason. No scenarios were rerun in this pass.'),
    ('Static receipt presentation', 'G1/G4/H4/N4 HTML renders dated receipt facts.',
     ['g1_loopback.html', 'g4_session_switch.html', 'h4_identity_surface.html', 'n4_owner_policy.html'],
     'Static Accepted/Live labels and data-intent buttons are not fresh product execution. G4 tab switching changes visible embedded readback. Their presentation-only role does not itself establish a dead product-action bug.'),
    ('Native receipt harness ownership', 'Small historical crates import product modules by path.',
     ['moot_resources_reopen_20261013/harness', 'node_ladder_20261011/harness', 'presentation_live_20261011/harness', 'presentation_rules_20261011/harness', 'relationship_dynamics_20261011/harness'],
     'These are narrow original-module harnesses, not independent copies of owner logic. Frozen supplier pins and patch tables record their basis and do not prove current consumer compilation.'),
    ('Content and source custody', 'Download verification and retained merge checks have explicit bounded contracts.',
     ['retained_content_20261012/clip-export-browser/verify_download.py', 'retained_merge_20261011/manifest.json', 'retained_content_20261012/turnstone-source-patches.toml'],
     'Download verifier checks an explicit file and selected source/body/privacy assertions, not actual HTTP media headers. Retained merge marks runtime_qualified false. Frozen absolute patch paths and two spellings of supplier URLs are historical configuration rather than a current single dependency authority.'),
    ('Lease cleanup evidence', 'Browser lease wrappers bound launch duration and signal their process groups.',
     ['viewer_budget_20261012/*/run-*.py', 'viewer_budget_20261012/*/shipping*.py'],
     'Cleanup marks closed after its final kill attempt without a final wait/re-poll in observed copies. This is a source-only reporting limitation; no surviving process or resource leak was reproduced and no lease was launched.'),
]
entries = [{'id': f'A{690+i:03d}', 'domain': d, 'claim': claim, 'implementations': impl,
            'assessment': assessment, 'state': 'whole-source review with historical and runtime limits', 'verification': NAME}
           for i, (d, claim, impl, assessment) in enumerate(arows)]
authority['entries'].extend(entries)
next_read = {'path': 'ports/graphshell/examples/caller_owned_toolkits.rs', 'start': 0,
             'reason': 'Read remaining native Graphshell examples, bins, scenarios and tests; then all other unread Mere source.'}
notes['current_area'] = 'All inventoried Graphshell historical receipt source/tooling'
notes['next_area'] = next_read['reason']
notes['checkpoint'] = {'classification': 'progress', 'classification_of_previous_turn': 'progress',
    'goal_status': 'active; full-source objective incomplete', 'coverage': stats,
    'finding_count': len(notes['findings']), 'authority_entries': len(authority['entries']),
    'new_receipts': [NAME, 'receipt_packaging_probe_initial_failure.json', 'receipt_packaging_probe_v2_receipt.json'],
    'production_edits_this_turn': False, 'next_read': next_read, 'pending_verification': None,
    'diagnostic_outcomes': 'Initial in-memory mock failed before completing a case; preserved. Repaired distinct probe passed 15 cases. No real build, package, browser, deployment or production file mutation.'}
diff = subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True, text=True)
assert diff.returncode == 0, diff.stdout + diff.stderr
with (HERE / 'receipt_tooling_diff_check.txt').open('x', encoding='utf-8') as stream:
    stream.write(diff.stdout + diff.stderr)
evidence = ['record_receipt_tooling_review.py', 'record_receipt_tooling_review_v2.py', 'receipt_tooling_checkpoint_initial_failure.json', 'receipt_tooling_diff_check.txt',
            'receipt_packaging_probe.py', 'receipt_packaging_probe_initial_failure.json',
            'receipt_packaging_probe_v2.py', 'receipt_packaging_probe_v2_receipt.json']
budget = sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file())
assert budget < 100_000_000
addition = f'''

### Historical receipt-tool source review, 2026-10-17

At {head}, all 91 inventoried files / 423,707 characters under ports/graphshell/docs/receipts are now completely read. This includes frozen scripts, native module harnesses, manifests, scenarios, HTML and lock patches; 89 files / 417,532 characters are additional to the prior checkpoint. Two harness files had been read earlier. Full coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters. The full Mere source review remains incomplete. All prior 1,319 protected source paths, three evidence hashes and the prior document prefix are preserved.

Q272: five host-presenter package.py copies accept the expected number of successful records and assign fixed totals of 68-88 passing CPU tests without proving those tests ran. The exact unchanged scripts, executed with entirely in-memory filesystem/process substitutes, accept zero-test fixtures and emit those positive totals. Q273: those executions also carry earlier-A binary bytes into a package labelled with newer-B source HEAD/hashes. Artifact hashing is present, but the scripts do not validate a build record connecting the bytes to the stated source. Both findings concern a rerun verification weakness, not proof that original dated test totals or artifacts were false. Five positive counterexamples and ten failed-exit/missing-record refusal controls passed. The first checkpoint recorder also stopped before mutation because it counted two previously read harness files as additional; the original recorder and failure record remain. The initial probe failed in its mock Path sorting before completing a case; it and its failure record remain, with a distinct repaired probe and receipt.

A690-A699 distinguish preserved historical copies from competing live authorities. Native receipt harnesses import original modules by path. Later driver revisions add synchronization, value assertions and input hashing; the copies do not all provide identical coverage. Static G1/G4/H4/N4 HTML is dated receipt presentation, not fresh authenticated product interaction. Physical-labelled tests call headed Playwright mouse/keyboard APIs; this source review does not qualify human or OS-device input. A fresh navigator adapter query does not establish shared production rendering handles. Project navigation is fulfilled by a test route rather than fetched from the external service.

The keyboard diagnostic is explicitly diagnosisOnly; retaining a failed scenario there is not a false acceptance. The corrected scenario uses twelve Tabs rather than eleven. Some negative tests prove only a fail result while wheel planted controls identify the exact failed assertion. The toolkit qualifier parses eight actual named test totals and treats bench/boundary commands separately; supplied same-device facts remain supplied evidence. The download verifier checks the explicit downloaded bytes and privacy assertions, not browser HTTP headers. Retained-merge custody explicitly disclaims runtime qualification. No historical script, package, log, source or original test was rewritten.

No actual build, browser, GPU, network or deployment ran. No audit-owned job is pending; no isolated target, Cargo home or worktree was created. The shared dirty main remains nine commits ahead and twelve behind the last fetched origin/main, with another Mere build live; upstream additions are not covered by this basis. The [receipt-tooling checkpoint](../../../support/receipts/authority-success-audit/{NAME}) binds the whole-file coverage, two findings, ten ownership entries and diagnostic evidence. Next is native Graphshell consumers and all remaining unread source. Goal remains active.
'''
doc.write_bytes(prefix + addition.encode('utf-8'))
save('review_notes.json', notes)
save('authority_map.json', authority)
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': head,
    'prior_receipt': {'path': 'web_scenario_review_receipt.json', 'sha256': sha(HERE / 'web_scenario_review_receipt.json')},
    'kind': 'Complete historical Graphshell receipt source review; full Mere objective incomplete',
    'coverage': stats, 'whole_current_files': whole, 'whole_current_chars': 417532,
    'source_sha256': {n: sha(ROOT / n) for n in whole}, 'preserved_production_and_test_hashes': guards,
    'completed_subtree': {'prefix': 'ports/graphshell/docs/receipts/', 'whole_files': len(area), 'chars': 423707},
    'new_findings': [r[0] for r in rows], 'authority_entries': [e['id'] for e in entries],
    'diagnostic_cases_passed': 15, 'initial_diagnostic_failure_preserved': True,
    'actual_browser_build_package_runs': 0, 'production_edits_this_pass': False,
    'pending_verification': None, 'next_read': next_read,
    'document': {'path': old['document']['path'], 'sha256': sha(doc), 'preserved_prefix_bytes': len(prefix),
                 'preserved_prefix_sha256': hashlib.sha256(prefix).hexdigest()},
    'evidence_sha256': {n: sha(HERE / n) for n in evidence}, 'retained_isolation': [],
    'receipt_budget': {'cap_bytes': 100_000_000, 'retained_bytes_before_receipt': budget},
    'limits': 'Exact historical packager code executed with mocked dependencies in memory; no original dated result disproved, no actual source-to-binary mismatch claimed, no production runtime qualification.'}
save(NAME, receipt)
print(json.dumps({'receipt': NAME, 'coverage': stats, 'new_whole_files': len(whole),
    'new_findings': receipt['new_findings'], 'authority_entries': receipt['authority_entries'],
    'protected_paths': len(guards), 'retained_bytes': budget}, indent=2))
