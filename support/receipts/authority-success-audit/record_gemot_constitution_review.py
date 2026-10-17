"""Record Gemot governance/member/delegation/hosting review and divergence proof."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
NAME = 'gemot_constitution_review_receipt.json'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE / n).read_text(encoding='utf-8-sig'))
def save(n, v): (HERE / n).write_text(json.dumps(v, indent=2) + '\n', encoding='utf-8')
def git(*a): return subprocess.check_output(['git', *a], cwd=ROOT, text=True).strip()

assert not (HERE / NAME).exists()
old = load('gemot_examples_review_receipt.json')
head = git('rev-parse', 'HEAD')
assert head == old['basis_commit']
guards = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}
aux, runtime = old['auxiliary_read_sha256'], old['runtime_input_sha256']
assert len(guards) == 1983
assert all(sha(ROOT / p) == h for p, h in {**guards, **aux, **runtime}.items())
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
assert stats['complete_files'] == 2469 and stats['read_chars'] == 29472900
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
area = 'crates/moot/gemot/'
files = [p for p in inv['files'] if p.startswith(area)]
complete = [p for p in files if cov.get(p, {}).get('through') == inv['files'][p]['chars']]
fresh = [p for p in complete if p not in guards]
service = area + 'src/moot/service.rs'
assert len(files) == 51 and len(complete) == 37 and len(fresh) == 16
assert sum(inv['files'][p]['chars'] for p in fresh) == 197774
assert cov[service]['through'] == 25978
source = fresh + [service]
assert all(cov[p]['sha256'] == sha(ROOT / p) == inv['files'][p]['sha256'] for p in source)
probe = load('gemot_constitution_probe_v2_receipt.json')
assert probe['build_exit'] == probe['run_exit'] == 0 and probe['checks'] == 3
assert probe['original_tests'] == 0 and probe['generated_output_removed']
assert probe['probe_sha256'] == sha(HERE / 'gemot_constitution_probe.rs')
assert probe['runner_sha256'] == sha(HERE / 'run_gemot_constitution_probe_v2.py')
assert all(sha(ROOT / p) == h for p, h in probe['source_sha256'].items())
assert all(sha(Path(v['path'])) == v['sha256'] for v in probe['libraries'].values())
assert all(not Path(v['path']).exists() for v in probe['removed_outputs'])
miss = load('gemot_constitution_v1_supplier_miss.json')
assert not miss['executed_compilation'] and miss['executed_checks'] == 0
assert miss['runner_sha256'] == sha(HERE / 'run_gemot_constitution_probe.py')
assert not list(Path('C:/t/cargo-targets/mere').glob('authority-success-gemot-constitution.*'))

notes, authority = load('review_notes.json'), load('authority_map.json')
assert notes['findings'][-1]['id'] == 'Q354' and authority['entries'][-1]['id'] == 'A1011'
notes['findings'].append({
    'id': 'Q355', 'title': 'Current-revision constitution admission makes competing eligible amendments diverge across replicas',
    'source': area+'src/moot/constitution/store.rs:223,249; '+area+'src/moot/constitution/fold.rs:106,164',
    'status': 'reproduced four unchanged current production modules over ten exact retained suppliers',
    'evidence': 'A valid founder genesis configures two distinct electors and threshold one. Each signs a different replacement rule set naming that same genesis revision, using an independent sequence-zero log. Both operations verify. The pure fold over the complete corpus returns the same deterministic law in either iteration order. Two real ConstitutionStore/MemoryBackend replicas instead accept opposite first amendments, reject each other\'s amendment as StaleRevision before retention, and keep different rules/revisions. Repeated cross-delivery is still refused. This is not a same-signer log fork. Sequential founder amendments and an amendment retried after its initially missing genesis both pass as controls.',
    'scope': probe['qualification'], 'verification': NAME,
    'diagnostics': ['gemot_constitution_probe_v2_receipt.json'],
    'repair_triage': {'disposition': 'document high-effect constitutional reconciliation issue under read-only audit',
        'reason': 'Separate cryptographic retention and deterministic authority resolution, or supply another explicit convergent reconciliation rule for competing prior-revision proposals. Current-state authorization before retaining the rival corpus cannot inherit the fold\'s convergence guarantee. Preserve bounded admission and historical authority validation; changing constitutional conflict/retention semantics is larger than an isolated mechanical fix. Do not choose a replacement governance policy during this audit.'}
})
rows = [
    ('Constitutional fold and store admission supply incompatible convergence boundaries',
     'The fold resolves competing accepted proposals by operation hash, while accept requires the currently selected revision.',
     [area+'src/moot/constitution/fold.rs', area+'src/moot/constitution/store.rs'],
     'Q355 reproduces different laws under opposite arrival order using distinct eligible authors and no per-author fork. Determinism given the same corpus does not establish convergence when admission prevents that corpus from being retained.'),
    ('Constitution live apply and corpus fold deliberately support different quorum scopes',
     'Constitution.apply returns PendingQuorum for quorum rules; the corpus fold combines distinct supporters and derives a proposal-based revision.',
     [area+'src/moot/constitution/fold.rs', area+'src/moot/constitution/governance.rs'],
     'The stable quorum revision depends on prior revision and rule digest, not which supporting signature arrives last. Governance.amend retains a signature then returns the current snapshot, so an incomplete quorum can return Ok with unchanged law. Its replace-current-clauses wording needs a retained-vote versus accepted-amendment qualification; no separate high-level runtime proof ran.'),
    ('Checkpoint authority history is reconstructed from sequential corpus prefixes',
     'checkpoint_authority_history sorts by per-author sequence and folds every growing prefix.',
     [area+'src/moot/constitution/governance.rs'],
     'This is a separate reconstruction around the shared fold. Per-author sequence is not a global amendment order, and transient competing-prefix winners need qualification. Trace retention consumers before claiming the emitted list is exactly the final accepted ancestry. No historical checkpoint bypass or aggregate-drop runtime case was executed here.'),
    ('Delegation remote retention defers authority to one materializer',
     'MootDelegationStore accepts valid signer-bound statements before resolving parent/root authority; delegations retries unresolved statements.',
     [area+'src/moot/delegation.rs', area+'src/moot/delegation/store.rs', area+'src/moot/delegation/wire.rs'],
     'Portable Personae certificates own signature/attenuation; Gemot owns constitutional roots and chain liveness. Current rules can invalidate retained descendants without erasing evidence. Missing-parent and rejected authority facts are not separate mutable graph authority. Scope-key epochs count accepted certificate revocations; secret generation/distribution remains elsewhere.'),
    ('Membership has an explicit remote-retention and deterministic projection split',
     'MootGroupStore retains bounded structurally valid records, resolves dependency frontiers and reports pending/rejected counts.',
     [area+'src/moot/group.rs', area+'src/moot/group/store.rs', area+'src/moot/group/wire.rs'],
     'The stable Gemot action grammar is translated to p2panda-auth rather than exposing upstream internal serde as the wire protocol. Local authoring preflights auth; remote retention and materialization are distinct. Source tests cover pending cross-author dependencies and durable reopen but were not run in this pass.'),
    ('Membership repeats a stable-root attestation binding recipe',
     'group/wire.rs stable_author checks membership salt, derived signer and master root locally.',
     [area+'src/moot/group/wire.rs', 'crates/moot/commons/src/chat.rs', 'crates/moot/commons/src/lib.rs'],
     'This repeats the attestation-to-stable-subject recipe already tracked in Commons chat versus the shared Stickleback helper. Domain salts and typed errors differ; duplication is real but does not establish a failed signature check or forged root.'),
    ('Membership epoch bindings name host secrets without distributing them',
     'MootGroup tracks resolved member-set transitions and binds supplied GroupSecretId values to group or delegation-scope epochs.',
     [area+'src/moot/group.rs', area+'src/moot/delegation.rs'],
     'Access-level-only changes deliberately leave the epoch stable. Bind methods validate their local group/epoch prerequisites and return identifiers, not an executed encryption rotation or recipient-distribution receipt. Replay/current-host lifetime and root-grant removal must be traced separately from certificate-revocation counts.'),
    ('Production hosting promises and older rehearsal commitments are different implementations',
     'hosting.rs provides bounded HostingPromise values through the signed Standing lane; author-offline-publication still uses proof-local HostingCommitmentV1.',
     [area+'src/moot/hosting.rs', area+'src/moot/hosting_tests.rs', area+'examples/author-offline-publication.rs'],
     'The production API owns audience, expiry, byte/time ceilings, policy revision and authority cut. The earlier rehearsal does not exercise this complete owner. Its candidate-grammar limitations must not be repeated as a claim that the production promise grammar is absent. Shared Standing evidence is real; duplicate promise formats remain visible.'),
    ('Hosting preparation, signing and serving have separate explicit owners',
     'prepare_hosting bounds a request; commit_hosting rebuilds and compares the preparation; check_hosting checks live records, current authority, policy, contribution and ceilings.',
     [area+'src/moot/hosting.rs'],
     'The host must serialize these operations with authority ingestion across separate lane stores. Content-byte custody and real availability are explicit content-owner prerequisites, not inferred by the signed promise. AuthorityCut is carried evidence; no historical-frontier authenticity or crash-atomic multi-store execution was proved by this source pass.'),
    ('FLORA social validation is distinct from numerical tensor verification',
     'FloraProjection validates positive ranks/weights, exact sum/budget, full contributor receipts and declared candidate rank.',
     [area+'src/moot/flora.rs', area+'src/moot/artifact.rs'],
     'The fold retains all facts and excludes incompatible candidates, but artifact references and matching declarations do not establish actual tensor dimensions, training correctness or execution. Distillery/ESP owns numerical execution; Gemot carries public references. Continue aggregate authorization consumers before treating low-level signed fact admission as a product authorization bypass.'),
    ('Aggregate ownership does not make seven lane stores one transaction',
     'Moot composes governance, objects, Standing, Tulpa, FLORA, delegation and membership; durable open uses separate backend files.',
     [area+'src/moot/service.rs', area+'src/moot/hosting.rs'],
     'Only service.rs characters0:25978 reviewed so far. amend persists governance, separately rebuilds checkpoint authority/history, updates the object policy and reads a snapshot. Its atomically-switch wording needs a precise boundary under concurrent ingestion/failure. Hosting already states caller serialization. No whole-service race or atomicity fault was executed.'),
    ('Current capability routes intersect signed grants with injected facts',
     'authorize_current_capability combines direct or delegated roots with provider coverage and constitutional admission.',
     [area+'src/moot/service.rs', area+'src/moot/group.rs'],
     'The lower authorize route intentionally evaluates only injected structural facts under the signed policy. MootGroup.inputs supplies membership/write access and default other Standing facts. Trace host commands under nonzero Standing floors before claiming that real retained Standing scores are consumed automatically; later service consumers remain unread.'),
]
entries = [{'id': f'A{1012+i}', 'domain': d, 'claim': c, 'implementations': p, 'assessment': a,
            'state': 'Current source reviewed within recorded ranges; three constitutional diagnostic cases executed', 'verification': NAME}
           for i, (d, c, p, a) in enumerate(rows)]
authority['entries'].extend(entries)
completed = []
for prefix_name, expected in [('src/moot/constitution/', 7), ('src/moot/delegation', 4), ('src/moot/group', 3)]:
    selected = [p for p in complete if p.startswith(area + prefix_name)]
    assert len(selected) == expected
    completed.append({'prefix': area+prefix_name, 'files': len(selected),
                      'chars': sum(inv['files'][p]['chars'] for p in selected), 'basis_commit': head,
                      'verification': NAME, 'qualification': 'Every inventoried matching file read. Runtime scope is only four constitutional modules over retained suppliers; other package, live sync and durable tests were source-reviewed, not rerun.'})
notes['completed_source_areas'].extend(completed)
partial = {'prefix': area, 'files_read': len(complete), 'total_files': len(files),
           'chars_read': sum(cov.get(p, {}).get('through', 0) for p in files), 'total_chars': 777720,
           'verification': NAME, 'basis_commit': head,
           'partial_ranges': [{'path': service, 'through': 25978, 'chars': 104697}],
           'remaining': [p for p in files if p not in complete]}
for previous in notes.get('partial_source_areas', []):
    if previous.get('prefix') == area: previous['superseded_by'] = NAME
notes.setdefault('partial_source_areas', []).append(partial)
followups = [
    ('Quorum command outcome', 'MootGovernance.amend retains a valid supporter and returns current snapshot even when threshold is incomplete. Distinguish vote retained from rules accepted in command status/prose before calling the replacement complete.'),
    ('Historical checkpoint authority reconstruction', 'Prefix folding sorted only by per-author sequence is not automatically final accepted ancestry. Trace actual consumers and competing/quorum history before asserting a historical signer bypass.'),
    ('Aggregate authority publication boundary', 'Moot.amend durable governance write, refreshed authority/history, object policy switch and snapshot are separate operations. Qualify atomic wording and inspect actual host serialization/failure paths; no broad race was run.'),
    ('Standing facts at automatic command seams', 'MootGroup.inputs fills only membership/write coverage and defaults the rest. Trace convenience commands under a positive Standing floor and the current Standing owner before asserting those commands consume retained reputation.'),
]
notes['review_followups'].extend({'verification': NAME, 'area': a, 'detail': d} for a, d in followups)
next_read = {'path': service, 'start': 25978, 'reason': 'Finish unread aggregate command/import/snapshot/test source, then remaining Gemot lane/record/Tulpa/typed-authority files; full repository review remains incomplete.'}
notes['current_area'] = 'Gemot constitutional, delegation, membership, FLORA and hosting source reviewed; aggregate service partial; constitutional admission divergence reproduced'
notes['next_area'] = next_read['reason']
notes['checkpoint'] = {'classification': 'progress', 'classification_of_previous_turn': 'progress',
    'goal_status': 'active; full source incomplete', 'coverage': stats, 'finding_count': len(notes['findings']),
    'authority_entries': len(authority['entries']), 'new_receipts': [NAME, 'gemot_constitution_probe_v2_receipt.json'],
    'audit_authored_production_edits_this_turn': False, 'next_read': next_read,
    'pending_verification': 'Q352 whole-owner key-replacement remains unexecuted; no whole Gemot/current Cargo graph, live transport, headed or disk gate rerun; diagnostic terminal',
    'diagnostic_outcomes': 'Three current constitutional module cases pass: sequential convergence control, opposite-arrival eligible proposals with permanent ordinary-admission divergence, missing-genesis retry control. Initial d128 supplier selection stopped before rustc; v2 used a complete retained family. Generated outputs removed.'}
diff = subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True, text=True)
assert diff.returncode == 0
with (HERE / 'gemot_constitution_diff_check.txt').open('x', encoding='utf-8') as out: out.write(diff.stdout + diff.stderr)
addition = f'''

### Gemot constitutional admission, delegation, membership and hosting, 2026-10-17

Read16new complete files /197,774characters plus25,978characters from the aggregate service at unchanged HEAD {head}. Constitution7/7, delegation4/4 and group3/3 inventoried files are now fully read; FLORA and both hosting files are also read. Gemot is37/51files with the service still partial at25,978of104,697characters. Repository coverage is{stats['complete_files']:,}/{stats['total_files']:,}files /{stats['read_chars']:,}/{stats['total_chars']:,}characters, with no stale or pending read credit. Full source review remains active and incomplete.

Q355 reproduces constitutional divergence under opposite arrival order. A valid founder genesis makes two distinct electors eligible at quorum threshold1. Each signs a different amendment of the same revision in its independent sequence-zero log. All signatures check, and the pure fold over the complete corpus deterministically selects the same law in either order. Real ConstitutionStore/MemoryBackend replicas instead accept different first arrivals, reject each other's valid proposal as StaleRevision before retaining it, and keep different laws and revisions. Repeated cross-delivery remains refused. This is not a same-signer log fork. Sequential founder amendments and retry after initially missing genesis pass as controls.

The diagnostic freshly compiles the unchanged complete event/fold/wire/store modules over ten exact retained suppliers. Real operation signatures, shared OperationProcessor and MemoryBackend execute; no fake admission or storage is substituted. This is not whole Gemot, a current Cargo graph, live LogSync, aggregate service, disk or headed acceptance. Initial retained-family selection failed before compilation because its identity fingerprint had no matching current marker/artifact pair. A targeted inventory found two other complete families; v2 used2a2dd68ccc68833a and passed all3auditcases. The initial miss is preserved. Generated executable/PDB/link objects are removed.

A1012-A1023 record the missing link between deterministic folding and actual admission convergence, live-apply versus corpus quorum scope, separate checkpoint-history reconstruction, actual delegation and membership retain/project ownership, duplicated stable-author binding checks, host-secret identifier limits, current production hosting versus the older proof-local grammar, hosting preparation/sign/serve boundaries, social FLORA declarations versus numerical execution, separate aggregate stores and signed-grant/provider intersections.

Current hosting.rs already provides the complete bounded HostingPromise grammar through signed Standing records. The older publication rehearsal's candidate HostingCommitmentV1 is a different proof-local implementation and does not exercise this owner. Preparation checks policy/authority/contribution and ceilings, commit rebuilds the preparation, and serving checks live promise/current authority/expiry/withdrawal/ceilings. Hosts explicitly serialize against ingestion across separate stores; content custody and observed availability belong elsewhere. This source review does not independently authenticate historical AuthorityCut completeness or execute durable hosting tests.

Higher-effect followups are documented rather than patched: quorum command Ok can mean one retained signature with unchanged law; checkpoint history folds sequence-sorted corpus prefixes; aggregate amend persists governance and separately switches the object policy despite atomic wording; automatic MootGroup facts default non-membership Standing fields. Trace the remaining service/authority consumers before promoting those source concerns to broader product failures. No constitutional conflict, GC or encryption-policy semantics are chosen here.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}), [constitutional counterexample](../../../support/receipts/authority-success-audit/gemot_constitution_probe_v2_receipt.json) and [initial supplier miss](../../../support/receipts/authority-success-audit/gemot_constitution_v1_supplier_miss.json). No production edits. Protected source/runtime/auxiliary hashes, prior evidence, shared WIP and Git index remain intact; previous document bytes remain an exact prefix. Diff check passes. No live diagnostic process or retained isolated target/home/worktree. Evidence remains under100MB. Next: service.rs from character25,978, then remaining Gemot sources.
'''
doc.write_bytes(prefix + addition.encode('utf-8'))
save('review_notes.json', notes)
save('authority_map.json', authority)
evidence = ['record_gemot_constitution_review.py', 'gemot_constitution_diff_check.txt',
            'gemot_constitution_probe.rs', 'run_gemot_constitution_probe.py',
            'prepare_gemot_constitution_v2.py', 'gemot_constitution_v1_supplier_miss.json',
            'run_gemot_constitution_probe_v2.py', 'gemot_constitution_probe_v2_compile.log',
            'gemot_constitution_probe_v2_output.log', 'gemot_constitution_probe_v2_receipt.json']
budget = sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file())
assert budget < 100_000_000
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': head,
    'kind': 'Gemot constitutional admission divergence, authority source review; aggregate service partial; repository incomplete',
    'prior_receipt': {'path': 'gemot_examples_review_receipt.json', 'sha256': sha(HERE / 'gemot_examples_review_receipt.json')},
    'coverage': stats, 'completed_source_areas': completed, 'partial_source_areas': [partial],
    'whole_current_files': fresh, 'partial_source_ranges': partial['partial_ranges'],
    'source_sha256': {p: sha(ROOT / p) for p in source}, 'auxiliary_read_sha256': aux,
    'runtime_input_sha256': runtime, 'preserved_production_and_test_hashes': guards,
    'shared_working_file_sha256': shared, 'new_findings': ['Q355'],
    'authority_entries': [e['id'] for e in entries], 'audit_authored_production_edits_this_pass': False,
    'diagnostics': {'gemot_constitution_probe_v2_receipt.json': {'build_exit': 0, 'run_exit': 0, 'checks': 3, 'original_tests': 0, 'audit_cases': 3, 'generated_output_removed': True}},
    'pending_verification': notes['checkpoint']['pending_verification'], 'next_read': next_read,
    'document': {'path': old['document']['path'], 'sha256': sha(doc), 'preserved_prefix_bytes': len(prefix),
                 'preserved_prefix_sha256': hashlib.sha256(prefix).hexdigest()},
    'evidence_sha256': {p: sha(HERE / p) for p in evidence}, 'git_index_sha256': index_hash,
    'retained_isolation': [], 'receipt_budget': {'cap_bytes': 100_000_000, 'retained_bytes_before_receipt': budget},
    'limits': probe['qualification'] + ' Aggregate service only partially read; full Gemot/repository review incomplete.'}
save(NAME, receipt)
assert sha(index) == index_hash and doc.read_bytes().startswith(prefix)
assert all(sha(ROOT / p) == h for p, h in {**guards, **receipt['source_sha256'], **aux, **runtime}.items())
assert all(sha(HERE / p) == h for p, h in receipt['evidence_sha256'].items())
assert all(sha(ROOT / p) == h for p, h in shared.items() if p != old['document']['path'])
assert subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True).returncode == 0
print(json.dumps({'receipt': NAME, 'coverage': stats, 'new_findings': receipt['new_findings'],
                  'authority_entries': receipt['authority_entries'], 'protected_sources': len({**guards, **receipt['source_sha256']}),
                  'retained_bytes': budget}, indent=2))
