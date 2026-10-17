"""Complete lexical source review, six Commons files, and bounded original tests."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
NAME = 'lexical_commons_review_receipt.json'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def save(n, value):
    (HERE/n).write_text(json.dumps(value, indent=2)+'\n', encoding='utf-8')
def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()

assert not (HERE/NAME).exists()
old = load('retained_host_reread_receipt.json')
head = git('rev-parse', 'HEAD')
assert head == old['basis_commit']
guards = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}
aux = old['auxiliary_read_sha256']
assert len(guards) == 1946
assert all(sha(ROOT/n) == h for n, h in {**guards, **aux}.items())
assert all(sha(HERE/n) == h for n, h in old['evidence_sha256'].items())
shared = old['shared_working_file_sha256']
doc = ROOT/old['document']['path']
prefix = doc.read_bytes()
assert sha(doc) == old['document']['sha256']
assert all(sha(ROOT/n) == h for n, h in shared.items() if n != old['document']['path'])
index = Path(git('rev-parse', '--git-path', 'index'))
if not index.is_absolute():
    index = ROOT/index
index_hash = sha(index)
assert index_hash == old['git_index_sha256']
inv, cov = load('source_inventory.json'), load('read_coverage.json')
assert inv['commit'] == head and load('pending_read.json') == []
stats = json.loads(subprocess.check_output(['python', str(HERE/'read_coverage.py'), 'status'], cwd=ROOT))
assert stats['complete_files'] == 2429 and stats['read_chars'] == 28898340
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
files = sorted(n for n in inv['files'] if n.startswith(('crates/intel/mora-cmudict/', 'crates/intel/reference-data/')))
commons = ['crates/moot/commons/'+n for n in ['Cargo.toml', 'examples/commons_rf_fixture.rs', 'src/call.rs', 'src/chat.rs', 'src/encrypted.rs', 'src/keys.rs']]
files = sorted(files+commons)
assert len(files) == 13 and sum(inv['files'][n]['chars'] for n in files) == 301789
assert all(cov[n]['sha256'] == sha(ROOT/n) == inv['files'][n]['sha256'] and cov[n]['through'] == inv['files'][n]['chars'] for n in files)
areas = [{'prefix': area, 'files': len(members := [n for n in files if n.startswith(area)]), 'chars': sum(inv['files'][n]['chars'] for n in members), 'verification': NAME, 'basis_commit': head, 'qualification': 'Every inventoried source file fully emitted, reviewed and acknowledged; exact retained supplier test scope is recorded separately.'} for area in ['crates/intel/mora-cmudict/', 'crates/intel/reference-data/']]
probe = load('lexical_original_tests_receipt.json')
assert probe['basis_commit'] == head and probe['generated_output_removed']
assert all(r['build_exit'] == r['run_exit'] == 0 and r['failed'] == 0 for r in probe['results'].values())
assert probe['results']['reference-data']['passed'] == 23 and probe['results']['reference-data']['ignored'] == 1
assert probe['results']['mora-cmudict']['passed'] == 2
assert probe['runner_sha256'] == sha(HERE/'run_lexical_original_tests.py')
assert all(sha(ROOT/n) == h for n, h in {**probe['source_sha256'], **probe['runtime_input_sha256']}.items())
assert all(sha(Path(v['path'])) == v['sha256'] for v in probe['libraries'].values())
assert all(not Path(v['path']).exists() for v in probe['removed_outputs'])
aux = {**aux, 'crates/intel/reference-data/fixtures/bilingual.json': probe['runtime_input_sha256']['crates/intel/reference-data/fixtures/bilingual.json']}
runtime = {**probe['runtime_input_sha256'], 'Cargo.lock': sha(ROOT/'Cargo.lock')}
notes, authority = load('review_notes.json'), load('authority_map.json')
assert notes['findings'][-1]['id'] == 'Q351' and authority['entries'][-1]['id'] == 'A980'
finding = {
    'id': 'Q352',
    'title': 'Encrypted edit splits its promised authoring key snapshot and discards a not-inserted result',
    'source': 'crates/moot/commons/src/encrypted.rs:166,173,187,363,367,380;crates/moot/commons/src/keys.rs:34',
    'status': 'source-confirmed key snapshot split and ignored insertion outcome; rotation-race execution not yet reproduced',
    'evidence': 'GroupKeys::current documents one snapshot per authoring turn. EncryptedReplica::edit takes keys before awaiting load, builds and seals against that Arc, then calls self.accept. accept_into takes keys.current again and may park an operation naming an unheld epoch, returning Ok(false). edit applies ? only to the Result, discards the bool, and returns Ok(operation), despite documenting that it signs and stores the edit. A key-handle replacement between the snapshots can therefore choose different sealing and admission owners. Its public edit closure can itself capture and replace a GroupKeys clone, giving a bounded deterministic reproduction seam without a timing race. Existing tests rotate between completed edits.',
    'scope': 'Complete current encrypted.rs and keys.rs reviewed. The two snapshots, false-return path and ignored bool are directly source-verified. No fresh Commons compilation, callback-rotation fixture, live lane or observed user data loss in this pass.',
    'verification': NAME,
    'repair_triage': {'disposition': 'documented under active read-only full-source audit; retain public API reproduction as next Commons diagnostic', 'reason': 'Use the same captured keyring through sealing and local admission, and preserve an explicit inserted/parked/refused outcome. Reproduce by replacing the shared handle inside the edit closure, with an unchanged-handle successful insertion control. Check lifecycle and host rotation policy before broadening the fix across chat and joined lanes.'},
}
notes['findings'].append(finding)
rows = [
    ('Pronunciation data is a provider above the phone engine', 'CMUdict parsing and spelling normalization are owned by mora-cmudict, while phones/rhyme come from Mora.', ['crates/intel/mora-cmudict/Cargo.toml', 'crates/intel/mora-cmudict/src/lib.rs'], 'The provider delegates CMUdict entry and phone parsing to Mora, retains distinct variant phone lists and normalizes only trim plus ASCII case. Embedded dictionary initialization is OnceLock-backed. Two original tests freshly pass, including the full shipped dictionary parse and a real cat/hat rhyme; this does not establish all-language pronunciation quality or arbitrary word normalization.'),
    ('Lexical pack integrity is explicit and separate from trust', 'ValidatedPack checks schema, bounds, payload digest, entry/sense identities and in-pack relations.', ['crates/intel/reference-data/src/lib.rs', 'crates/intel/reference-data/src/validation.rs'], 'Private ValidatedPack storage has one JSON admission path; callers cannot construct validated state by deriving Deserialize. Digest covers struct-order entries, preserves list order, and intentionally is not an authenticity signature. The raw NormalizedPack remains public data. Metadata URL checks are shaped HTTP(S) metadata checks, not transport acquisition or origin trust.'),
    ('Registry catalog, install and enable are distinct durable states', 'SourceRegistry validates before publication and updates memory after successful publication.', ['crates/intel/reference-data/src/registry.rs', 'crates/intel/reference-data/src/tests.rs'], 'Advertise is inert, imported packs start disabled, reimport cannot replace an installed version and exact lookup requires explicit enabled sources. Bounded file reads and content-hash pack filenames avoid supplied identifier paths. Twenty-three original tests freshly pass on Windows, including corruption refusal, failed publication, reopen limits and advisory lock exit. Directory fsync and symlink tests are Unix-specific and unexecuted here; no power-cut claim.'),
    ('Lexical byte bounds are owner-specific', 'Registry bounds referenced installed bytes and metadata, not every file ever placed in its directory.', ['crates/intel/reference-data/src/registry.rs', 'crates/intel/reference-data/src/validation.rs'], 'A failed publication may intentionally leave an inert unreferenced immutable pack, as documented. Reopen counts referenced packs before allocating them. The caller supplies a private trusted directory, so source ID path checks are not a general hostile-filesystem ownership guarantee. No leak or total process-memory limit is asserted.'),
    ('Call fold owns control state without admission or audio I/O', 'Retained invitation/terminal facts and expiring live controls are different grammars.', ['crates/moot/commons/src/call.rs'], 'Sender-local sequence deduplication detects equivocation; terminal facts dominate nonterminal frames and deterministic semantic ranking avoids wall-clock settlement. Connected is control-derived, not measured media delivery. AudioParameters validate a wire description, not codec support or negotiated format overlap. Original permutation/property/expiry tests were read, not run this pass.'),
    ('RF fixture verification is a byte and signature boundary', 'commons_rf_fixture verifies received canonical operation bytes and decrypts a fixed expected event.', ['crates/moot/commons/examples/commons_rf_fixture.rs'], 'The example itself does not observe a radio, PHY or transfer provenance; its DIRECT-PHY success phrase needs an independently observed carriage receipt. It reads fixture-supplied key state and expected operation and checks exact canonical bytes, signature/body and plaintext. No fresh RF example, network or physical execution in this pass.'),
    ('Chat structural admission and communal authority projection are explicit layers', 'projection_with_authority classifies stable Personae roots and retains pending/revoked facts separately.', ['crates/moot/commons/src/chat.rs'], 'Derived-key attestation checks bind operation signer to stable root and chat-space salt. Structural storage deliberately uses AllowAllAuthority; communal display must call projection_with_authority. The permissive compatibility projection is not a current membership decision. Original revocation tests distinguish storage from visibility and cover checkpointed contributions; those tests were read rather than executed.'),
    ('Checkpoint replay has a shared content owner but a separate authority lifetime', 'Chat checkpoints rebuild content before admission and retain covered author facts for later capability reevaluation.', ['crates/moot/commons/src/chat.rs'], 'Admission checks chain, signed/encrypted frontier agreement, content rebuild and current authority revision. Replay reads the signed chain and decrypts only its tip, relying on admission of predecessor content. Joined lane captures a clone of checkpoint_authority once, while GroupKeys stays refreshable; tracing host rejoin/update policy remains required before claiming stale authority is accepted in a live product.'),
    ('One replay load is not an exclusive store transaction', 'Chat author_onto says nothing else can have written between its snapshot and admission.', ['crates/moot/commons/src/chat.rs'], 'The same retained snapshot is actually reused and avoids repeated full replay. However sync_store and joined accept closures hold clones of the backing store, and multiple awaited store reads are not an exclusive snapshot transaction in this module. Concurrent-store admission/author-head safety requires the processor/store owner trace; no lost write or fork race is promoted from this comment alone.'),
    ('Encrypted and plaintext graph profiles share batch and projection meaning', 'EncryptedReplica decrypts into the root CommonsRecord and calls shared batch/frontier/fold functions.', ['crates/moot/commons/src/encrypted.rs', 'crates/moot/commons/src/keys.rs'], 'Different signed header type, marker and lane kind are intentional wire boundaries. Shared author_batch/check_batch/validate_counter_frontier/fold_with_authority avoid duplicating graph meaning. GroupKeys clones share one RwLock over Arc snapshots and remain unchanged after replacement; Q352 identifies the authoring call path that takes two snapshots. Key rotation between completed edits in the source tests does not cover that gap.'),
]
entries = [{'id': f'A{981+i}', 'domain': d, 'claim': c, 'implementations': paths, 'assessment': assessment, 'state': 'thirteen complete source files; 25 fresh lexical original tests; Commons source-only this pass', 'verification': NAME} for i, (d, c, paths, assessment) in enumerate(rows)]
authority['entries'].extend(entries)
notes['completed_source_areas'].extend(areas)
notes.setdefault('partial_source_areas', []).append({'prefix': 'crates/moot/commons/', 'verification': NAME, 'fully_read_files': commons, 'remaining': [n for n in inv['files'] if n.startswith('crates/moot/commons/') and n not in files], 'qualification': 'Six files fully reviewed, including entire chat and encrypted implementations/tests. Root graph profile, parking and pruning remain unread.'})
notes['review_followups'].extend([
    {'verification': NAME, 'area': 'Commons authoring key snapshot reproduction', 'detail': 'Q352: compile current Commons owner and run public EncryptedReplica::edit with a GroupKeys replacement inside the supplied edit closure. Assert actual insertion/parking outcome and compare an unchanged-handle control. Available lexical suppliers do not constitute a compiled Commons owner; no full model/network build is needed conceptually.'},
    {'verification': NAME, 'area': 'Joined checkpoint authority refresh', 'detail': 'ChatReplica::set_checkpoint_authority replaces a local Option while join captured a clone at join time. Trace host policy and rejoin before promoting live stale-authority acceptance; the refreshed GroupKeys path is a different lifetime.'},
    {'verification': NAME, 'area': 'Chat retained-store concurrency', 'detail': 'author_onto reuses a load but also exposes sync_store and joined cloned writers. Trace OperationProcessor and Muniment transaction/locking semantics before relying on the comment that nothing else can have written or claiming an observed concurrent author-head fork.'},
])
next_read = {'path': 'crates/moot/commons/src/lib.rs', 'start': 0, 'reason': 'Complete Commons graph profile, parking and pruning; bound the Q352 reproduction against its actual owner.'}
notes['current_area'] = 'Lexical providers complete; Commons call/chat/encrypted/key source reviewed'
notes['next_area'] = next_read['reason']
notes['checkpoint'] = {'classification': 'progress', 'classification_of_previous_turn': 'progress', 'goal_status': 'active; full source incomplete', 'coverage': stats, 'finding_count': len(notes['findings']), 'authority_entries': len(authority['entries']), 'new_receipts': [NAME, 'lexical_original_tests_receipt.json'], 'audit_authored_production_edits_this_turn': False, 'next_read': next_read, 'pending_verification': 'Q352 public callback-rotation reproduction; not a live process', 'diagnostic_outcomes': '25 original lexical tests pass. One ignored reference helper is invoked by its parent subprocess-exit test; Unix-only test absent on Windows. No fresh Commons build or execution.'}
diff = subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True, text=True)
assert diff.returncode == 0
with (HERE/'lexical_commons_diff_check.txt').open('x', encoding='utf-8') as f:
    f.write(diff.stdout+diff.stderr)
addition = f'''

### Lexical providers and Commons source review, 2026-10-17

Thirteen newly read source files /301,789characters at unchanged HEAD {head} raise current valid coverage to {stats['complete_files']:,}/{stats['total_files']:,}files and {stats['read_chars']:,}/{stats['total_chars']:,}characters. Mora-cmudict and reference-data are complete. Six Commons files are fully read, including the entire call, chat, encrypted-graph and refreshable-key code plus their embedded tests. Commons root graph profile, parking and pruning remain unread; the full repository audit is active and incomplete.

Q352 is source-confirmed: EncryptedReplica::edit captures a key snapshot, loads/builds/seals against it, then self.accept captures another. Admission can park and return Ok(false), but edit discards that bool and returns the operation as success. This contradicts the one-authoring-turn key snapshot promise and stored-edit wording. A public edit callback can replace the shared GroupKeys handle between the reads. That deterministic public API reproduction remains pending; no fresh Commons compilation or observed user data loss is claimed. The contained repair candidate is one captured keyring through local seal/admission with an explicit insertion outcome.

A981-A990 separate actual sharing from broader claims. Mora owns phones; CMUdict is a distinct data/spelling provider. ValidatedPack privately owns its invariant-checked form, while SourceRegistry owns inert catalog, disabled installation, explicit enablement, exact source lookup and publication. Integrity digests do not provide source authenticity. Encrypted Commons uses the root batch/frontier/projection functions behind a distinct wire marker/lane. Chat deliberately separates retained structural admission from communal authority projection. Its joined checkpoint authority is captured at join while encryption keys refresh; host lifecycle policy remains a followup. Reusing one replay load does not by itself serialize cloned store writers. The RF fixture verifies bytes/signature/plaintext, with physical carriage requiring separate evidence.

Twenty-five unchanged original lexical tests freshly pass:23reference-data and2mora-cmudict, including the bundled dictionary parse, registry state/reopen, corruption refusal, failed publication, byte bounds and advisory lock release on child process exit. The ignored subprocess helper runs through its parent test. Compilation uses exact retained serde/serde_json/blake3/mora suppliers; no fresh Cargo graph, Unix symlink/directory-fsync, power-cut, import UI, Commons lane, radio or media run. Original Commons tests were fully read where emitted, not executed. Generated binaries/debug files and owned temporary fixtures were hashed/checked and removed.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}) and [original lexical tests](../../../support/receipts/authority-success-audit/lexical_original_tests_receipt.json). The bilingual JSON fixture was read; the bundled dictionary was hash-bound and exercised, not given manual source-read credit. No production edits. Protected sources, auxiliary/runtime data, prior evidence, shared WIP and Git index remain preserved; prior document bytes are an exact prefix. Diff check passes. No retained isolated target, Cargo home, worktree or new build output. Next: Commons root graph, parking and pruning.
'''
doc.write_bytes(prefix+addition.encode('utf-8'))
save('review_notes.json', notes)
save('authority_map.json', authority)
evidence = ['record_lexical_commons_review.py', 'lexical_commons_diff_check.txt', 'run_lexical_original_tests.py', 'lexical_original_tests_reference-data_compile.log', 'lexical_original_tests_reference-data_output.log', 'lexical_original_tests_mora-cmudict_compile.log', 'lexical_original_tests_mora-cmudict_output.log', 'lexical_original_tests_receipt.json']
budget = sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file())
assert budget < 100_000_000
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': head, 'kind': 'Lexical providers complete; six Commons files source-reviewed; repository incomplete', 'prior_receipt': {'path': 'retained_host_reread_receipt.json', 'sha256': sha(HERE/'retained_host_reread_receipt.json')}, 'coverage': stats, 'completed_source_areas': areas, 'whole_current_files': files, 'source_sha256': {n: sha(ROOT/n) for n in files}, 'auxiliary_read_sha256': aux, 'runtime_input_sha256': runtime, 'preserved_production_and_test_hashes': guards, 'shared_working_file_sha256': shared, 'new_findings': ['Q352'], 'authority_entries': [e['id'] for e in entries], 'audit_authored_production_edits_this_pass': False, 'diagnostics': {'lexical_original_tests_receipt.json': probe['results']}, 'pending_verification': notes['checkpoint']['pending_verification'], 'next_read': next_read, 'document': {'path': old['document']['path'], 'sha256': sha(doc), 'preserved_prefix_bytes': len(prefix), 'preserved_prefix_sha256': hashlib.sha256(prefix).hexdigest()}, 'evidence_sha256': {n: sha(HERE/n) for n in evidence}, 'git_index_sha256': index_hash, 'retained_isolation': [], 'receipt_budget': {'cap_bytes': 100_000_000, 'retained_bytes_before_receipt': budget}, 'limits': probe['qualification']+' Q352 is source-confirmed only; Commons root/parking/pruning and other repository code remain unread.'}
save(NAME, receipt)
assert sha(index) == index_hash and doc.read_bytes().startswith(prefix)
assert all(sha(ROOT/n) == h for n, h in {**guards, **receipt['source_sha256'], **aux, **runtime}.items())
assert all(sha(HERE/n) == h for n, h in receipt['evidence_sha256'].items())
assert all(sha(ROOT/n) == h for n, h in shared.items() if n != old['document']['path'])
assert subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True).returncode == 0
print(json.dumps({'receipt': NAME, 'coverage': stats, 'new_findings': receipt['new_findings'], 'authority_entries': receipt['authority_entries'], 'protected_sources': len({**guards, **receipt['source_sha256']}), 'original_tests': 25, 'retained_bytes': budget}, indent=2))
