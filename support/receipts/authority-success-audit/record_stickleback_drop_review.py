"""Record fourteen complete Stickleback reads; keep the remaining source open."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess
HERE = Path(__file__).resolve().parent; ROOT = HERE.parents[2]
NAME = 'stickleback_drop_review_receipt.json'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE / n).read_text(encoding='utf-8-sig'))
def save(n, value):
    with (HERE / n).open('x', encoding='utf-8') as f: json.dump(value, f, indent=2); f.write('\n')
def git(*args): return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()
assert not (HERE / NAME).exists()
old = load('shell_chrome_comms_review_receipt.json'); head = git('rev-parse', 'HEAD'); assert head == old['basis_commit']
guards = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}; assert len(guards) == 2132
aux, runtime = old['auxiliary_read_sha256'], old['runtime_input_sha256']
assert all(sha(ROOT / p) == h for p, h in {**guards, **aux, **runtime}.items())
assert all(sha(HERE / p) == h for p, h in old['evidence_sha256'].items())
shared = old['shared_working_file_sha256']; doc = ROOT / old['document']['path']; prefix = doc.read_bytes()
assert sha(doc) == old['document']['sha256']
assert all(sha(ROOT / p) == h for p, h in shared.items() if p != old['document']['path'])
index = Path(git('rev-parse', '--git-path', 'index'))
if not index.is_absolute(): index = ROOT / index
index_hash = sha(index); assert index_hash == old['git_index_sha256']
inv, cov = load('source_inventory.json'), load('read_coverage.json')
stats = json.loads(subprocess.check_output(['python', str(HERE / 'read_coverage.py'), 'status'], cwd=ROOT))
assert stats['complete_files'] == 2616 and stats['read_chars'] == 30927517
assert inv['commit'] == head and load('pending_read.json') == [] and not stats['changed_since_inventory']
all_files = [p for p in inv['files'] if p.startswith('crates/stickleback/')]
fresh = [p for p in all_files if cov.get(p, {}).get('through') == inv['files'][p]['chars']]
assert len(all_files) == 30 and len(fresh) == 14 and sum(inv['files'][p]['chars'] for p in fresh) == 200309
assert not set(fresh).intersection(guards)
assert all(cov[p]['sha256'] == sha(ROOT / p) == inv['files'][p]['sha256'] for p in fresh)
assert next(p for p in all_files if p not in fresh) == 'crates/stickleback/src/group_lane.rs'
attempts = [(n, load(n + '_receipt.json')) for n in ['stickleback_export_probe', 'stickleback_export_probe_v2', 'stickleback_export_probe_v3']]
assert attempts[0][1]['results']['owner_compile']['exit'] == 1 and attempts[0][1]['audit_cases'] == 0
assert attempts[1][1]['results']['owner_compile']['exit'] == 0 and attempts[1][1]['results']['consumer_compile']['exit'] == 1 and attempts[1][1]['audit_cases'] == 0
probe = attempts[2][1]; assert probe['audit_cases'] == 2 and probe['original_tests'] == 0 and all(v['exit'] == 0 for v in probe['results'].values())
for name, item in attempts:
    assert item['runner_sha256'] == sha(HERE / ('run_' + name + '.py'))
    source = 'stickleback_export_probe_v3.rs' if name.endswith('_v3') else 'stickleback_export_probe.rs'
    assert item['probe_sha256'] == sha(HERE / source)
    assert item['generated_output_removed'] and all(not Path(v['path']).exists() for v in item['removed_outputs'])
    assert all(sha(ROOT / p) == h for p, h in item['source_sha256'].items())
    assert all(sha(Path(v['path'])) == v['sha256'] for v in item['libraries'].values())
assert len(probe['source_sha256']) == 30 and len(probe['libraries']) == 17
assert {p: v['bytes'] for p, v in probe['fixtures'].items()} == {
    'drop-export-refused-plain.meredrop': 0, 'drop-export-refused-selected.meredrop': 0, 'drop-export-success-control.meredrop': 557}
assert all(sha(HERE / p) == v['sha256'] for p, v in probe['fixtures'].items())
notes, authority = load('review_notes.json'), load('authority_map.json')
assert notes['findings'][-1]['id'] == 'Q378' and authority['entries'][-1]['id'] == 'A1102'
b = 'crates/stickleback/'
finding = {'id': 'Q379', 'title': 'Refused native-drop file exports truncate an existing destination before validating the drop limits',
    'source': b + 'src/drop_io.rs:288,318,350; ' + b + 'src/drop.rs:260',
    'status': 'reproduced two public plaintext file-export variants over freshly compiled complete current owner',
    'evidence': 'A real signed operation is inserted through the actual processor/MemoryBackend. Both export_plain_topic_file and export_selected_plain_topic_file open an existing audit-owned sentinel file, then return a configured max_protected_bytes0 limit refusal. Each destination is empty afterward. A successful default-limits control round-trips the signed stored operation with matching semantic drop ID and leaves the source store at one operation.',
    'scope': probe['qualification'] + ' File export does not explicitly promise atomic replacement, but this observed failure destroys a prior destination before producing a valid replacement. This is not an ignored error or demonstrated corruption of a user export.',
    'diagnostics': ['stickleback_export_probe_v3_receipt.json'], 'verification': NAME,
    'repair_triage': {'disposition': 'bounded file-publication repair candidate documented under active read-only audit',
        'reason': 'Validate before touching the destination where possible, write/flush a new artifact and publish only after successful completion under an explicit replacement policy. Preserve the old destination on refusal or write failure. Coordinate portable replace/durability semantics; do not delete existing evidence to make an export succeed.'}}
notes['findings'].append(finding)
rows = [
('Stickleback public exports centralize the domain-neutral seams',
 'The public root re-exports private processor/store/drop-I/O/key helpers; domains supply grammar and authority.',
 [b + 'Cargo.toml', b + 'src/lib.rs', b + 'src/authority.rs'],
 'CheckpointAuthority is a real domain-injected interface, not membership inferred from transport. Its actual processor/store call paths remain unread this checkpoint, so compilation and trait prose do not certify all erasure gates.'),
('The address book shares Backend/Codec while retaining separate soft transport state',
 'NodeInfo and topic-interest slots are transport directory data, not Mere handle resolution.',
 [b + 'src/address_book.rs', b + 'src/address_book/tests.rs'],
 'This replaces the concrete SQLite address-book store and reuses Backend transactions for insert-if-absent. Other insert/remove/age flows read and then write separately; concurrent refresh versus eviction and set_topics versus node removal need controlled cases before assuming all directory mutations are atomic. Topic updates do not refresh the node local timestamp. Queries scan and clock-based pick is explicitly noncryptographic.'),
('Causal ordering is actually shared mechanics, not event authorization',
 'The helper propagates missing roots, topologically orders the complete subset and bounds declared metadata.',
 [b + 'src/causal.rs'],
 'CausalIndex avoids rebuilding an index for every reachability query; complete_frontier removes transitively covered heads. Caller-provided entries still need structural admission: sequence-fork checks are not backlink/identity validation. observed_frontier can name pending heads by design, while complete_frontier excludes them. Seven original source tests were read, not freshly run.'),
('Drop framing separates semantic ID, body integrity and domain policy',
 'Canonical manifest hashes identify records; plain/protected covers verify carrier bytes with explicit suite injection.',
 [b + 'src/drop.rs'],
 'Plain visitation explicitly stages before final body digest; the public importer uses read_plain_drop to finish integrity before operations. Protected visitation authenticates through the supplied suite first. TestProtector is an explicit fake, not proof of production AEAD. Readers stop at declared body length rather than proving generic reader EOF; record/manifest bounds and suite behavior have their stated scopes.'),
('Canonicalization changed manifest identity inside the same drop version',
 'Golden source comments re-freeze bytes after the CBOR encoder migration while VERSION remains1.',
 [b + 'src/drop.rs'],
 'New code checks strict canonical struct re-encoding and records a changed DropId/golden cover. Old artifact compatibility is not established by the statement that the drop format itself is unchanged. Retain an old valid fixture before claiming compatibility or classifying its rejection as tampering; no old-reader/writer migration execution ran.'),
('Carrier verification is followed by shared operation processing and batched final effects',
 'import_drop_records assembles verified chunks, decodes operations, preflights each, stages the corpus then calls process_batch_atomic.',
 [b + 'src/drop_io.rs'],
 'This is actual processor reuse, with stage cleanup and local receipt in final writes. Full concurrency/rollback guarantees depend on the unread processor/store/Backend transaction implementations, not this call shape alone. Staging is a separate durable batch on purpose so continuity failure can resume.'),
('Public verified-record import is a caller trust seam',
 'import_drop_records accepts an already verified DropId/corpus; aggregate domains admit critical evidence first.',
 [b + 'src/drop_io.rs', b + 'src/drop.rs'],
 'Raw API does not recompute that semantic identity or apply domain-specific evidence effects. Receipt-hit short-circuits current preflight, and resume trusts staged metadata/records subject to current operation preflight. These preconditions need actual consumer tracing before a replay/critical-evidence bypass claim. Generic non-operation counts do not claim those facts were applied.'),
('Peer advisory receipts and local completion receipts remain separate namespaces',
 'Peer receipts are scoped to an explicit authenticated-peer token and do not become local import receipts.',
 [b + 'src/drop_io.rs'],
 'Peer scope is caller-supplied authentication evidence; no wire peer proof is verified by this storage helper. Local completion is a durable batch receipt, not a recipient read receipt or a guarantee the material survives later pruning. Resume checks count/record bounds; total staged-corpus byte and identity checks remain a source followup.'),
('Payload hydration and content storage are distinct from operation admission',
 'Chunk assembly checks layout/digest; absent headers may leave pending payload bytes for later operations.',
 [b + 'src/drop_io.rs'],
 'Original source tests exercise same-drop, later-header, later-payload and erased-body nonresurrection. They were read but not run this pass. Content-only imports intentionally require no operation, so their storage quota/lifetime is a host policy; do not mislabel them as successful domain event authorization.'),
('File publication has a different failure boundary from in-memory framing',
 'Each export helper opens/truncates the path and only then calls the validating codec.',
 [b + 'src/drop_io.rs', b + 'src/drop.rs'],
 'Q379 reproduces two plaintext variants. Protected export has the same source ordering but was not executed. sync_all after successful write is real durability work; it does not preserve an earlier destination if validation fails.'),
('Epoch pruning is a dry-run over supplied authority facts',
 'The proposal checks mode,current key,known chronology,matching revisions,continuation and per-epoch holds.',
 [b + 'src/epoch_retention.rs', b + 'src/group_crypto.rs'],
 'is_executable means mechanical blockers absent under caller-supplied facts, not that an authentic checkpoint was verified or destruction authorized. Any blocker retains all present epochs. Current epoch survives even floor0; actual minimum floors/holds remain product choices. No pruning or destruction semantics were selected or executed here.'),
('Data encryption is shared; membership and epoch authority are not supplied by the key container',
 'DataKeyring uses the p2panda data engine, explicit installed order and versioned validation.',
 [b + 'src/group_crypto.rs', b + 'src/epoch_retention.rs'],
 'Legacy chronology stays decryptable but blocks pruning. install/authenticated welcome and forget_authorized are host preconditions, not checked governance permissions. Out-of-order backfill/order consistency needs a real engine case before a finding. Persisted bytes include secret state; storage protection is outside this container. Group session/lane ownership remains unread.'),
('The inventory example is a finite policy model with explicit exclusions',
 'It consumes the actual inventory API/wire but models payload loss,slot budgets,topology and owner reception.',
 [b + 'examples/inventory_compare/main.rs', b + 'examples/inventory_compare/model.rs', b + 'examples/inventory_compare/run.py', b + 'examples/ledger.rs'],
 'Runner uses Cargo-reported executable, failure-checking subprocesses and two fresh-process byte equality, stronger than a printed digest alone. Report excludes signed operation/domain/physical/RF/FEC acceptance. Recovery assertions cover selected finite scenarios rather than every loss/retention case; fair-slot schedules differ explicitly between eager and inventory modes. Ledger is a print-oriented neutral-domain example, not a passing acceptance test.'),
('Source coverage remains distinct from whole-owner compilation',
 'Fourteen of30files are source complete; the diagnostic freshly compiles the whole owner over17retained suppliers.',
 [b + 'Cargo.toml', b + 'src/lib.rs'],
 'First audit attempt lacked Cargo package-version environment; second compiled owner but used a private module path. Both terminal failures are preserved. Third corrects the audit root import and runs2public cases. Zero original tests, fresh Cargo graph, live transport or actual user export loss claimed. All30source hashes protect the compile observation, not read credit.')]
entries = [{'id': f'A{1103 + i}', 'domain': d, 'claim': claim, 'implementations': paths, 'assessment': assessment,
            'state': 'bounded Stickleback source checkpoint; full remaining source review active', 'verification': NAME}
           for i, (d, claim, paths, assessment) in enumerate(rows)]
authority['entries'].extend(entries)
completed = []
for p in fresh:
    item = {'prefix': p, 'files': 1, 'chars': inv['files'][p]['chars'], 'basis_commit': head, 'verification': NAME,
            'qualification': 'Exact whole file read and acknowledged. Stickleback source remains14of30complete; whole-owner compile is separate evidence and not remaining-source read credit.'}
    completed.append(item); notes['completed_source_areas'].append(item)
notes['review_followups'].extend([
    {'verification': NAME, 'area': 'Drop file publication', 'detail': 'Q379 is a bounded failure-path repair candidate. Preserve prior destinations under limit, codec, encryption and disk errors; qualify atomic replacement/directory durability per platform.'},
    {'verification': NAME, 'area': 'Address-book races and drop retry identity', 'detail': 'Control age-scan versus refresh, set_topics versus removal, total stage bounds and trusted semantic DropId on resume. Do not call a batch atomic until the processor/store/Backend paths are read.'},
    {'verification': NAME, 'area': 'Epoch order, authority and compatibility', 'detail': 'Verify authenticated group lane/session integration and out-of-order secret backfill. Compare old CBOR drop artifacts before a version-compatibility claim. Keep dry-run blockers distinct from authentic checkpoint authority.'}])
next_read = {'path': 'crates/stickleback/src/group_lane.rs', 'start': 0, 'reason': 'Continue unread Stickleback group-key lane/session, inventory, processor/store, carriage and boundary tests.'}
notes['current_area'] = 'Stickleback14of30files source complete; file-export refusal truncation reproduced'
notes['next_area'] = next_read['reason']
notes['checkpoint'] = {'classification': 'progress', 'classification_of_previous_turn': 'progress', 'goal_status': 'active; full source incomplete',
    'coverage': stats, 'finding_count': len(notes['findings']), 'authority_entries': len(authority['entries']),
    'new_receipts': [NAME, 'stickleback_export_probe_receipt.json', 'stickleback_export_probe_v2_receipt.json', 'stickleback_export_probe_v3_receipt.json'],
    'audit_authored_production_edits_this_turn': False, 'next_read': next_read,
    'pending_verification': 'No owned process live. Sixteen Stickleback files remain unread. Original suites,fresh Cargo graph,protected export,network/RF and old-wire compatibility unexecuted; Q352/previous metadata timeout remain qualified.',
    'diagnostic_outcomes': 'Whole owner compiles;2public cases reproduce refusal truncation and successful control. Two preceding harness compile errors preserved. Original tests0. Generated owner rlib/consumer executable,PDB/link files removed; three tiny new audit-owned final export files retained.'}
diff = subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True, text=True); assert diff.returncode == 0
with (HERE / 'stickleback_drop_diff_check.txt').open('x', encoding='utf-8') as f: f.write(diff.stdout + diff.stderr)
addition = f'''

### Stickleback drop/storage boundary checkpoint:14of30source files, 2026-10-17

Read14new complete files /200,309characters at unchanged HEAD {head}: manifest, all4examples, address book plus tests, retention-authority seam, causal helpers, drop codec/import-export, epoch-retention proposal, data keyring and public lib exports. Stickleback remains16files unread, starting group_lane.rs; full crate compilation does not grant their source-reading credit. Coverage is{stats['complete_files']:,}/{stats['total_files']:,}files /{stats['read_chars']:,}/{stats['total_chars']:,}characters, with no stale/pending credit. Full review remains active and incomplete.

Q379 reproduces export_plain_topic_file and export_selected_plain_topic_file truncating an existing destination before returning a configured max_protected_bytes0 refusal. The two destinations are new audit-owned sentinel files and are empty afterward. A successful default-limits control round-trips one genuinely signed operation stored through the actual processor/MemoryBackend, with matching semantic drop ID and unchanged source-store count. No real user file or protected drop was exercised. File export does not explicitly promise atomic replacement; the evidence is the concrete destruction of a previous destination on a validation failure. The protected helper shares that source ordering but remains unexecuted. Repair should preserve old destinations until a valid new artifact can be published under an explicit replacement policy.

The complete unchanged current Stickleback owner and public consumer are freshly compiled over17exact retained Cargo-fingerprint-matched suppliers. First harness attempt omitted CARGO_PKG_VERSION; second compiled the owner but imported a private module instead of its public root re-export. Both failures remain preserved and neither executed file cases. Third corrects only audit setup/import and passes2cases; original tests0, no fresh Cargo graph or live transport. All30source hashes protect compile provenance only. Three tiny final fixtures are retained (0,0,557bytes); generated owner/library/executable/PDB/link outputs removed.

A1103-A1116 distinguish shared Backend address-book storage from soft directory metadata and remaining read/write races; actual causal mechanics from authenticated entries; manifest/carrier integrity from domain policy; pre/post-CBOR canonical identity migration inside version1; shared operation preflight/batch calls from still-unread atomic storage mechanics; trusted verified-record import and staged retry identity; advisory peer receipts from local durable completion; payload hydration/content-only storage from operation admission; file publication from in-memory validation; dry-run epoch facts from authenticated destruction authority; p2panda data crypto/chronology from membership control; finite inventory-model byte/replay evidence from signed/domain/physical/RF/FEC acceptance; and exact source coverage from compilation. No checkpoint/GC/destruction policy is chosen, and no caller trust assumption is presented as a verified permission.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}), [first compile failure](../../../support/receipts/authority-success-audit/stickleback_export_probe_receipt.json), [second consumer failure](../../../support/receipts/authority-success-audit/stickleback_export_probe_v2_receipt.json), and [public export cases](../../../support/receipts/authority-success-audit/stickleback_export_probe_v3_receipt.json). No production edits. Prior evidence, all protected/runtime/auxiliary hashes, shared WIP and Git index remain intact; all previous report bytes remain an exact prefix. Diff check passes. No retained isolated target,Cargo home or worktree, and no owned process live; retained evidence remains below100MB. Next: unread group-key lane/session and remaining Stickleback source.
'''
doc.write_bytes(prefix + addition.encode('utf-8'))
for n, value in [('review_notes.json', notes), ('authority_map.json', authority)]: (HERE / n).write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')
evidence = ['record_stickleback_drop_review.py', 'stickleback_drop_diff_check.txt', 'stickleback_export_probe.rs', 'stickleback_export_probe_v3.rs',
    'run_stickleback_export_probe.py', 'run_stickleback_export_probe_v2.py', 'run_stickleback_export_probe_v3.py',
    'stickleback_export_probe_receipt.json', 'stickleback_export_probe_owner_compile.log',
    'stickleback_export_probe_v2_receipt.json', 'stickleback_export_probe_v2_owner_compile.log', 'stickleback_export_probe_v2_consumer_compile.log',
    'stickleback_export_probe_v3_receipt.json', 'stickleback_export_probe_v3_owner_compile.log', 'stickleback_export_probe_v3_consumer_compile.log', 'stickleback_export_probe_v3_output.log', *probe['fixtures']]
budget = sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file()); assert budget < 100_000_000
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': head,
    'kind': 'fourteen complete Stickleback file reads; whole-owner file-export diagnostic; repository incomplete',
    'prior_receipt': {'path': 'shell_chrome_comms_review_receipt.json', 'sha256': sha(HERE / 'shell_chrome_comms_review_receipt.json')},
    'coverage': stats, 'completed_source_areas': completed, 'whole_current_files': fresh,
    'source_sha256': {p: sha(ROOT / p) for p in fresh}, 'compile_source_sha256': probe['source_sha256'],
    'compile_protection_is_not_read_credit': True, 'remaining_stickleback_files': [p for p in all_files if p not in fresh],
    'preserved_production_and_test_hashes': guards, 'auxiliary_read_sha256': aux, 'runtime_input_sha256': runtime, 'shared_working_file_sha256': shared,
    'new_findings': ['Q379'], 'authority_entries': [e['id'] for e in entries], 'audit_authored_production_edits_this_pass': False,
    'diagnostics': {n + '_receipt.json': {'exits': {k: v['exit'] for k, v in item['results'].items()}, 'audit_cases': item['audit_cases'], 'original_tests': 0} for n, item in attempts},
    'pending_verification': notes['checkpoint']['pending_verification'], 'next_read': next_read,
    'document': {'path': old['document']['path'], 'sha256': sha(doc), 'preserved_prefix_bytes': len(prefix), 'preserved_prefix_sha256': hashlib.sha256(prefix).hexdigest()},
    'evidence_sha256': {p: sha(HERE / p) for p in evidence}, 'git_index_sha256': index_hash,
    'receipt_budget': {'cap_bytes': 100_000_000, 'retained_bytes_before_receipt': budget}, 'retained_isolation': [], 'limits': probe['qualification']}
save(NAME, receipt)
assert sha(index) == index_hash and doc.read_bytes().startswith(prefix)
assert all(sha(ROOT / p) == h for p, h in {**guards, **receipt['source_sha256'], **probe['source_sha256'], **aux, **runtime}.items())
assert all(sha(HERE / p) == h for p, h in receipt['evidence_sha256'].items())
assert all(sha(ROOT / p) == h for p, h in shared.items() if p != old['document']['path'])
assert subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True).returncode == 0
print(json.dumps({'receipt': NAME, 'coverage': stats, 'new_findings': ['Q379'], 'authority_entries': receipt['authority_entries'],
    'audit_cases': 2, 'protected_sources': len({**guards, **receipt['source_sha256']}), 'compile_protected_stickleback_files': 30,
    'remaining_stickleback_files': len(receipt['remaining_stickleback_files']), 'retained_bytes': budget}, indent=2))
