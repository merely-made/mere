"""Checkpoint complete Servitor reading, original unit run and public cases."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
NAME = 'servitor_review_receipt.json'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE / n).read_text(encoding='utf-8-sig'))
def save(n, value):
    with (HERE / n).open('x', encoding='utf-8') as f:
        json.dump(value, f, indent=2); f.write('\n')
def git(*args): return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()

assert not (HERE / NAME).exists()
old = load('script_hosts_review_receipt.json')
head = git('rev-parse', 'HEAD'); assert head == old['basis_commit']
guards = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}
assert len(guards) == 2098
aux, runtime = old['auxiliary_read_sha256'], old['runtime_input_sha256']
assert all(sha(ROOT / p) == h for p, h in {**guards, **aux, **runtime}.items())
assert all(sha(HERE / p) == h for p, h in old['evidence_sha256'].items())
assert all(sha(Path(p)) == h for p, h in old['external_contract_sha256'].items())
shared = old['shared_working_file_sha256']
doc = ROOT / old['document']['path']; prefix = doc.read_bytes()
assert sha(doc) == old['document']['sha256']
assert all(sha(ROOT / p) == h for p, h in shared.items() if p != old['document']['path'])
index = Path(git('rev-parse', '--git-path', 'index'))
if not index.is_absolute(): index = ROOT / index
index_hash = sha(index); assert index_hash == old['git_index_sha256']
inv, cov = load('source_inventory.json'), load('read_coverage.json')
stats = json.loads(subprocess.check_output(['python', str(HERE / 'read_coverage.py'), 'status'], cwd=ROOT))
assert stats['complete_files'] == 2580 and stats['read_chars'] == 30534939
assert inv['commit'] == head and load('pending_read.json') == []
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
fresh = [p for p in inv['files'] if p.startswith('crates/servitor/')]
assert len(fresh) == 12 and sum(inv['files'][p]['chars'] for p in fresh) == 207425
assert not set(fresh).intersection(guards)
assert all(cov[p]['through'] == inv['files'][p]['chars'] and cov[p]['sha256'] == sha(ROOT / p) == inv['files'][p]['sha256'] for p in fresh)
first = load('servitor_owner_probe_receipt.json'); probe = load('servitor_owner_probe_v2_receipt.json')
assert first['original_tests'] == 98 and first['audit_cases'] == 0
assert [first['results'][k]['exit'] for k in ['owner_compile', 'unit_compile', 'unit_output', 'consumer_compile']] == [0, 0, 0, 1]
assert probe['audit_cases'] == 4 and probe['original_tests_this_attempt'] == 0
assert all(v['exit'] == 0 for v in probe['results'].values()) and len(probe['results']) == 3
assert probe['prior_original_tests']['sha256'] == sha(HERE / 'servitor_owner_probe_receipt.json')
for name, receipt in [('servitor_owner_probe', first), ('servitor_owner_probe_v2', probe)]:
    assert receipt['runner_sha256'] == sha(HERE / ('run_' + name + '.py'))
    assert receipt['probe_sha256'] == sha(HERE / (name + '.rs'))
    assert receipt['generated_output_removed'] and all(not Path(v['path']).exists() for v in receipt['removed_outputs'])
    assert all(sha(ROOT / p) == h for p, h in receipt['source_sha256'].items())
    assert all(sha(Path(v['path'])) == v['sha256'] for v in receipt['libraries'].values())

trace_specs = [
    ('crates/servitor/src/delegation.rs', [(7, 37), (63, 70), (267, 331)]),
    ('crates/servitor/src/lib.rs', [(101, 145)]),
    ('crates/servitor/src/watch.rs', [(7, 29), (149, 159), (195, 227), (247, 304)]),
    ('crates/servitor/src/grant.rs', [(52, 77), (86, 107), (153, 163)]),
    ('crates/servitor/src/cascade.rs', [(177, 224)]),
    ('crates/servitor/src/deadband.rs', [(170, 205), (263, 301)]),
    ('crates/servitor/src/gate.rs', [(165, 190), (251, 273), (306, 350)]),
    ('crates/servitor/src/tick.rs', [(107, 122), (171, 211)]),
    ('crates/moot/gemot/src/moot/delegation.rs', [(271, 289)]),
    ('crates/moot/gemot/src/moot/typed_authorization.rs', [(7, 26)])]
external = Path('C:/Users/mark_/.cargo/git/checkouts/dramatis-c1632f2f3bdc2597/cf562f4/crates/insigne/src/delegation.rs')
def excerpt(path, ranges):
    lines = path.read_text(encoding='utf-8-sig').splitlines()
    return {'path': str(path), 'sha256': sha(path), 'excerpts': [
        {'first_line': a, 'last_line': b, 'text': '\n'.join(lines[a - 1:b])} for a, b in ranges]}
save('servitor_source_traces.json', {'basis_commit': head,
    'production': [excerpt(ROOT / p, ranges) for p, ranges in trace_specs],
    'external_contract': excerpt(external, [(49, 82), (145, 181)]),
    'qualification': 'Exact source excerpts and hashes. Cached Insigne explains the application-owned domain/resource check and inclusive expiry contract; this external source does not count toward Mere coverage. Public executions are separately recorded.'})
notes, authority = load('review_notes.json'), load('authority_map.json')
assert notes['findings'][-1]['id'] == 'Q371' and authority['entries'][-1]['id'] == 'A1073'
b = 'crates/servitor/src/'
rows = [
('Q372', 'Servitor coverage discards the signed application domain and resource',
 b + 'delegation.rs:67,317,321; ' + b + 'gate.rs:308',
 'A genuine root identity signs a structurally valid certificate for unrelated.application/world-Z, with the regular encoded trail path. Shared SignedDelegationCertificate::check succeeds. DelegationTable adopts it, covers the subject for trail/today and Gate commits an in-scope node. A normal mere.denizen/world-A certificate works and an outside-path control is refused. Domain/resource are signed, but covers_inner asks only subject/path/action/time and verified chain; the table has no expected resource context.',
 'The execution proves that foreign-domain coverage can reach a real Gate/GraphLog commit. It does not prove a live host imported a foreign certificate or leaked another on-disk graph. Resource absence is an unrepresentable audience check in this provider seam, not a demonstrated wrong product resource selected by a host. Gemot explicitly checks is_moot_scope, a real separate mitigation.',
 'Bind coverage to an explicit expected application/resource context and validate the exact signed scope at adoption or read. Preserve common cryptographic/attenuation reuse while fixing the application-owned audience check. Wider provider/wire/host integration may exceed the audit time budget and is documented.'),
('Q373', 'Truncated journal author labels merge distinct subjects and suppress their cross-wakes',
 b + 'lib.rs:141; ' + b + 'watch.rs:123; ' + b + 'gate.rs:319',
 'Two different public Subject values sharing their first four bytes produce the same denizen:first8hex author. Both are explicitly granted scope authority and make separate successful Gate commits, whose journal author fields are equal. A watch for the first subject then suppresses the second subject label as self-authored. A different-label control wakes normally.',
 'Synthetic distinct32-byte Subject inputs accepted by the public API; no collision search over generated cryptographic keys, identity authentication bypass or live peer trial. Attribution and watch matching are executed. This is a32-bit label namespace collapsing a256-bit identity, not an assertion that signatures verify as the other subject.',
 'Keep full subject identity in persisted attribution and self-wake matching, with shortened labels confined to display. Specify legacy-author replay/migration and journal identity conventions before changing durable strings; this is documented rather than extending source audit into a cross-tier migration.'),
('Q374', 'A registered watch continues revealing changes after its read grant expires',
 b + 'watch.rs:203,255,287; ' + b + 'cascade.rs:197',
 'At host time9 a read grant expiring at10 registers a private-scope watch. The same authority is advanced to10 and refuses Read, but would_wake still names the subject and wake returns the private entry sequence. Explicit removal stops later events. Only registration consults a provider; neither wake path nor run_cascade accepts one. Source promises containment and says narrowed grants are checked at wake time.',
 'Actual in-memory library decisions and sequence disclosure to its caller, not a live product sending a body/data to a revoked peer. A host can remove or filter watches and separately gate runs/actions, but those are additional caller-owned decisions absent from the claimed core enforcement.',
 'Revalidate current Read coverage at the common wake admission seam, keeping stored subscriptions distinguishable from currently authorized delivery. Cover expiry and narrowing as well as explicit uninstall, and preserve cursor/deferred-work semantics under refusal.'),
('Q375', 'Overlapping watches yield duplicate wakes for one subject while the peek deduplicates it',
 b + 'watch.rs:255,274,287; ' + b + 'cascade.rs:197,207',
 'One subject registers trail and trail/today. A single external trail/today/a event makes would_wake return one subject, but wake returns two Wake records for that subject carrying the same sequence1. Replaying the same event correctly returns nothing after cursor advancement. The mutation path sorts but never groups wakes by subject, unlike its read-only naming twin.',
 'Actual public matcher output with permitted overlapping registrations. No production body was run twice; the host-supplied cascade runner may deduplicate independently. This establishes duplicate scheduling inputs and peek/drain disagreement, not an observed doubled external side effect.',
 'Aggregate matching positions by subject at wake production, preserving sorted unique sequences and each subscription cursor. This is a bounded implementation candidate, documented under the read-only objective; do not require every consumer to invent another dedupe policy.')]
findings = []
for ident, title, source, evidence, scope, repair in rows:
    findings.append({'id': ident, 'title': title, 'source': source,
        'status': 'reproduced public APIs of freshly compiled complete unchanged Servitor',
        'evidence': evidence, 'scope': scope + ' ' + probe['qualification'], 'verification': NAME,
        'diagnostics': ['servitor_owner_probe_v2_receipt.json'],
        'repair_triage': {'disposition': 'documented under active read-only full-source audit', 'reason': repair}})
notes['findings'].extend(findings)
authority_rows = [
('Typed capability algebra is actually shared',
 'Cap owns equality for powers and segment-prefix for scopes/facets; GrantTable delegates coverage to it.',
 [b + 'cap.rs', b + 'grant.rs', b + 'delegation.rs'],
 'Scope validation, cross-kind refusal and percent/slash escaping of power/facet segments are real implementation reuse. Powers are equality-closed names, not a fixed compile-time enum of all application actions. One trailing separator/root spelling is intentional.'),
('Shared certificate grammar still has application-owned chain/read policy',
 'Servitor calls Insigne signature checking and attenuation, then owns table storage, root checks, recursive traversal, clock and local revocation IDs.',
 [b + 'delegation.rs', 'crates/moot/gemot/src/moot/delegation.rs'],
 'The cryptographic grammar reuse is real. Owns none of the machinery and two tiers differ only in root overstate it: Gemot maintains another chain traversal and validates Moot scope; Servitor omits that audience layer (Q372). Local revoke(id) is a host mutation, not validation of an independently received signed revocation.'),
('Signed audience and path coverage are separate checks',
 'Certificate covers(path,action,time) does not itself compare domain/resource against a caller audience.',
 [b + 'delegation.rs', 'crates/moot/gemot/src/moot/delegation.rs', 'crates/moot/gemot/src/moot/typed_authorization.rs'],
 'Q372 uses genuine signature verification and real Gate commit. Gemot covers explicitly calls is_moot_scope and is not relabeled as sharing the bug. Mere.denizen encoding by honest writers is not an enforcement check on adopted facts.'),
('Unsigned and signed provider expiry boundaries differ',
 'Grant expires at now>=bound; signed certificate covers through now<=bound.',
 [b + 'grant.rs', b + 'delegation.rs'],
 'Exact source contracts show a one-instant difference across the shared AuthorityProvider seam. Default host-set clock0 and explicit Grant::covers ignoring expiry are documented caller contracts. No new boundary diagnostic beyond the unsigned expiry/watch case ran, and no expiry policy is chosen here.'),
('Subject identity is full-width at authority but shortened at attribution',
 'Gate checks full Subject and then commits through Subject::to_author using only the first8hex.',
 [b + 'lib.rs', b + 'gate.rs', b + 'watch.rs'],
 'Q373 executes equal labels for distinct subjects and consequent false self suppression. Journal labels are not authentication, but the distinction does not recover lost attribution or matcher identity. Main GraphJournal full-hex convention is a separate existing path, not a globally unified author encoding.'),
('The graph gate centralizes its ordinary validation pipeline',
 'Both petition entry points call validate_petition for reserved projection IDs, node scope and facet coverage.',
 [b + 'gate.rs'],
 'This is actual shared implementation. Exact-ID assertion edits look up retained endpoints; missing IDs fall through to atomic Chartulary errors. Facet writes require a separate typed namespace. Assertion source attribution remains independently supplied metadata while journal commit author is the acting subject; the source test says so explicitly.'),
('Grant projections are materialized records, not certificate authority',
 'Gate writes explicit cap/mode/subject/expiry tags and read_projection reconstructs Grant.',
 [b + 'gate.rs'],
 'Projection records are schema/media-type stamped when written, but the reader checks tags/fields rather than media-type, ID prefix or authorship. They are protected from ordinary Gate petitions; arbitrary graph/source mutation is outside that protection. Projection ID is cap-only, so multi-grant persistence assumptions need consumer tracing before claiming lossless collection reconstruction.'),
('Wake containment is registration-only in the current matcher',
 'register checks Read; parse/adopt preserve stored watches; wake/would_wake consult no authority.',
 [b + 'watch.rs', b + 'cascade.rs'],
 'Q374 contradicts the never-side-channel and wake-time revalidation prose. Retaining a subscription record is distinct from authorizing delivery. Host run/action gates and explicit remove_subject can mitigate different paths but do not make this core matcher revalidate current Read.'),
('Wake matching has one source but two aggregation policies',
 'Both calls use Watch::matches; would_wake deduplicates subjects while wake emits per subscription.',
 [b + 'watch.rs', b + 'cascade.rs'],
 'Q375 executes the resulting duplicate scheduling inputs. Input events can also repeat sequence positions in one slice; matching output sorts without uniquing. Cursor order assumes complete journal drains; source0 entries and late lower-sequence arrivals need the journal adapter contract, not an automatic data-loss finding.'),
('Cascade budget bounds rounds and relies on host-retained journal work',
 'run_cascade advances current wakes then peeks beyond budget without advancing the deferred cursors.',
 [b + 'cascade.rs'],
 'Naming of exhausted subjects and cursor behavior are real. The returned Cascade stores rounds/outcome, not produced pending entries; a host must actually journal or retain the closure outputs for the next drain, as the original deferral test manually resupplies them. One round limit is not a time/CPU/per-body allocation bound.'),
('Deadband admission records only successful specialized gate commits',
 'petition_behavior checks/commits/records sequentially; generic petition intentionally has no deadband input.',
 [b + 'deadband.rs', b + 'gate.rs'],
 'The original failed-revision regression passes. i128 distance safely covers the full signed scalar range, and clock backsteps refuse via saturating elapsed time. Declaration changes reset baseline; undeclared subjects are deliberately untracked. Hosts must route declared behavior actuation through this entry point; a copyable public admission is not a one-use cross-thread authority token.'),
('Clock scheduling is a separate policy with deliberate no-catchup semantics',
 'TimeWatchTable fires sorted unique subjects and resets due schedules to supplied now.',
 [b + 'tick.rs'],
 'Clock triggers deliberately require no read capability. Install review, lifecycle removal and body/action gating remain caller-owned. Period choices are a closed Minute/Hour/Day set. TimeWatch::parse ignores trailing fields unlike strict Deadband parsing, and schedules can duplicate though due subjects deduplicate; source followups, not new executed product bugs.'),
('Resident tickets and run reduction correctly disclaim dispatch authority',
 'admit/revalidate consume current binding and provider facts; RunReducer only reduces already-recorded intents/results.',
 [b + 'resident.rs', b + 'run.rs'],
 'RunTicket is public mutable snapshot data, not a capability; revalidation checks original plus new requirements. Reducer atomically clones on rejected events, checks correlation/time/counter arithmetic and holds uncertain consequential work for reconciliation. It neither persists an intent nor runs/retries an effect; hosts own those obligations and declared effect/usage truth.'),
('Existing green source tests miss the public counterexamples',
 'Complete unchanged owner unit suite passes98; corrected separate consumer reproduces4cases.',
 ['crates/servitor/Cargo.toml', b + 'lib.rs'],
 'The first audit consumer had a missing Issue trait import after the owner and all98unit tests already passed. Preserve that compile failure and its passing tests. V2 fixes only the audit import and reruns whole-owner/consumer compilation and4cases; original tests are not rerun. Three exact retained suppliers, not a fresh Cargo graph or live host receipt.')]
entries = [{'id': f'A{1074 + i}', 'domain': d, 'claim': c, 'implementations': p, 'assessment': a,
            'state': 'whole Servitor source complete; original unit tests and public counterexamples scoped separately', 'verification': NAME}
           for i, (d, c, p, a) in enumerate(authority_rows)]
authority['entries'].extend(entries)
area = {'prefix': 'crates/servitor/', 'files': 12, 'chars': 207425, 'basis_commit': head, 'verification': NAME,
        'qualification': probe['qualification']}
notes['completed_source_areas'].append(area)
notes['review_followups'].extend([
    {'verification': NAME, 'area': 'Signed audience and persisted actor identity',
     'detail': 'Q372 audience context and Q373 full identity migration have broad integration effects; document and preserve compatibility decisions rather than prolong the audit with a cross-tier provider/journal rewrite.'},
    {'verification': NAME, 'area': 'Current wake authorization and aggregation',
     'detail': 'Q374 requires a current Read decision before delivery; Q375 is bounded shared aggregation. Keep source subscription retention, refused delivery, cursor progression and deferred journal work explicit.'},
    {'verification': NAME, 'area': 'Source policy differences and host contracts',
     'detail': 'Signed/unsigned expiry inclusivity differs; projection readers do not verify provenance; clock wire parser tolerates trailing fields; specialized deadband and pure run reducer depend on host routing/journaling. No automatic host-execution claim follows from these portable helpers.'}])
next_read = {'path': 'crates/shell/chrome/Cargo.toml', 'start': 0, 'reason': 'Continue full unread shell chrome/navigation/authority source and remaining repository areas.'}
notes['current_area'] = 'Servitor source complete;98original unit tests pass and4public authority/watch cases reproduced'
notes['next_area'] = next_read['reason']
notes['checkpoint'] = {'classification': 'progress', 'classification_of_previous_turn': 'progress', 'goal_status': 'active; full source incomplete',
    'coverage': stats, 'finding_count': len(notes['findings']), 'authority_entries': len(authority['entries']),
    'new_receipts': [NAME, 'servitor_owner_probe_receipt.json', 'servitor_owner_probe_v2_receipt.json', 'servitor_source_traces.json'],
    'audit_authored_production_edits_this_turn': False, 'next_read': next_read,
    'pending_verification': 'No owned process live. Live host ingress/delivery, generated key collision search, durable author migration, product scheduling and fresh Cargo dependency graph unexecuted. Earlier Q352 and metadata timeout remain qualified.',
    'diagnostic_outcomes': 'Complete owner compiled; all98original unit tests pass. First new consumer compile fails from missing audit Issue import; corrected V2 consumer compiles and reproduces4public cases. All generated owner libraries/unit and consumer executables/PDB/link outputs removed.'}
diff = subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True, text=True); assert diff.returncode == 0
with (HERE / 'servitor_diff_check.txt').open('x', encoding='utf-8') as f: f.write(diff.stdout + diff.stderr)
addition = f'''

### Servitor source complete: signed audiences, author identity and wake authority, 2026-10-17

Read all12Servitor files /207,425characters at unchanged HEAD {head}. Coverage is{stats['complete_files']:,}/{stats['total_files']:,}files /{stats['read_chars']:,}/{stats['total_chars']:,}characters, with no stale/pending credit. Full audit remains active and incomplete.

Fresh direct compilation of the complete unchanged owner over3exact retained Cargo-fingerprint-matched suppliers passes all98original unit tests. The first separate public consumer then fails to compile because the audit omitted identity::delegation::Issue; its receipt/source/output are preserved. V2 adds that audit-only import, freshly recompiles owner/consumer and reproduces4public cases without needlessly rerunning the unchanged original suite. This is not a production compilation failure, fresh Cargo dependency graph or live host acceptance.

Q372: a genuine root-signed certificate for unrelated.application/world-Z passes the shared signature check, answers a typed trail request in DelegationTable and reaches an actual Gate/GraphLog commit. Normal-domain and outside-path controls work. Servitor does not compare the signed domain/resource against an expected audience; its provider has no resource context. No live host foreign-certificate import or wrong on-disk graph is claimed. Gemot explicitly checks is_moot_scope and is a distinct mitigation. Shared signature grammar does not supply application-owned audience checks automatically.

Q373: two distinct raw Subject values with the same first4bytes receive the same denizen:first8hex journal author. Authorized Gate commits from both have equal author fields; a watch for one suppresses the other's label as self. A different-label control wakes. This uses synthetic public Subject values, not generated colliding cryptographic keys or an identity-authentication bypass. Full-width durable attribution/self identity and legacy author migration are a broad seam to document.

Q374: a private Read watch registered at time9 still wakes and reveals matched sequence1 after the same current provider advances to expiry10 and refuses Read. Explicit removal stops it. register checks authority, but wake/would_wake/run_cascade do not accept a provider, contradicting the core containment/wake-time prose. No live product delivers a body to a revoked peer in this case. Q375: one subject's overlapping trail and trail/today subscriptions make peek name it once but drain return2Wake records for one entry. Cursor replay control is quiet. This proves duplicated scheduling inputs; no product body/external effect ran twice.

A1074-A1087 record real typed capability/signature/attenuation reuse versus separately owned chain/root/read/revocation policy; missing signed audience versus Gemot scope checks; inclusive signed versus exclusive unsigned expiry; full authority subject versus shortened authors; actual shared petition validation and exact-ID endpoint checks; projections versus proven authority; registration versus current wake permission; matcher reuse versus divergent aggregation; deferred cursors versus host-retained pending entries; deadband versus generic petitions; clock scheduling versus journal watches; public resident snapshots versus dispatch permission; and pure run reduction versus durable/executed effects. Additional parser/projection/host-routing observations remain source-qualified, with no new expiry policy, persisted identity migration or lifecycle economics chosen.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}), [original unit run and preserved failed consumer](../../../support/receipts/authority-success-audit/servitor_owner_probe_receipt.json), [corrected public cases](../../../support/receipts/authority-success-audit/servitor_owner_probe_v2_receipt.json), and [source traces](../../../support/receipts/authority-success-audit/servitor_source_traces.json). No production edits. Protected source/runtime/auxiliary hashes, prior evidence, shared WIP and Git index remain intact; every previous report byte remains an exact prefix. All generated owner rlib/unit/consumer executable/PDB/link outputs removed; no process remains live. Diff check passes. No retained isolated target, Cargo home or worktree; retained evidence remains below100MB. Next: unread shell/chrome source.
'''
doc.write_bytes(prefix + addition.encode('utf-8'))
for n, value in [('review_notes.json', notes), ('authority_map.json', authority)]:
    (HERE / n).write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')
evidence = ['record_servitor_review.py', 'servitor_diff_check.txt', 'servitor_source_traces.json',
    'servitor_owner_probe.rs', 'run_servitor_owner_probe.py', 'servitor_owner_probe_receipt.json',
    'servitor_owner_probe_owner_compile.log', 'servitor_owner_probe_unit_compile.log', 'servitor_owner_probe_unit_output.log', 'servitor_owner_probe_consumer_compile.log',
    'servitor_owner_probe_v2.rs', 'run_servitor_owner_probe_v2.py', 'servitor_owner_probe_v2_receipt.json',
    'servitor_owner_probe_v2_owner_compile.log', 'servitor_owner_probe_v2_consumer_compile.log', 'servitor_owner_probe_v2_output.log']
budget = sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file()); assert budget < 100_000_000
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': head,
    'kind': 'Servitor source complete;98original unit tests and4public cases; repository incomplete',
    'prior_receipt': {'path': 'script_hosts_review_receipt.json', 'sha256': sha(HERE / 'script_hosts_review_receipt.json')},
    'coverage': stats, 'completed_source_areas': [area], 'whole_current_files': fresh,
    'source_sha256': {p: sha(ROOT / p) for p in fresh}, 'preserved_production_and_test_hashes': guards,
    'auxiliary_read_sha256': aux, 'runtime_input_sha256': runtime, 'shared_working_file_sha256': shared,
    'external_contract_sha256': {str(external): sha(external)},
    'source_trace_sha256': {p: sha(ROOT / p) for p, _ in trace_specs},
    'new_findings': [f['id'] for f in findings], 'authority_entries': [e['id'] for e in entries],
    'audit_authored_production_edits_this_pass': False,
    'diagnostics': {'servitor_owner_probe_receipt.json': {'owner_compile_exit': 0, 'unit_compile_exit': 0, 'unit_run_exit': 0, 'original_tests': 98, 'consumer_compile_exit': 1, 'audit_cases': 0},
                    'servitor_owner_probe_v2_receipt.json': {'owner_compile_exit': 0, 'consumer_compile_exit': 0, 'run_exit': 0, 'original_tests_this_attempt': 0, 'audit_cases': 4}},
    'pending_verification': notes['checkpoint']['pending_verification'], 'next_read': next_read,
    'document': {'path': old['document']['path'], 'sha256': sha(doc), 'preserved_prefix_bytes': len(prefix), 'preserved_prefix_sha256': hashlib.sha256(prefix).hexdigest()},
    'evidence_sha256': {p: sha(HERE / p) for p in evidence}, 'git_index_sha256': index_hash,
    'receipt_budget': {'cap_bytes': 100_000_000, 'retained_bytes_before_receipt': budget}, 'retained_isolation': [],
    'limits': probe['qualification']}
save(NAME, receipt)
assert sha(index) == index_hash and doc.read_bytes().startswith(prefix)
assert all(sha(ROOT / p) == h for p, h in {**guards, **receipt['source_sha256'], **aux, **runtime}.items())
assert all(sha(HERE / p) == h for p, h in receipt['evidence_sha256'].items())
assert all(sha(ROOT / p) == h for p, h in shared.items() if p != old['document']['path'])
assert subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True).returncode == 0
print(json.dumps({'receipt': NAME, 'coverage': stats, 'new_findings': receipt['new_findings'], 'authority_entries': receipt['authority_entries'],
                  'original_tests': 98, 'audit_cases': 4, 'protected_sources': len({**guards, **receipt['source_sha256']}), 'retained_bytes': budget}, indent=2))
