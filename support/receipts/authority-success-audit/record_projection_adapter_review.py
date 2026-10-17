"""Checkpoint completed RDF projection, identity and two surface adapters."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
NAME = 'projection_adapter_review_receipt.json'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE / n).read_text(encoding='utf-8-sig'))
def save(n, value):
    (HERE / n).write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')
def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()

assert not (HERE / NAME).exists()
old = load('observation_ingest_review_receipt.json')
head = git('rev-parse', 'HEAD')
assert head == old['basis_commit']
guards = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}
assert len(guards) == 1831
assert all(sha(ROOT / n) == h for n, h in guards.items())
assert all(sha(HERE / n) == h for n, h in old['evidence_sha256'].items())
shared = old['shared_working_file_sha256']
doc = ROOT / old['document']['path']
prefix = doc.read_bytes()
assert sha(doc) == old['document']['sha256']
assert all(sha(ROOT / n) == h for n, h in shared.items() if n != old['document']['path'])
index = Path(git('rev-parse', '--git-path', 'index'))
index = index if index.is_absolute() else ROOT / index
index_hash = sha(index)
assert index_hash == old['git_index_sha256']
inv, cov = load('source_inventory.json'), load('read_coverage.json')
assert inv['commit'] == head and load('pending_read.json') == []
areas = []
for area, count, chars in [
    ('crates/graph/linked-data/', 20, 430036),
    ('crates/incipit/', 2, 11372),
    ('crates/inker/engines/graft-engine/', 4, 41060),
    ('crates/inker/engines/scrying-engine/', 5, 63083),
    ('crates/inker/knot-editor-host/', 4, 16801),
]:
    files = [n for n in inv['files'] if n.startswith(area)]
    assert len(files) == count and sum(inv['files'][n]['chars'] for n in files) == chars
    assert all(cov.get(n, {}).get('sha256') == inv['files'][n]['sha256'] and cov[n]['through'] == inv['files'][n]['chars'] and sha(ROOT / n) == inv['files'][n]['sha256'] for n in files)
    areas.append({'prefix': area, 'files': count, 'chars': chars, 'verification': NAME, 'basis_commit': head, 'qualification': 'Complete source read. Executed tests only for Incipit and stub/witness Graft adapter; remaining sources reviewed without full feature/runtime execution.'})
whole = sorted(n for n in inv['files'] if any(n.startswith(a['prefix']) for a in areas) and n not in guards)
assert len(whole) == 25 and sum(inv['files'][n]['chars'] for n in whole) == 364123
stats = json.loads(subprocess.check_output(['python', str(HERE / 'read_coverage.py'), 'status'], cwd=ROOT))
assert stats['complete_files'] == 2328 and stats['read_chars'] == 27573443 and not stats['changed_since_inventory'] and not stats['pending_chunks']
tests = load('identity_graft_original_suite_v2_receipt.json')
assert tests['builds'] == [{'crate': 'incipit', 'exit': 0}, {'crate': 'graft_engine', 'exit': 0}]
assert tests['runs'] == [{'crate': 'incipit', 'exit': 0, 'checks': 5}, {'crate': 'graft_engine', 'exit': 0, 'checks': 6}]
assert tests['generated_output_removed'] and all(not Path(p['path']).exists() for p in tests['removed_outputs'])
assert all(sha(ROOT / n) == h for n, h in tests['source_sha256'].items())
assert all(sha(Path(v['path'])) == v['sha256'] for v in tests['libraries'].values())
assert tests['runner_sha256'] == sha(HERE / 'run_identity_graft_original_suite_v2.py')
preflight = load('identity_graft_original_suite_preflight_receipt.json')
assert preflight['runner_sha256'] == sha(HERE / preflight['runner']) and preflight['builds'] == 0 and preflight['tests'] == 0

# These are exact source excerpts, not an emulated RDF or system-WebView test.
traces = []
for name, path, first, last, required in [
    ('held metadata selects the first parallel edge', 'crates/graph/linked-data/src/lib.rs', 643, 737, ['for target in graph.out_neighbors(key)', 'graph.find_edge_key(key, target)', 'payload.held_assertions()']),
    ('direct facts visit every outgoing bucket', 'crates/graph/linked-data/src/lib.rs', 520, 641, ['surface_outgoing_relation_buckets', 'held_assertions()']),
    ('one shared query and export dataset traversal', 'crates/graph/linked-data/src/lib.rs', 748, 876, ['pub fn dataset_quads(', 'pub(crate) fn dataset_quad_iter(', 'resource_metadata::quads']),
    ('kernel first arc contract', 'crates/graph/graph-kernel/src/graph/edge_ops.rs', 1000, 1018, ['Find the first directed edge', 'find_edge(from, to)']),
    ('kernel outgoing neighbor duplicates and topology filter', 'crates/graph/graph-kernel/src/graph/query.rs', 600, 611, ['has_held_topology()', '.map(|edge| edge.target())']),
    ('Scrying mouse kind ignores the changed button', 'crates/inker/engines/scrying-engine/src/translation.rs', 243, 279, ['mouse_kind_to_scrying(ev.kind)', 'InkerMouseEventKind::Pressed => (ScryingMouseEventKind::LeftButtonDown, 0)', 'Some(InkerMouseButton::Right) => vk.right_button = true']),
    ('Scrying producer forwards the mapped mouse event', 'crates/inker/engines/scrying-engine/src/producer.rs', 83, 87, ['send_mouse_input(map_mouse(ev))']),
]:
    text = (ROOT / path).read_text(encoding='utf-8')
    snippet = ''.join(text.splitlines(keepends=True)[first - 1:last])
    assert all(s in snippet for s in required), name
    traces.append({'name': name, 'path': path, 'first_line': first, 'last_line': last, 'source_sha256': sha(ROOT / path), 'excerpt': snippet})
trace_name = 'projection_adapter_source_traces.json'
assert not (HERE / trace_name).exists()
save(trace_name, {'qualification': 'Exact current source excerpts and API-contract trace only. No full RDF/SPARQL/file-I/O, system-WebView or native-input execution.', 'traces': traces})

notes, authority = load('review_notes.json'), load('authority_map.json')
assert notes['findings'][-1]['id'] == 'Q331' and authority['entries'][-1]['id'] == 'A882'
findings = [
    {'id': 'Q332', 'title': 'Held Surface assertion metadata omits parallel buckets that direct RDF facts include', 'source': 'crates/graph/linked-data/src/lib.rs:646,654,655,751,859', 'status': 'verified current source and kernel multigraph API contracts; full RDF runtime not executed', 'evidence': 'node_direct_quads traverses every surface_outgoing_relation_buckets payload. node_metadata_quads instead traverses out_neighbors and resolves each target with find_edge_key, explicitly documented by the kernel to select the first directed edge. With two parallel held Surface buckets, both fact predicates are exported but only the first bucket gets reifiers, statement IDs, attribution, lifecycle and basis metadata. Repeated target neighbors repeat the same first metadata and dataset dedup conceals that repetition. dataset_quad_iter has no later Surface metadata repair; JSON-LD, file serializers and borrowed/materialized SPARQL share it. The existing parallel-bucket test checks fact direction/scopes only. Existing round-trip and query parity checks compare projections through this same owner and cannot detect pre-projection omission. Ordinary Resource statements use the separate per-resource-edge path and are not accused by this finding.', 'diagnostics': [trace_name], 'repair_triage': {'disposition': 'documented under active read-only audit goal', 'reason': 'Use the same all-outgoing-buckets traversal for held metadata as direct facts and assert every known parallel handle/provenance against the source graph. This is a bounded local fix; full RDF dependencies are not present in the retained supplier set.'}},
    {'id': 'Q333', 'title': 'Scrying translates every mouse button press and release as a left click', 'source': 'crates/inker/engines/scrying-engine/src/translation.rs:243,253,268;crates/inker/engines/scrying-engine/src/producer.rs:83', 'status': 'verified unchanged current mapping and producer call chain; live system-WebView input not executed', 'evidence': 'map_mouse passes only ev.kind to mouse_kind_to_scrying, which unconditionally maps Pressed/Released to LeftButtonDown/LeftButtonUp. The changed button only sets virtual-key flags. Right, Middle, Back and Forward therefore reach the producer with left-button event kinds even though their flag is set. ScryingProducer forwards that mapped event and returns the backend Result. The original test covers Left only; pointer-events input is a different path. The adapter nevertheless labels mouse support Supported. This source finding does not claim a headed right-click replay or reproduce concrete backend behavior.', 'diagnostics': [trace_name], 'repair_triage': {'disposition': 'documented under active read-only audit goal', 'reason': 'Map the changed button together with phase, including the backend X-button selector, and exercise non-left pairs through the adapter. Confirm wheel axes separately; do not broaden into a full input redesign.'}},
]
for f in findings:
    f['verification'] = NAME
notes['findings'].extend(findings)
rows = [
    ('RDF projection has two held Surface walks with different completeness', 'Every held exact assertion is advertised as predicate plus reifier.', ['crates/graph/linked-data/src/lib.rs'], 'Q332 disproves this for parallel Surface buckets. The base and metadata walkers implement independent selection recipes; only the former uses the all-buckets kernel API. History separately walks all buckets, but only for withdrawn assertions.'),
    ('Projection parity is a shared-owner oracle', 'Borrowed SPARQL matches a materialized spareval Dataset.', ['crates/graph/linked-data/src/query.rs', 'crates/graph/linked-data/src/query/dataset.rs', 'crates/graph/linked-data/src/lib.rs'], 'Both use dataset_quad_iter through dataset_quads, so matching results prove adapter/evaluator parity rather than completeness against retained kernel assertions. Pattern matching deduplicates exact quads; broad patterns rescan the full projection and build node-local vectors.'),
    ('Resource metadata remains a distinct canonical content projection', 'Shared Resource properties, affirmative rdf:type and complete tag definitions project independently of appearances.', ['crates/graph/linked-data/src/resource_metadata.rs', 'crates/graph/linked-data/src/lib.rs'], 'Source tests cover resource-only graphs, multiple attributable literals, exact handles and four formats. Legacy Surface metadata stays a second path. The new parallel omission is not inferred for per-resource-edge iteration. Tests read here were not executed with the full RDF dependency graph.'),
    ('Reifier handle transport is shared and explicitly reversible', 'One module distinguishes legacy valid IDs from encoded unsafe IDs and malformed reserved formats.', ['crates/graph/linked-data/src/reifier.rs', 'crates/graph/linked-data/src/ingest.rs', 'crates/graph/linked-data/src/lib.rs'], 'Prefixes are distinct, so encoded-looking valid handles do not collide with encoded unsafe strings. Source tests check opaque/newline/NUL/empty handles, malformed UTF-8/hex refusal, actual file I/O and query exposure. This is real sharing, not duplicated codec authority; full parser tests were not run.'),
    ('File import erases by graph name instead of checking alignment membership', 'The vocabulary graph is described as re-derivable schema and all of it is dropped.', ['crates/graph/linked-data/src/serialize.rs', 'crates/graph/linked-data/src/vocab.rs', 'crates/graph/linked-data/src/lib.rs'], 'is_vocabulary_quad checks graph name only. GraphScope::Custom accepts the same vocabulary IRI, so foreign extra facts or instance metadata placed there can disappear from N-Quads/TriG input with no refusal. Document reserved-name admission and narrow filtering policy before choosing a repair. JSON-LD does not pass through this file-only filter.'),
    ('Vocabulary alignment delegates relation identity to the kernel', 'One exhaustive enum match categorizes exact, approximate and Mere-only alignments.', ['crates/graph/linked-data/src/vocab.rs'], 'Canonical predicate and recognized-kind enumeration come from kernel, while standards alignment belongs here. No inference engine is implemented by merely appending schema quads. Serialization prose calls the projection sorted, but dataset order is not a global normalized sort; compare normalized datasets when claiming byte stability.'),
    ('Linked-data completion prose contains stale capability deferrals', 'The crate still calls rdf:type mapping future work and says the kernel lacks raw standalone predicates.', ['crates/graph/linked-data/src/lib.rs', 'crates/graph/linked-data/src/statements.rs', 'crates/graph/linked-data/src/ingest/apply.rs'], 'Current ingest and tests implement rdf:type and raw predicates. The statements helper still intentionally recognizes only vocabulary slugs; that limitation belongs to this helper rather than the kernel. It retains unrecognized LinkStatements, but an absent source returns an all-zero outcome without reporting supplied statements.'),
    ('Pending extraction and retained truth remain separate owners', 'Inker extracts LinkStatements; linked-data resolves vocabulary and queues/retries; kernel owns admission and replay suppression.', ['crates/graph/linked-data/src/statements.rs', 'crates/graph/linked-data/src/statements_pending_tests.rs'], 'Source tests cover arrival, resource-only targets, navigation ownership, ambiguity, purge, prepared identities, recorded handles before ambient derivation and suppression after panic/clone. Rebuild adds current inputs without clearing the cache. apply_link_statements also adds assertions from retrying the global pending queue, so edges_asserted is not exclusively a count for the supplied source/list.'),
    ('Query display strings are distinct from identity-bearing membership', 'SELECT/ASK cells expose bare lexical strings; saved membership requires NamedNode cells and resolves held Resource IRIs.', ['crates/graph/linked-data/src/query.rs', 'crates/graph/linked-data/src/query/resource_member_tests.rs'], 'The ordinary display loses datatype/language identity by design. Membership retains typed result checks before string display, deduplicates IDs and reports missing resources with coverage; it does not invent graph records. No claimed SPARQL conformance from reading these tests alone.'),
    ('Incipit transports a citation without becoming a resolver', 'Shelfmark stores opaque adapter locators, per-role generations, reading parameters and target-owned deltas.', ['crates/incipit/src/lib.rs'], 'validate checks schema/nonempty fields, not adapter existence, authorization, generation agreement or resolution. Deserialize does not invoke validate. Ordered maps stabilize serialization, and GraphId/SessionId are distinct nominal wrappers. Five freshly compiled original tests pass over exact retained serde/uuid/json suppliers.'),
    ('Graft exposes a host seam rather than embedding another Servo', 'The factory supplies a non-Send composite surface; the adapter forwards owned frame custody and typed controls.', ['crates/inker/engines/graft-engine/Cargo.toml', 'crates/inker/engines/graft-engine/src/engine.rs', 'crates/inker/engines/graft-engine/src/producer.rs'], 'Dependency graph remains Inker/tracing only. Accessibility and other unimplemented defaults explicitly refuse controls; set_offset is a documented composited-texture no-op. Six fresh stub/witness tests pass, including ordering/refusal/semantic activation, not a live Servo/GPU acceptance receipt.'),
    ('Scrying capability projection distinguishes backend features from exposed controls', 'Adapter-specific unsupported/partial statuses replace optimistic platform claims for unwired operations.', ['crates/inker/engines/scrying-engine/src/translation.rs', 'crates/inker/engines/scrying-engine/src/producer.rs'], 'Find/typed zoom/capture/PDF/permissions/auth/IME/accessibility/delete are explicitly unsupported; popup/download request events remain partial. Cookie reads are narrowed by platform. These are honest local limits, but Q333 shows the hardcoded Supported mouse status overlooks incorrect non-left translation.'),
    ('Scrying frame ownership stays intact across type erasure', 'A concrete owned native frame is carried in OwnedPayload until the selected host importer consumes it.', ['crates/inker/engines/scrying-engine/src/translation.rs', 'crates/inker/engines/scrying-engine/src/producer.rs'], 'Wrong payload recovery returns the full original SurfaceFrame. CPU/PNG/overlay frames map to None, while capability transport still mirrors backend preferred mode. Empty-poll recovery counts only None from try_acquire_frame, so repeated Some with unsupported frame variants can mask a texture stall. Trace a real composited consumer before promoting a product bug.'),
    ('Scrying keeps two independently maintained navigation translators', 'Legacy map_navigation_event coexists with richer ordered map_web_event/map_web_surface_event.', ['crates/inker/engines/scrying-engine/src/translation.rs'], 'Legacy TitleChanged becomes Finished with an empty URL; richer path preserves TitleChanged. SourceChanged uses Some(AddressChanged).or(Some(Committed)), making the Committed fallback unreachable. Ordered correlated request IDs survive; future unsupported events become diagnostics. Follow current host event consumers before choosing the lifecycle repair.'),
    ('Knot readout derives from the host buffer and editor retains an explicit save checkpoint', 'KnotReadout holds only engine/lexer machinery; KnotEditor owns one TextInput plus the accepted original text.', ['crates/inker/knot-editor-host/src/editor.rs', 'crates/inker/knot-editor-host/src/model.rs'], 'Dirty status compares actual source to an explicit accepted-source checkpoint, preserving edits after an async save. There is no path/store/writer in this model, and preview delegates to Nematic instead of reparsing document content independently. Original string is a checkpoint, not a competing editable authority.'),
    ('Host lexer pack overrides the portable parser floor', 'Markdown syntax uses pulldown-cmark offset events over the portable Illume registry.', ['crates/inker/knot-editor-host/src/lib.rs', 'crates/inker/knot-editor-host/Cargo.toml'], 'full_pack adds three aliases; CSS/HTML/JS/Turtle precision remains future work. Start/end tracked constructs use parallel exhaustive-looking lists and one stack rather than a second structural model. These tests were read; full Knot/Cambium/preview feature graph was not built.'),
]
entries = [{'id': f'A{883 + i}', 'domain': d, 'claim': c, 'implementations': paths, 'assessment': a, 'state': 'complete named source reads; qualified Incipit/Graft original tests only', 'verification': NAME} for i, (d, c, paths, a) in enumerate(rows)]
authority['entries'].extend(entries)
notes['completed_source_areas'].extend(areas)
for item in notes.get('partial_source_areas', []):
    if item['prefix'] == 'crates/graph/linked-data/':
        item['superseded_by_complete_review'] = NAME
notes['review_followups'].extend([
    {'verification': NAME, 'area': 'Reserved RDF graph admission versus filtering', 'detail': 'File imports drop every quad with the vocabulary graph IRI, although Custom scope can name it. Decide explicit reservation/refusal or filter only actual schema alignment; do not silently choose namespace policy. No live consumer/file round trip was executed.'},
    {'verification': NAME, 'area': 'Scrying event and transport mismatch', 'detail': 'Trace SourceChanged consumers before fixing the unreachable Committed fallback. Preferred CpuSnapshot can coexist with an adapter that returns no frame for CPU data, and repeated Some unsupported frames reset stall recovery. Keep actual mode/host proof separate from source paths.'},
    {'verification': NAME, 'area': 'Statements outcome boundaries', 'detail': 'Absent source returns an empty outcome without reporting supplied statements; edges_asserted also adds retries from the global pending cache. Establish current callers before claiming false product success. Raw/type future-work prose is stale.'},
])
next_read = {'path': 'crates/inker/engines/weld-engine/Cargo.toml', 'start': 0, 'reason': 'Read remaining Weld adapter, then every unread source including intelligence/search/inference and product ports.'}
notes['current_area'] = 'Linked-data, Incipit, Graft, Scrying and Knot editor host source complete'
notes['next_area'] = next_read['reason']
notes['checkpoint'] = {'classification': 'progress', 'classification_of_previous_turn': 'progress', 'goal_status': 'active; full source incomplete', 'coverage': stats, 'finding_count': len(notes['findings']), 'authority_entries': len(authority['entries']), 'new_receipts': [NAME, trace_name, 'identity_graft_original_suite_v2_receipt.json'], 'audit_authored_production_edits_this_turn': False, 'next_read': next_read, 'pending_verification': None, 'diagnostic_outcomes': 'Fresh original Incipit five and Graft six tests pass over retained suppliers. Q332/Q333 are verified source paths only, not full RDF or system-WebView runtime. Outputs removed.'}
diff = subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True, text=True)
assert diff.returncode == 0
with (HERE / 'projection_adapter_diff_check.txt').open('x', encoding='utf-8') as f:
    f.write(diff.stdout + diff.stderr)
addition = f'''

### RDF projection completeness and engine adapters, 2026-10-17

Twenty-five further files / 364,123 characters are fully read at unchanged HEAD {head}. Linked-data is now complete (20 files / 430,036 characters), along with Incipit (2 / 11,372), Graft (4 / 41,060), Scrying (5 / 63,083), and Knot editor host (4 / 16,801). Overall coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters. Current source hashes match; no chunk is pending. The full repository review remains active and incomplete.

Q332 records a source-verified completeness defect: direct RDF facts traverse every parallel Surface bucket, but held metadata repeatedly selects only the first edge to each outgoing neighbor. Other held buckets lose assertion reifiers, provenance, lifecycle and basis metadata. The kernel explicitly documents the first-edge selection and provides the all-buckets API already used by direct facts. JSON-LD, file export and both borrowed/materialized query oracles share this projection, so parity between those paths does not prove completeness. The existing parallel-bucket test checks base facts and direction, not all source handles. Resource-edge projection is separate and is not accused by this finding.

Q333 records another source-verified adapter defect: Scrying maps all Pressed/Released mouse inputs to LeftButtonDown/LeftButtonUp, while using the changed button only for virtual-key flags. Right/middle/back/forward therefore retain the wrong event kind. The adapter advertises mouse support and forwards this mapping; its source test exercises left clicks only. Neither finding is labeled an executed full RDF or native-input replay. Exact excerpts and current kernel API contracts are retained in [source traces](../../../support/receipts/authority-success-audit/{trace_name}).

Fresh unchanged original Incipit tests (5) and Graft adapter tests (6) pass over exact retained supplier artifacts. Graft uses explicit stub/witness surfaces; no live Servo, system-WebView, GPU import or headed result is claimed. An initial preflight refused two serde feature rows before compiling anything; its script/hash/failure is retained, and the distinct v2 runner selects the rc-enabled row. Generated executables/debug outputs were marker-checked and removed.

A883-A898 record the independent Surface metadata walker, shared-owner test oracles, canonical Resource metadata, shared reversible reifier codec, broad vocabulary-graph filtering, stale raw/type deferrals, pending-input custody, display versus identity query results, opaque shelfmark custody, Graft host seam, adapter-local Scrying limits and native-frame ownership, dual navigation translators, source-buffer/save-checkpoint ownership and host lexer overrides. Hairier follow-ups remain explicit: reserved RDF namespace admission versus silently dropping foreign quads, exact import's whole-snapshot cost/refusal reporting, attribution/literal fidelity, SourceChanged's unreachable Committed fallback, preferred CPU transport versus unsupported frame mapping, and global pending retry counts. These are documented without taking a broad redesign detour.

The [checkpoint](../../../support/receipts/authority-success-audit/{NAME}) preserves all prior prose as an exact prefix and protects current reviewed source, shared WIP and index. No production edits, new worktree, Cargo home or isolated target. Continue the unread Weld adapter and remaining source tree.
'''
doc.write_bytes(prefix + addition.encode('utf-8'))
save('review_notes.json', notes)
save('authority_map.json', authority)
evidence = ['record_projection_adapter_review.py', trace_name, 'projection_adapter_diff_check.txt', 'run_identity_graft_original_suite.py', 'prepare_identity_graft_v2.py', 'run_identity_graft_original_suite_v2.py', 'identity_graft_original_suite_preflight_receipt.json', 'identity_graft_original_suite_v2_receipt.json', *('identity_graft_original_suite_v2_' + c + '_' + suffix + '.log' for c in ['incipit', 'graft_engine'] for suffix in ['compile', 'output'])]
budget = sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file())
assert budget < 100_000_000
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': head, 'kind': 'RDF projection and two surface adapters source complete; full source review incomplete', 'prior_receipt': {'path': 'observation_ingest_review_receipt.json', 'sha256': sha(HERE / 'observation_ingest_review_receipt.json')}, 'coverage': stats, 'completed_source_areas': areas, 'whole_current_files': whole, 'source_sha256': {n: sha(ROOT / n) for n in whole}, 'preserved_production_and_test_hashes': guards, 'shared_working_file_sha256': shared, 'new_findings': [f['id'] for f in findings], 'authority_entries': [e['id'] for e in entries], 'audit_authored_production_edits_this_pass': False, 'diagnostics': {'identity_graft_original_suite_v2_receipt.json': {'builds': tests['builds'], 'runs': tests['runs']}, trace_name: {'kind': 'source/API-contract trace only'}}, 'pending_verification': None, 'next_read': next_read, 'document': {'path': old['document']['path'], 'sha256': sha(doc), 'preserved_prefix_bytes': len(prefix), 'preserved_prefix_sha256': hashlib.sha256(prefix).hexdigest()}, 'evidence_sha256': {n: sha(HERE / n) for n in evidence}, 'git_index_sha256': index_hash, 'retained_isolation': [], 'receipt_budget': {'cap_bytes': 100_000_000, 'retained_bytes_before_receipt': budget}, 'limits': 'Eleven fresh original test executions over exact retained supplier artifacts; Graft uses stub/witness surfaces. Findings verified by current source and API contract trace. No full linked-data/SPARQL/file-I/O, live Servo/system-WebView/native input, GPU or product acceptance.'}
save(NAME, receipt)
assert sha(index) == index_hash and doc.read_bytes().startswith(prefix)
assert all(sha(ROOT / n) == h for n, h in {**guards, **receipt['source_sha256']}.items())
assert all(sha(HERE / n) == h for n, h in receipt['evidence_sha256'].items())
assert all(sha(ROOT / n) == h for n, h in shared.items() if n != old['document']['path'])
assert subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True).returncode == 0
print(json.dumps({'receipt': NAME, 'coverage': stats, 'new_findings': receipt['new_findings'], 'authority_entries': receipt['authority_entries'], 'protected_sources': len({**guards, **receipt['source_sha256']}), 'retained_bytes': budget}, indent=2))
