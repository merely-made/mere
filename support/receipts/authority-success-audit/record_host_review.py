"""Checkpoint the shared-host source review without touching production code."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import subprocess
import re

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda name: json.loads((HERE / name).read_text(encoding='utf-8'))
def save(name, obj):
    (HERE / name).write_text(json.dumps(obj, indent=2)+'\n', encoding='utf-8')

notes, authority = load('review_notes.json'), load('authority_map.json')
old = load('testkit_review_receipt.json')
preserved = old['preserved_production_and_test_hashes']
assert all(sha(ROOT / p) == digest for p, digest in preserved.items()), 'Concurrent source changed: do not credit stale receipts'
assert load('pending_read.json') == []
inventory, coverage = load('source_inventory.json')['files'], load('read_coverage.json')
host_paths = [p for p in inventory if p.startswith('crates/graphshell/graphshell-host/')]
assert len(host_paths) == 24
assert all(coverage[p]['sha256'] == sha(ROOT/p) == inventory[p]['sha256'] and coverage[p]['through'] == inventory[p]['chars'] for p in host_paths)
probe = load('host_admission_probe_receipt.json')
assert probe['test_exit'] == 0 and probe['generated_native_output_removed']
assert all(sha(ROOT/p) == digest for p, digest in probe['source_sha256'].items())
default_log = (HERE/'graphshell_host_default_test_build_output.txt').read_text(encoding='utf-8-sig', errors='replace')
assert 'due to 36 previous errors' in default_log and 'unresolved import `crate::carrier`' in default_log

def finding(id, title, source, related, evidence, scope, repair, status='reproduced; documented, not repaired', diagnostic='host_admission_probe_receipt.json'):
    return dict(id=id, status=status, title=title, source=source, related_sources=related,
                evidence=evidence, scope=scope, verification='host_review_receipt.json', diagnostic=diagnostic,
                repair_triage={'disposition': 'documented; production edits excluded by active audit objective', 'reason': repair})
findings = [
finding('Q056', 'Message admission returns an Accept frame alongside a local service-vocabulary refusal',
 'crates/graphshell/graphshell-host/src/admission.rs:176',
 ['crates/graphshell/graphshell-host/src/webrtc_session.rs:245', 'crates/graphshell/graphshell-host/src/webrtc_join.rs:311', 'crates/graphshell/graphshell-host/src/carrier.rs:292'],
 'admit_session preserves the Notochord reply and then may replace an accepted principal with ActionNotOffered. With a signed grant and owner policy allowing administer, current public sans-I/O admission returns local ActionNotOffered and a decoded Accept. Supported-connect and policy-refused controls agree. host_join sends the reply before testing the local Result; the peer maps Accept to successful PeerJoin. The existing unsupported-action test discards the reply.',
 'Actual native execution of public sans-I/O admission with supplied authenticated memory-carrier facts; complete host/peer caller source read. No full WebRTC failure reproduction or authorization bypass: the host still refuses to serve. Native carrier.rs explicitly documents policy admission followed by service refusal/stream close; do not erase that intentional two-stage contract.',
 'Make the message helper final outcome and peer-visible reply agree, or expose the policy/service stages explicitly on both sides. Test an owner-allowed but service-unsupported action with both decoded reply and local outcome. Coordinate with the intentionally two-stage native path.'),
finding('Q057', 'Reconnect uniqueness claim exceeds the freshness enforced by admission',
 'crates/graphshell/graphshell-host/src/lifecycle.rs:23',
 ['crates/graphshell/graphshell-host/src/lifecycle.rs:54', 'crates/graphshell/graphshell-host/src/lifecycle.rs:472', 'crates/graphshell/graphshell-host/src/webrtc_join.rs'],
 'Module prose says reconnecting produces a different id and makes old cached scenes unreachable by construction. The projection_session function correctly qualifies that caller nonce freshness is not enforced. Two current public admissions of one signed hello produce identical session ids; a fresh-nonce control differs. The reconnect unit test instead supplies two already-different principal ids.',
 'Documentation/test overclaim on the public helper contract. Current WebRTC peers generate fresh join/hello nonces; no ordinary WebRTC reconnect failure, foreign-subject collision, or authority bypass is established. The valid public input demonstrates why the unqualified universal claim is false.',
 'Align the broad claim and its test with the already accurate function-level contract. If stronger uniqueness is desired, define which carrier/admission owner supplies freshness before introducing replay state.',
 status='public API counterexample reproduced; claim needs qualification'),
finding('Q058', 'N4 open-Graphshell receipt and its oracle both use the Murm domain',
 'crates/graphshell/graphshell-host/src/policy_projection.rs:188',
 ['crates/graphshell/graphshell-host/src/policy_projection.rs:143', 'crates/graphshell/graphshell-host/src/policy_projection.rs:312', 'crates/graphshell/graphshell-host/src/admission.rs'],
 'The scenario uses one rule helper for Murm and Graphshell, hardcoding mere.network. Graphshell uses mere.graphshell. Its unit claims helper repeats mere.network, so the test proves a synthetic request agrees with the same wrong template. A byte-exact extracted rule marked Public refuses a real signed Graphshell hello in public admission; the correct-domain control accepts. Stage prose still says Graphshell remains open.',
 'Receipt/sample-policy defect, not a demonstrated general outage in stored user policies. Native diagnostic executes the exact rule helper plus public admission, not the complete public N4 persistence or headed scenario.',
 'Give each service its actual domain and test the generated policy against the production Graphshell hello builder rather than repeating a local string in its oracle.'),
finding('Q059', 'Default host unit-test compilation depends on native-only definitions',
 'crates/graphshell/graphshell-host/src/webrtc_door.rs:658',
 ['crates/graphshell/graphshell-host/src/webrtc_door.rs:681', 'crates/graphshell/graphshell-host/src/webrtc_door.rs:684', 'crates/graphshell/graphshell-host/Cargo.toml', 'crates/graphshell/graphshell-host/src/lib.rs:51'],
 'cargo test -p graphshell-host --lib --no-run --locked --offline at default features exits 101 with 36 compilation errors. Tests are cfg(test), but import crate::carrier and native-only aliases/functions; transport::initiator_link_binding also requires the notochord feature enabled by host/native. Default is webrtc-join, without native. Integration tests mostly declare required-features, but that does not gate this unit module.',
 'Current Windows default-feature unit build. It reached graphshell-host; this is not an unrelated dependency failure. No unit tests ran. Admission-only production library builds successfully. This does not claim the native-feature or wasm production libraries fail.',
 'Separate browser-clean tests from native fixtures with matching feature gates/imports and qualify default/native/wasm feature combinations.',
 diagnostic='graphshell_host_default_test_build_output.txt')
]
entries = [
 dict(id='A052', domain='Policy decision, service vocabulary and message admission reply', claim='Stream and message admission end in the same serves_action gate.',
      implementations=['Shared serves_action implementation is real reuse', 'Sans-I/O admission retains the policy reply before its service check', 'WebRTC host writes that reply then observes local failure; peer concludes admission on Accept', 'Native stream path explicitly documents policy admission followed by service refusal'],
      assessment='Q056 identifies divergent coordination around a truly shared predicate. One predicate does not make wire outcome and host conclusion one authority.', state='sans-I/O mismatch reproduced; full caller source traced', verification='host_review_receipt.json'),
 dict(id='A053', domain='Projection namespace and reconnect freshness', claim='Reconnect changes the transcript-derived projection session by construction.',
      implementations=['Notochord transcript includes caller hello nonce', 'projection_session only encodes the resulting digest', 'Function docs explicitly assign distinctness to caller', 'WebRTC joins currently supply fresh random nonces; reconnect unit test supplies different ids'],
      assessment='Q057 qualifies a universal namespace claim. Freshness is caller/carrier work, not a guarantee added by the projection_session wrapper. Same-subject repeats alone are not an authority breach.', state='counterexample and fresh-nonce control reproduced', verification='host_review_receipt.json'),
 dict(id='A054', domain='Service vocabulary versus policy receipt oracle', claim='N4 demonstrates independent owner control while Graphshell remains open.',
      implementations=['Production Graphshell constants and hello builder use mere.graphshell', 'N4 helper hardcodes mere.network for two services', 'N4 claims helper repeats that same domain', 'Rendered stage describes Graphshell as open'],
      assessment='Q058 is self-referential receipt validation around duplicated service identity. Policy persistence may be real while the claimed usable service remains refused.', state='exact helper plus real hello refusal reproduced; full persistence scenario not run', verification='host_review_receipt.json'),
 dict(id='A055', domain='Feature ownership of shared host tests', claim='Default host remains browser-clean; native adds accept/loop/catalog implementations.',
      implementations=['Production module and function native gates', 'Cargo default enables webrtc-join only', 'Unconditional webrtc_door unit module imports native APIs and feature-gated transport adapter', 'Most integration targets declare required-features'],
      assessment='Q059 shows test fixture ownership does not follow production feature ownership. Default test success cannot be inferred from a native workspace gate.', state='standalone default unit compilation failure reproduced', verification='host_review_receipt.json')
]
notice_receipt = HERE / 'host_notice_authority_receipt.json'
if notice_receipt.exists():
    notice = load(notice_receipt.name)
    assert notice['test_exit'] == 0 and notice['generated_native_output_removed']
    assert all(sha(ROOT/p) == digest for p,digest in notice['source_sha256'].items())
    findings.append(finding('Q060', 'Idle notice pushes continue after grant expiry or revocation until a request arrives',
      'crates/graphshell/graphshell-host/src/session_loop.rs:175',
      ['crates/graphshell/graphshell-host/src/session_loop.rs:229', 'crates/graphshell/graphshell-host/src/session_notices.rs:19', 'crates/graphshell/graphshell-host/src/lifecycle.rs:280'],
      'The shared loop polls and writes notices before its only authority check. Idle read timeouts repeat this path without checking authority. A public native notifying-loop diagnostic starts live, receives a first notice, then expires or revokes the real signed grant. After an independent lapse assertion, a newly queued notice still reaches the idle peer. The next valid Close request is refused and terminates with the correct Lapsed result. A live control receives notices and closes normally.',
      'Current public native session loop over Tokio duplex with a constructed admitted session and real signed grant/revocation. No admission handshake, installed resident, remote link or DTLS test. Demonstrates session/epoch/revision notice disclosure and continued endpoint polling, not arbitrary snapshot/resource/intent authorization bypass; request gating still works.',
      'Apply retained-authority checks to asynchronous output and idle polling as well as inbound requests; define how a quiet peer is told authority lapsed without inventing a request id. Qualify mid-session revocation and expiry across both notice and request lanes.',
      diagnostic='host_notice_authority_receipt.json'))
    entries.append(dict(id='A056', domain='Retained session authority across requests and pushed notices', claim='One shared loop keeps authority checks and lapse handling in one place.',
      implementations=['Both entry points truly share session_loop', 'Valid inbound requests recheck fresh ledger and time before dispatch', 'Notice poll/drain/write precedes those checks and repeats on idle timeouts', 'Lifecycle intent adjudication deliberately withholds current revision from a lapsed peer'],
      assessment='Q060 shows the shared implementation owns only a partial authority boundary: request gates do not govern autonomous pushes.', state='mid-session expiry/revocation notice leakage reproduced on public native loop', verification='host_review_receipt.json'))

client_receipt = HERE / 'client_pending_receipt.json'
if client_receipt.exists():
    client_probe = load(client_receipt.name)
    assert client_probe['test_exit'] == 0 and client_probe['generated_native_output_removed']
    assert all(sha(ROOT/p) == digest for p,digest in client_probe['source_sha256'].items())
    findings.append(finding('Q061', 'Refusing a second client request overwrites the first pending operation',
      'crates/graphshell/graphshell-client/src/driver.rs:126',
      ['crates/graphshell/graphshell-client/src/core.rs:597', 'crates/graphshell/graphshell-client/src/driver.rs:216', 'crates/graphshell/graphshell-client/src/driver.rs:394'],
      'discover calls core.rediscover before send checks inflight. SessionCore::begin unconditionally replaces Pending. Public diagnostic starts a Close, attempts a second discover and gets the expected still-waiting refusal. The valid Closed response with the original id is then rejected because the core now expects a descriptor. Without the second call, the same close completes. The existing second-request unit test only asserts that the second send errors; it never answers the original request.',
      'Current compiled public client API and encoded NDJSON response, no I/O, browser or product UI. This is local pending-operation corruption after an explicitly refused operation, not request-id spoofing or endpoint authorization bypass. Full core.rs and driver.rs source read; the rest of the client package remains ahead.',
      'Make starting the core operation and claiming the driver request slot one coordinated transition. Checking only discover early fixes this entry but leaves public core_mut plus begin callers able to overwrite Pending before send refusal. Preserve the original operation after a refused start and exercise its legitimate answer.',
      diagnostic='client_pending_receipt.json'))
    entries.append(dict(id='A057', domain='Client in-flight request and semantic pending-operation ownership', claim='One shared SessionCore supplies the protocol; the driver refuses a second request while one is in flight.',
      implementations=['SessionCore.pending records the semantic operation', 'SessionDriver.inflight separately owns wire request identity', 'Core mutation occurs before driver send admission', 'Second-send test checks refusal without completing the first operation'],
      assessment='Q061 shows the two state authorities can disagree after a successful refusal. The core is truly shared, but call sequencing does not make the combined transition atomic.', state='original response rejection after refused rediscovery reproduced', verification='host_review_receipt.json'))

def upsert(items, added, key):
    for entry in added:
        existing = next((i for i,x in enumerate(items) if x.get(key) == entry[key]), None)
        if existing is None: items.append(entry)
        else: items[existing] = entry
upsert(notes['findings'], findings, 'id')
upsert(authority['entries'], entries, 'id')
promoted = 'Service-vocabulary refusal can follow an already accepted policy handshake'
notes['candidates'] = [x for x in notes['candidates'] if x.get('title') != promoted]
candidates = [
 {'title': 'Concurrent WebRTC catalog joins snapshot the same finite session allowance', 'source': 'crates/graphshell/graphshell-host/src/projection_host.rs:477', 'note': 'admit_webrtc_catalog snapshots live_sessions under a short lock, releases it across join, then serve_admitted increments live count without rechecking the ceiling by documented contract. Concurrent offers can use the same active count. Requires finite configured ceiling and concurrent WebRTC joins. Source-confirmed reservation omission; actual concurrent carrier reproduction and current consumer adoption remain pending. Repair needs a reservation/rollback lifetime, not serialization across slow DTLS.'},
 {'title': 'WebRTC join evaluates a captured time and ledger snapshot after awaited exchanges', 'source': 'crates/graphshell/graphshell-host/src/webrtc_session.rs:164', 'note': 'host_join accepts one now_ms value through several awaited messages. Catalog adapter snapshots revocation ledger and samples time before join; no local wrapper timeout was found. Session request checks later use fresh time/ledger. Trace carrier/caller lifetime, idle timers and current consumers before claiming a reachable expired-grant service bypass.'},
 {'title': 'Custom invite action is broader than the peer join helper vocabulary', 'source': 'crates/graphshell/graphshell-host/src/webrtc_join.rs:hello_and_reply', 'note': 'InviteTerms and minted scope can name custom domain/path/action, while current peer helper opens fixed Graphshell root connect. Ordinary projection invitations match. Caller adoption of custom terms and intended extension contract remain pending before classifying unsupported custom invites as a product defect.'},
 {'title': 'Admitted host line assembly lacks an observed pre-allocation request-size ceiling', 'source': 'crates/graphshell/graphshell-host/src/line.rs', 'note': 'Tokio lines assembles complete text before JSON parse. No normal-request size budget is applied at that boundary in session_loop. Trace transport/caller limits and public configured budgets before asserting a remote memory-exhaustion path. Shares the network client line-limit candidate, not a second proven exploit.'},
 {'title': 'Some join format refusals close without sending the promised typed refusal', 'source': 'crates/graphshell/graphshell-host/src/webrtc_session.rs:host_join', 'note': 'Unexpected Open/Redeem shapes, wrong proof size and some signing/mint failures return via ? without ToPeer::Refused. Typed UnknownInvite and redeem errors do send it. Complete source read; exact caller-visible diagnostics and intentional fatal-channel contract need qualification.'},
 {'title': 'Presentation resolve retries after applying a response without retaining the requested hash', 'source': 'crates/graphshell/graphshell-client/src/core.rs:Pending::Resolve', 'note': 'Pending::Resolve retains session/instance but not the requested resource. on_response applies a ResourceResponse then resolves again, whereas Pending::MediaResource explicitly compares requested session/hash. Trace ClientState admission/cache rules before promoting wrong-scope acceptance or an unbounded refetch claim; lib.rs and remaining client source have not yet been fully read.'}
]
upsert(notes['candidates'], candidates, 'title')
qualifications = [
 {'area': 'Graphshell complete shared host source', 'note': 'All 24 inventoried host source/manifests/tests fully read. Catalog factory erasure, common dispatch, line codec and shared loop are real sharing. Public AdmittedEndpointContext explicitly performs no authentication and is an in-process handoff, not a bearer credential. No production source edits in this pass.'},
 {'area': 'Graphshell WebRTC evidence and invite lifetime', 'note': 'Loopback integration source exercises real ICE/DTLS/SCTP; remote_session_live explicitly substitutes direct dispatch for WebRTC. Neither was run this pass. HostedInvite intentionally retains the invite seed in memory for the offer, documented as not serialized; this alone is not a seed-leak finding. Current peer nonce generation qualifies Q057.'},
 {'area': 'Graphshell receipt UI', 'note': 'view.rs intentionally renders EditableText read-only and states why. Receipt action buttons carry semantic data and separate intent outcome receipts; source presence alone does not establish a live interactive headed action gate. No new product defect promoted merely because a static receipt is not an editor.'}
]
upsert(notes['source_qualifications'], qualifications, 'area')
for area in ['crates/graphshell/graphshell-host']:
    if area not in notes['completed_areas']: notes['completed_areas'].append(area)
stats = json.loads(subprocess.check_output(['python', str(HERE/'read_coverage.py'), 'status'], cwd=ROOT))
next_path = next(p for p in sorted(inventory) if p.startswith('crates/graphshell/graphshell-client/') and (coverage.get(p,{}).get('sha256') != inventory[p]['sha256'] or coverage.get(p,{}).get('through',0) < inventory[p]['chars']))
next_read = {'path': next_path, 'start': coverage.get(next_path,{}).get('through',0)}
notes['current_area'] = 'Shared Graphshell host fully read; client state, correlation, resume and cache next'
notes['next_area'] = 'crates/graphshell/graphshell-client, then remaining Mere source'
notes['checkpoint'] = {'classification': 'progress', 'goal_status': 'active; full-source objective incomplete', 'coverage': stats,
 'finding_count': len(notes['findings']), 'authority_entries': len(authority['entries']), 'new_receipts': ['host_review_receipt.json'],
 'diagnostic_outcomes': 'Five admission diagnostics passed; default unit compilation failed with 36 feature-gate errors.'+(' Three native notice diagnostics passed while confirming Q060.' if notice_receipt.exists() else ''),
 'next_read': next_read, 'workspace': 'Reused C:/t/cargo-targets/mere; standalone generated native probe outputs removed; no isolated Cargo home, target or worktree.'}
notes['audit_continuation'] = {'previous_turn': 'progress: Djinn/testkit complete and Q052-Q055 diagnosed', 'classification_of_previous_turn': 'progress',
 'production_edits_this_turn': False, 'completed_areas_this_pass': ['crates/graphshell/graphshell-host/'], 'coverage_checkpoint': stats, 'next_read': next_read,
 'user_instruction': 'Continue full source audit and preserve concurrent work; active goal excludes production edits.',
 'coverage_note': 'Full host source reading is complete. Diagnostic passes confirm defects/qualifications, not repairs, all-feature execution or remote acceptance.'}
assert len({x['id'] for x in notes['findings']}) == len(notes['findings'])
assert len({x['id'] for x in authority['entries']}) == len(authority['entries'])
save('review_notes.json', notes)
save('authority_map.json', authority)
evidence = ['host_admission_probe.rs', 'run_host_admission_probe.py', 'n4_policy_rule_extracted.rs', 'graphshell_host_admission_build_output.jsonl',
 'host_admission_probe_build_output.txt', 'host_admission_probe_output.txt', 'host_admission_probe_receipt.json', 'graphshell_host_default_test_build_output.txt']
if notice_receipt.exists():
    evidence += ['host_notice_authority_probe.rs', 'run_host_notice_authority_probe.py', 'graphshell_host_native_build_output.jsonl', 'host_notice_probe_build_output.txt', 'host_notice_probe_output.txt', notice_receipt.name]
if client_receipt.exists():
    evidence += ['client_pending_probe.rs', 'run_client_pending_probe.py', 'client_pending_build_output.txt', 'client_pending_output.txt', client_receipt.name]
    notes['checkpoint']['diagnostic_outcomes'] += ' Two client pending-operation diagnostics passed while confirming Q061.'
    save('review_notes.json', notes)
diff = subprocess.run(['git','diff','--check'], cwd=ROOT, capture_output=True, text=True)
(HERE/'host_review_diff_check.txt').write_text(diff.stdout+diff.stderr, encoding='utf-8')
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
 'kind': 'source audit and bounded native defect diagnostics; no repairs', 'previous_turn_classification': 'progress',
 'findings': [x['id'] for x in findings], 'authority_entries': [x['id'] for x in entries],
 'complete_areas': ['crates/graphshell/graphshell-host/'], 'host_files': len(host_paths), 'host_chars': sum(inventory[p]['chars'] for p in host_paths),
 'coverage': stats, 'next_read': next_read, 'production_edits_this_pass': False,
 'preserved_production_and_test_hashes': preserved, 'host_source_sha256': {p:sha(ROOT/p) for p in host_paths},
 'coverage_limits': ['No whole-source or workspace completion claim.', 'No headed browser, real remote peer, installed resident or Linux interoperability gate.', 'Source-read existing integration tests were not run in this pass.'],
 'default_unit_build': {'command': 'cargo test -p graphshell-host --lib --no-run --locked --offline -j 1 --target-dir C:/t/cargo-targets/mere --config build.build-dir=C:/t/cargo-targets/mere/build', 'exit': 101, 'errors': 36, 'tests_executed': False},
 'retained_isolation': [], 'generated_output': 'All standalone native probe executable/PDB sets removed after execution.',
 'diff_check': {'exit':diff.returncode,'log':'host_review_diff_check.txt'}, 'evidence_sha256': {p:sha(HERE/p) for p in evidence}}
doc = 'design_docs/mere_docs/technical_architecture/2026-10-15_transport_agnosticism_note.md'
receipt['governing_document_sha256'] = {doc:sha(ROOT/doc)}
receipt['diagnostic_test_counts'] = {'admission':5, 'notices':3 if notice_receipt.exists() else 0, 'client_pending':2 if client_receipt.exists() else 0}
receipt['native_host_build'] = {'exit':0, 'features':'native; no default features', 'log':'graphshell_host_native_build_output.jsonl'} if notice_receipt.exists() else None
receipt['partial_client_reading'] = {p:coverage[p] for p in coverage if p.startswith('crates/graphshell/graphshell-client/')}
save('host_review_receipt.json', receipt)
print(json.dumps({'coverage':stats, 'findings':len(notes['findings']), 'authorities':len(authority['entries']), 'next_read':next_read, 'preserved_hashes':len(preserved), 'diff_check':diff.returncode}))
