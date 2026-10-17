"""Checkpoint complete source reads, one scoped launcher case and unresolved metadata."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
NAME = 'mooting_illume_probes_review_receipt.json'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE / n).read_text(encoding='utf-8-sig'))
def save(n, v):
    with (HERE / n).open('x', encoding='utf-8') as f:
        json.dump(v, f, indent=2); f.write('\n')
def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()

assert not (HERE / NAME).exists()
old = load('mien_moothold_review_receipt.json')
head = git('rev-parse', 'HEAD'); assert head == old['basis_commit']
guards = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}
assert len(guards) == 2031
aux, runtime = old['auxiliary_read_sha256'], old['runtime_input_sha256']
assert all(sha(ROOT / p) == h for p, h in {**guards, **aux, **runtime}.items())
assert all(sha(HERE / p) == h for p, h in old['evidence_sha256'].items())
shared = old['shared_working_file_sha256']
doc = ROOT / old['document']['path']; prefix = doc.read_bytes()
assert sha(doc) == old['document']['sha256']
assert all(sha(ROOT / p) == h for p, h in shared.items() if p != old['document']['path'])
index = Path(git('rev-parse', '--git-path', 'index'))
if not index.is_absolute(): index = ROOT / index
index_hash = sha(index); assert index_hash == old['git_index_sha256']
inv, cov = load('source_inventory.json'), load('read_coverage.json')
stats = json.loads(subprocess.check_output(['python', str(HERE / 'read_coverage.py'), 'status'], cwd=ROOT))
assert inv['commit'] == head and load('pending_read.json') == []
assert stats['complete_files'] == 2532 and stats['read_chars'] == 30151417
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
areas = [('crates/moot/mooting/', 3, 10090), ('crates/nematic/illume/', 9, 64005), ('crates/probes/', 19, 135730)]
fresh = [p for p in inv['files'] if any(p.startswith(a) for a, _, _ in areas)]
assert len(fresh) == 31 and sum(inv['files'][p]['chars'] for p in fresh) == 209825
assert all(cov[p]['through'] == inv['files'][p]['chars'] and cov[p]['sha256'] == sha(ROOT / p) == inv['files'][p]['sha256'] for p in fresh)
probe = load('lexical_launcher_failure_probe_receipt.json')
assert probe['checks'] == 2 and probe['original_tests'] == 0
assert probe['runner_sha256'] == sha(HERE / 'run_lexical_launcher_failure_probe.py')
assert all(sha(ROOT / p) == h for p, h in probe['source_sha256'].items())
assert all(sha(HERE / 'probe-native-stubs' / p) == h for p, h in probe['stub_sha256'].items())
assert [(c['cargo_stub_exit'], c['launcher_exit'], c['node_reached']) for c in probe['cases']] == [(0, 0, True), (7, 0, True)]
paths = load('probe_dependency_paths_receipt.json')
assert paths['metadata_exit'] == 0
assert all(sha(ROOT / p) == h for p, h in paths['source_sha256'].items())
assert sha(HERE / 'inspect_probe_dependency_paths.py') == paths['runner_sha256']
assert sha(HERE / 'wing_three_paths_metadata_output.log') == paths['metadata_output_sha256']
lock = ROOT / 'crates/probes/wing-three-paths/Cargo.lock'
assert sha(lock) == paths['lock_before_sha256'] == paths['lock_after_sha256']
assert not (HERE / 'probe_dependency_resolution_receipt.json').exists()
assert not (HERE / 'wing_three_paths_dependency_resolution_output.log').exists()
parent_config = ROOT.parents[1] / '.cargo/config.toml'
assert parent_config.is_file()
timeout_name = 'probe_dependency_resolution_timeout_observation.json'
save(timeout_name, {
    'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': head,
    'kind': 'qualification of preserved shallow result and terminal metadata timeout; no retry',
    'prior_path_observation': {'path': 'probe_dependency_paths_receipt.json', 'sha256': sha(HERE / 'probe_dependency_paths_receipt.json')},
    'shallow_result': 'cargo metadata --offline --locked --no-deps exited0. The original runner then failed its incorrect assertion expecting nonzero. This does not resolve dependencies or prove buildability.',
    'full_resolution_attempt': {'runner': 'resolve_probe_dependency_paths.py', 'runner_sha256': sha(HERE / 'resolve_probe_dependency_paths.py'),
        'command': ['cargo', 'metadata', '--offline', '--locked', '--format-version', '1', '--manifest-path', 'crates/probes/wing-three-paths/Cargo.toml'],
        'observed_terminal_result': 'Python subprocess.TimeoutExpired after30seconds; unified execution session58060 terminal exit1. No Cargo resolver exit result, full-resolution receipt or output log was obtained.',
        'qualification': 'This records the terminal tool observation, not captured Cargo stderr or a reproduced compile failure. No cause of the timeout is established.'},
    'current_parent_config': {'path': str(parent_config), 'sha256': sha(parent_config), 'observation': 'comments only, no active parent paths override'},
    'source_sha256': paths['source_sha256'], 'probe_lock_sha256': sha(lock),
    'process_observation': 'Win32 process inventory after terminal timeout found no cargo/rustup/rustc command matching metadata or wing-three-paths. No owned resolver remains live.',
    'conclusion': 'Three literal nested Mesocosm manifests are absent. Full Cargo dependency resolution remains unverified. The first receipt buildability wording is overstrong and superseded by this qualification. Do not count an established build failure or rendering defect.'})

notes, authority = load('review_notes.json'), load('authority_map.json')
assert notes['findings'][-1]['id'] == 'Q365' and authority['entries'][-1]['id'] == 'A1047'
finding = {'id': 'Q366', 'title': 'Lexical Wasm launcher ignores Cargo failure and can still exit successfully',
    'source': 'crates/probes/mesh-lexical-wasm/run.ps1:13',
    'status': 'reproduced unchanged complete launcher with explicit child-only native command doubles',
    'evidence': 'Cargo double exits7; the Node double is still reached and the launcher exits0. A Cargo0 control also reaches Node and exits0. ErrorActionPreference Stop alone does not enforce the native exit result in this invocation.',
    'scope': probe['qualification'], 'verification': NAME,
    'diagnostics': ['lexical_launcher_failure_probe_receipt.json'],
    'repair_triage': {'disposition': 'bounded no-brainer candidate documented under active read-only audit',
        'reason': 'Check native exit status before running the next command and propagate failure. Keep artifact location tied to the actual build output; compare pinned canonical output if this launcher is intended as a determinism gate. No production edit authorized by the active goal.'}}
notes['findings'].append(finding)
rows = [
('Mooting now owns recognition evaluation, not storage',
 'The crate supplies a pure evaluator and policy/electorate fingerprint; storage moved to Stickleback.',
 ['crates/moot/mooting/src/lib.rs', 'crates/moot/mooting/src/recognition.rs', 'crates/moot/gemot/src/lib.rs'],
 'Gemot introductory storage-ownership prose is stale. Fraction/minimum validation and distinct eligible signer counting are actual code. ElectorateSnapshot is a public caller-supplied DTO, not proof of a signed revision; consumer provenance remains Q357. Five source tests were read, not run.'),
('Illume separates source highlighting from document meaning, with duplicated offset pairing',
 'Both highlighting entry points and the structural tree reuse Jotdown events but maintain parallel container/stack logic.',
 ['crates/nematic/illume/src/highlight.rs', 'crates/nematic/illume/src/tree.rs'],
 'Shared parser ownership is real; event-offset pairing and container grammar are repeated. Different style versus tree output can justify different passes. No semantic correctness failure is inferred merely from separate traversal.'),
('Illume injection has one registry but trusts the lexer-relative span contract',
 'Case-insensitive dispatch and host override precedence are centralized in InjectionRegistry.',
 ['crates/nematic/illume/src/injection.rs', 'crates/nematic/illume/src/pack.rs', 'crates/nematic/illume/src/pack/lua.rs', 'crates/nematic/illume/src/pack/web.rs'],
 'lex_at offsets trusted trait results without range validation. No hostile mod adapter was executed. The default Logos floors exist now despite older later-tier prose; coarse Lua/raw-string/comment handling is explicitly limited and is not a full language parser.'),
('Illume source output is not a semantic URL or headed editor receipt',
 'Entity regexes identify text shapes, and source tree/highlight tests include print-only diagnostics.',
 ['crates/nematic/illume/src/entity.rs', 'crates/nematic/illume/src/highlight.rs', 'crates/nematic/illume/src/tree.rs'],
 'False-positive-free URL prose overstates scheme/nonwhitespace matching. No navigation authority is supplied. Heading breaks/non-Str text and fold boundaries remain source candidates, not promoted bugs; no original test or headed editor ran.'),
('Wasm lexical probe shares the codec but repeats its fixture and only prints a receipt',
 'The Wasm module includes the native lexical codec by path; run.mjs prints length and digest.',
 ['crates/probes/mesh-lexical-wasm/src/lib.rs', 'crates/probes/mesh-lexical-wasm/run.mjs', 'crates/mesh/mesh/src/resources/lexical.rs'],
 'Native/Wasm three-string64-dimensional fixture is duplicated and currently matches. Native source pins801bytes and digest4d0f647fa7f15639aad9591ae9e466d48d9d029ef581d3a6d25ca10e5258aa01; the JS runner does not assert it. Historical source comments are not a fresh cross-engine execution receipt.'),
('The lexical launcher success does not establish a successful build',
 'Cargo and Node run sequentially without an explicit native exit check.',
 ['crates/probes/mesh-lexical-wasm/run.ps1'],
 'Q366 is executed with explicit doubles. Source uses a relative target path; current repo config sets build-dir, not target-dir, so that setting alone does not prove artifact mismatch. Real Cargo/Node/Wasm and stale artifact reuse were not executed.'),
('Direct PHY probe exercises owner admission, not graph GUI acceptance',
 'Tulle/Reticulum endpoint source binds certificates/hello proof and checks accepted then policy-disabled posts.',
 ['crates/probes/murm-direct-phy/src/main.rs', 'crates/probes/murm-direct-phy/Cargo.toml'],
 'Source assertions cover protocol acceptance/history and driver shutdown; HEADED PASSED refers to this native protocol harness. No hardware or visual graph runtime ran in this pass. Deterministic keys and defaults are probe fixtures, not deployed trust policy.'),
('Pack distribution proof signs/transfers metadata without installing referenced payload',
 'Both library and RF binary build a signed-pack fixture and use real crypto/resource and verification owners.',
 ['crates/probes/pack-distribution/src/lib.rs', 'crates/probes/pack-distribution/src/bin/rf_pack_pair.rs', 'crates/probes/pack-distribution/Cargo.toml'],
 'Fixture signing is duplicated. RF sends manifest/envelope data, not referenced main.lua bytes. Broken verdict assertion is not an actual install-refusal execution; curated Moot handoff exists only in prose here. Local Retinue paths currently exist. No RF or package test ran.'),
('Three graphics paths share workload/device but differ explicitly in pose',
 'One WorldWorkload/camera feeds Aplanar fragments, Bsprites and resident LiveBodyRenderer.',
 ['crates/probes/wing-three-paths/src/main.rs', 'crates/probes/wing-three-paths/src/world.rs', 'crates/probes/wing-three-paths/src/path_a.rs', 'crates/probes/wing-three-paths/src/path_b.rs', 'crates/probes/wing-three-paths/src/path_c.rs'],
 'B uses quarter-yaw while A/C are continuous, explicitly reported. Hash binds workload and frames, not path quantization. Device and owner reuse are real; broad visual equivalence is not established. A obsolete-strip cache candidate needs a controlled case before a finding.'),
('The graphics comparison oracle shares geometry inputs and reports measurement, not a pass threshold',
 'CPU per-pixel depth oracle shares camera/mesher/shading while A/B rely on painter-style paths.',
 ['crates/probes/wing-three-paths/src/project.rs', 'crates/probes/wing-three-paths/src/measurement.rs', 'crates/probes/wing-three-paths/src/main.rs'],
 'Oracle supplies an independent depth-order check, not an independent geometry implementation. RGB mismatch/interior metrics ignore alpha and do not fail main; unchecked swarm and sampled frames are explicit. Exit0 does not assert all rendering correct. Hardcoded2026-09-13 JSON date is not a fresh timestamp.'),
('Graphics cache and backend receipts have narrow declared contracts',
 'B keys content/facing/projection; C reuses resident owner/depth on one GPU device.',
 ['crates/probes/wing-three-paths/src/path_b.rs', 'crates/probes/wing-three-paths/src/path_b_tests.rs', 'crates/probes/wing-three-paths/src/path_c.rs', 'crates/probes/wing-three-paths/src/gpu.rs'],
 'Placement/generation exclusion is intentional canonical reuse; C capacity is prevalidated. Requested backend may fall back, but actual adapter/backend is printed and retained. None of these source tests or real GPU paths ran here.'),
('Literal dependency path inventory is distinct from resolved Cargo buildability',
 'Pack Retinue/Netrender local manifests exist; three legacy nested Mesocosm paths do not.',
 ['crates/probes/pack-distribution/Cargo.toml', 'crates/probes/wing-three-paths/Cargo.toml', '.cargo/config.toml'],
 'Shallow no-deps metadata exited0; its runner then failed an incorrect expected-failure assertion. Full resolution terminated with a Python30second timeout, not a Cargo diagnosis. Current parent Cargo config has no active paths override. Preserve both attempts and defer build-failure judgment; no new finding is counted.')]
entries = [{'id': f'A{1048 + i}', 'domain': d, 'claim': c, 'implementations': p, 'assessment': a,
            'state': 'source complete; executed launcher diagnostic separated from unexecuted library/hardware claims', 'verification': NAME}
           for i, (d, c, p, a) in enumerate(rows)]
authority['entries'].extend(entries)
completed = []
for area, count, chars in areas:
    selected = [p for p in fresh if p.startswith(area)]
    assert len(selected) == count and sum(inv['files'][p]['chars'] for p in selected) == chars
    item = {'prefix': area, 'files': count, 'chars': chars, 'basis_commit': head, 'verification': NAME,
            'qualification': 'Whole source reviewed; only the unchanged lexical launcher executed using explicit native doubles. No original Rust tests, full Cargo graph, Wasm, Node engine, GPU, radio or install execution.'}
    completed.append(item); notes['completed_source_areas'].append(item)
notes['review_followups'].extend([
    {'verification': NAME, 'area': 'Probe launcher/native success propagation', 'detail': 'Q366 is a bounded repair candidate. Digest comparison and artifact location require a stated determinism-gate contract; historical printed digest is not fresh proof.'},
    {'verification': NAME, 'area': 'Standalone graphics probe dependency resolution', 'detail': 'Qualify the first receipt overstrong buildability sentence using the timeout observation. Three absent literal paths are known; full resolution cause and compile result remain unverified.'},
    {'verification': NAME, 'area': 'Illume traversal and graphics cache candidates', 'detail': 'Trace actual host span layering and heading break expectations; control A obsolete-strip behavior before treating source candidates as reproduced defects. Different output ownership is not itself duplicated authority.'}])
next_read = {'path': 'crates/script/app-host/Cargo.toml', 'start': 0, 'reason': 'Continue full unread script app-host source and remaining repository areas.'}
notes['current_area'] = 'Mooting, Illume and all probes source complete; Q366 launcher false success scoped and dependency timeout qualified'
notes['next_area'] = next_read['reason']
notes['checkpoint'] = {'classification': 'progress', 'classification_of_previous_turn': 'progress', 'goal_status': 'active; full source incomplete',
    'coverage': stats, 'finding_count': len(notes['findings']), 'authority_entries': len(authority['entries']),
    'new_receipts': [NAME, 'lexical_launcher_failure_probe_receipt.json', timeout_name],
    'audit_authored_production_edits_this_turn': False, 'next_read': next_read,
    'pending_verification': 'No owned process live. Full graphics-probe Cargo resolution timed out without a Cargo result; no original library/probe test, Node/Wasm, GPU or radio run. Earlier Q352 remains pending.',
    'diagnostic_outcomes': 'Two unchanged-launcher native-double cases prove failed Cargo still reaches Node and exits0. Successful shallow metadata and subsequent terminal full-resolution timeout are preserved separately and are not compile proof.'}
diff = subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True, text=True)
assert diff.returncode == 0
with (HERE / 'mooting_illume_probes_diff_check.txt').open('x', encoding='utf-8') as f: f.write(diff.stdout + diff.stderr)
addition = f'''

### Mooting, Illume and probe source complete: scoped success and ownership claims, 2026-10-17

Read31new complete files /209,825characters at unchanged HEAD {head}: Mooting3files /10,090characters, Illume9 /64,005 and all probes19 /135,730. Coverage is{stats['complete_files']:,}/{stats['total_files']:,}files /{stats['read_chars']:,}/{stats['total_chars']:,}characters, with no stale/pending credit. Full review remains active and incomplete.

Q366 reproduces the unchanged lexical-Wasm PowerShell launcher continuing after a Cargo native double exits7, reaching the successful Node double and exiting0. Cargo0 is the control. Two audit cases, zero original tests; fresh no-profile child PowerShell and child-only PATH doubles. No real Cargo build, Node engine, Wasm module or stale artifact ran. Native failure propagation is a bounded repair candidate; production is unchanged. The native and Wasm source fixtures currently match and reuse the actual codec, but the JS wrapper prints digest/length without comparing the native pin. Historical source comments are not a fresh cross-engine receipt.

A1048-A1059 record actual pure recognition ownership versus stale storage prose; caller-supplied electorate versus signed provenance; shared Jotdown parsing versus duplicated offset/stack loops; one trusted injection registry versus host span validation; coarse Logos floors versus stale later-tier prose; entity text matching versus semantic navigation; real protocol admission versus HEADED label; duplicate signed-pack fixtures and transferred manifests versus payload/install/curation; shared graphics workload/device versus explicit B quarter-yaw differences; a partially shared geometry oracle versus independent depth checks; measurements without a fail threshold; and intentional cache contracts versus unexecuted GPU evidence. Heading breaks and A obsolete-strip cache behavior remain source candidates awaiting controlled evidence.

The standalone graphics dependency observation is qualified explicitly. Three literal nested Mesocosm manifests are absent; current parent Cargo config has no active paths override. The first locked/offline --no-deps metadata command exited0, after which its runner failed an incorrect expected-failure assertion. Full locked/offline metadata then terminated in Python TimeoutExpired after30seconds, with no Cargo exit diagnosis, full-resolution receipt or log. The original path receipt's buildability wording is overstrong and superseded by the timeout observation. This is not an established compile failure or graphics defect. No resolver process remains live; source and probe lock hashes are unchanged.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}), [launcher diagnostic](../../../support/receipts/authority-success-audit/lexical_launcher_failure_probe_receipt.json), and [metadata qualification](../../../support/receipts/authority-success-audit/{timeout_name}). No production edits or original Rust/Node/Wasm/GPU/radio/installation tests. Prior evidence, all protected source/runtime/auxiliary hashes, shared WIP and Git index are preserved; previous report bytes remain an exact prefix. Diff check passes. No retained isolated target, Cargo home, worktree or generated build output; tiny audit command doubles remain as evidence within the100MB budget. Next: unread script/app-host source.
'''
doc.write_bytes(prefix + addition.encode('utf-8'))
for n, v in [('review_notes.json', notes), ('authority_map.json', authority)]:
    (HERE / n).write_text(json.dumps(v, indent=2) + '\n', encoding='utf-8')
evidence = ['record_mooting_illume_probes_review.py', 'mooting_illume_probes_diff_check.txt',
    'run_lexical_launcher_failure_probe.py', 'lexical_launcher_failure_probe_receipt.json',
    'probe-native-stubs/cargo.cmd', 'probe-native-stubs/node.cmd', 'inspect_probe_dependency_paths.py',
    'probe_dependency_paths_receipt.json', 'wing_three_paths_metadata_output.log',
    'resolve_probe_dependency_paths.py', timeout_name]
budget = sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file()); assert budget < 100_000_000
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': head,
    'kind': 'Mooting, Illume, probes source complete; scoped launcher false success; repository incomplete',
    'prior_receipt': {'path': 'mien_moothold_review_receipt.json', 'sha256': sha(HERE / 'mien_moothold_review_receipt.json')},
    'coverage': stats, 'completed_source_areas': completed, 'whole_current_files': fresh,
    'source_sha256': {p: sha(ROOT / p) for p in fresh}, 'preserved_production_and_test_hashes': guards,
    'auxiliary_read_sha256': aux, 'runtime_input_sha256': runtime, 'shared_working_file_sha256': shared,
    'new_findings': ['Q366'], 'authority_entries': [e['id'] for e in entries],
    'audit_authored_production_edits_this_pass': False,
    'diagnostics': {'lexical_launcher_failure_probe_receipt.json': {'checks': 2, 'original_tests': 0, 'audit_cases': 2},
        'probe_dependency_paths_receipt.json': {'metadata_exit': 0, 'runner_final_assertion_failed': True, 'full_resolution': False},
        timeout_name: {'terminal_python_timeout_seconds': 30, 'cargo_exit': None, 'full_resolution': False}},
    'pending_verification': notes['checkpoint']['pending_verification'], 'next_read': next_read,
    'document': {'path': old['document']['path'], 'sha256': sha(doc), 'preserved_prefix_bytes': len(prefix), 'preserved_prefix_sha256': hashlib.sha256(prefix).hexdigest()},
    'evidence_sha256': {p: sha(HERE / p) for p in evidence}, 'git_index_sha256': index_hash,
    'observed_config_sha256': {'.cargo/config.toml': sha(ROOT / '.cargo/config.toml'), str(parent_config): sha(parent_config)},
    'receipt_budget': {'cap_bytes': 100_000_000, 'retained_bytes_before_receipt': budget}, 'retained_isolation': [],
    'limits': probe['qualification'] + ' Full dependency resolution timed out without a Cargo result; absent literal paths alone are not a counted compile failure. No original tests, real Cargo compile, Wasm, Node engine, GPU, radio, install or headed product run.'}
save(NAME, receipt)
assert sha(index) == index_hash and doc.read_bytes().startswith(prefix)
assert all(sha(ROOT / p) == h for p, h in {**guards, **receipt['source_sha256'], **aux, **runtime}.items())
assert all(sha(HERE / p) == h for p, h in receipt['evidence_sha256'].items())
assert all(sha(ROOT / p) == h for p, h in shared.items() if p != old['document']['path'])
assert subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True).returncode == 0
print(json.dumps({'receipt': NAME, 'coverage': stats, 'new_findings': ['Q366'], 'authority_entries': receipt['authority_entries'],
                  'protected_sources': len({**guards, **receipt['source_sha256']}), 'retained_bytes': budget}, indent=2))
