"""Record script source completion, one helper diagnostic and qualified host seams."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
NAME = 'script_hosts_review_receipt.json'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE / n).read_text(encoding='utf-8-sig'))
def save(n, value):
    with (HERE / n).open('x', encoding='utf-8') as f:
        json.dump(value, f, indent=2); f.write('\n')
def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()

assert not (HERE / NAME).exists()
old = load('mooting_illume_probes_review_receipt.json')
head = git('rev-parse', 'HEAD'); assert head == old['basis_commit']
guards = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}
assert len(guards) == 2062
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
assert stats['complete_files'] == 2568 and stats['read_chars'] == 30327514
assert inv['commit'] == head and load('pending_read.json') == []
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
areas = [('crates/script/app-host/', 7, 29051), ('crates/script/document-host/', 21, 97875),
         ('crates/script/rhai/', 8, 51486), ('crates/script/wit/', 1, 7229)]
all_script = [p for p in inv['files'] if p.startswith('crates/script/')]
fresh = [p for p in all_script if p not in guards]
assert len(all_script) == 37 and len(fresh) == 36
assert sum(inv['files'][p]['chars'] for p in fresh) == 176097
assert set(all_script) - set(fresh) == {'crates/script/rhai/tests/scenograph_rules.rs'}
assert all(cov[p]['through'] == inv['files'][p]['chars'] and cov[p]['sha256'] == sha(ROOT / p) == inv['files'][p]['sha256'] for p in all_script)
probe = load('document_net_gate_probe_receipt.json')
assert probe['build_exit'] == probe['run_exit'] == 0 and probe['checks'] == 2 and probe['original_tests'] == 0
assert probe['runner_sha256'] == sha(HERE / 'run_document_net_gate_probe.py')
assert probe['probe_sha256'] == sha(HERE / 'document_net_gate_probe.rs')
assert all(sha(ROOT / p) == h for p, h in probe['source_sha256'].items())
assert sha(Path(probe['supplier']['path'])) == probe['supplier']['sha256']
assert probe['generated_output_removed'] and all(not Path(v['path']).exists() for v in probe['removed_outputs'])

upstream = Path('C:/Users/mark_/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wasmtime-45.0.3/src/runtime')
external = [(upstream / 'store.rs', [(1110, 1140)]),
            (upstream / 'component/component.rs', [(242, 267)]),
            (upstream / 'module.rs', [(376, 405)])]
trace_specs = [
    ('crates/script/app-host/src/lib.rs', [(176, 212), (239, 274)]),
    ('crates/script/app-host/build.rs', [(17, 29), (65, 91)]),
    ('crates/script/app-host/tests/emit.rs', [(20, 40)]),
    ('crates/script/document-host/build.rs', [(65, 109)]),
    ('crates/script/document-host/src/capabilities.rs', [(218, 263)]),
    ('crates/script/document-host/src/script.rs', [(94, 125), (129, 155), (165, 176)]),
    ('crates/script/document-host/src/runtime.rs', [(119, 177)]),
    ('crates/script/document-host/src/host.rs', [(58, 100)]),
    ('crates/script/document-host/src/dom_view.rs', [(76, 143)]),
    ('crates/system/registry/src/mod_loader/loader/free_fns.rs', [(90, 100), (130, 139)]),
    ('crates/system/registry/src/mod_loader/loader/types.rs', [(151, 158)]),
    ('crates/moot/gemot/examples/capsule-library/native.rs', [(142, 161), (210, 222)])]
def excerpts(path, ranges):
    lines = path.read_text(encoding='utf-8-sig').splitlines()
    return {'path': str(path), 'sha256': sha(path), 'excerpts': [
        {'first_line': first, 'last_line': last, 'text': '\n'.join(lines[first - 1:last])} for first, last in ranges]}
save('script_host_source_traces.json', {
    'basis_commit': head, 'production': [excerpts(ROOT / p, ranges) for p, ranges in trace_specs],
    'external_contract': [excerpts(p, ranges) for p, ranges in external],
    'qualification': 'Exact source excerpts and complete-file hashes, not executable containment, AOT safety, race or stale-artifact test receipts. External dependency excerpts do not count toward Mere source coverage.'})

notes, authority = load('review_notes.json'), load('authority_map.json')
assert notes['findings'][-1]['id'] == 'Q366' and authority['entries'][-1]['id'] == 'A1059'
b = 'crates/script/document-host/src/'
findings = [
    {'id': 'Q367', 'title': 'Document network host gate and backend URL parsing can disagree on the destination',
     'source': b + 'net.rs:33,52; ' + b + 'host.rs:80,93',
     'status': 'reproduced complete unchanged helper with exact retained url2.5.8 parser; no live fetch',
     'evidence': 'With allowlist allowed.test, raw https://outside.test\\@allowed.test/private is admitted by host_in_origins as allowed.test, while url::Url parses the destination host as outside.test. Passing the normalized URL through the same gate refuses it. Ordinary allow/deny controls work. ScriptHost passes the original string directly to the injected backend after this gate.',
     'scope': probe['qualification'], 'diagnostics': ['document_net_gate_probe_receipt.json'],
     'repair': 'Parse once with the transport-appropriate URL owner, authorize the resolved destination and pass that same normalized request to the backend. Trace redirect and DNS/private-network policy separately. This proves a helper boundary mismatch, not a live backend exfiltration or an automatic scheme/port violation of the documented host-only allowlist.'},
    {'id': 'Q368', 'title': 'A safe component attach path treats the cwasm suffix as sufficient native-code trust',
     'source': b + 'capabilities.rs:221,226,243; ' + b + 'script.rs:94,107; ' + b + 'runtime.rs:159',
     'status': 'source-confirmed trust precondition is asserted in a comment rather than established by the safe loader',
     'evidence': 'load_component selects unsafe Component::deserialize_file solely by the .cwasm extension. Public safe DocumentScript::attach accepts an arbitrary Path and reaches that loader without authenticating provenance or restricting it to bytes produced by this process. The SAFETY comment assumes a trusted first-party artifact. Cached Wasmtime45.0.3 explicitly requires previously serialized/precompiled unmodified bytes; config/version checks are not validation of arbitrary native code.',
     'scope': 'No malformed native code was generated or executed, no unsafe exploit or product ingress was reproduced. Registry disk discovery requires .wasm and validates its binary, a real mitigating control for that discovery path. WasmModSource also has public path fields; direct safe attach remains broader than disk discovery.',
     'diagnostics': ['script_host_source_traces.json'],
     'repair': 'Separate ordinary validated component loading from an explicitly trusted precompiled-artifact API. Establish provenance of exact bytes before unsafe deserialization, or expose the necessary unsafe obligation at the trusted boundary. Do not claim that a filename, compatible config or signed metadata alone verifies arbitrary machine-code safety.'},
    {'id': 'Q369', 'title': 'App host epoch deadline expires across idle time and successive normal turns',
     'source': 'crates/script/app-host/src/lib.rs:251,258,264,270; ' + b + 'script.rs:134,169',
     'status': 'source-confirmed cumulative deadline; no fresh Wasmtime timing execution',
     'evidence': 'AppScript sets a relative epoch deadline once during attach and never renews it for activate, on_event or deactivate. Watchdog continuously advances the shared engine epoch, so idle ticks consume that deadline and later ordinary calls can trap. The private store exposes no rearm method. DocumentScript independently resets its deadline before each guest call. The capsule native example runs one continuous watchdog with Some400 on both app instances.',
     'scope': 'The cached upstream Store::set_epoch_deadline contract confirms the relative-to-current epoch semantics. This is a source path conclusion, not a newly run two-second idle test or live capsule failure. The app API is more primitive than DocumentScript, but its exposed methods do not let an embedder renew the private store deadline.',
     'diagnostics': ['script_host_source_traces.json'],
     'repair': 'Retain the configured call allowance and rearm before each guest entry, or expose an explicit and enforceable lifetime/call-budget contract. Share the containment mechanism where policies agree; preserve caller-selected quotas and watchdog ownership.'},
    {'id': 'Q370', 'title': 'Document extension runtime links the live net capability regardless of requested or allowed Network',
     'source': b + 'runtime.rs:119,123,129,156; ' + b + 'capabilities.rs:184,201',
     'status': 'source-confirmed grant derivation drift; no live network access demonstrated',
     'evidence': 'Runtime activate checks only capabilities the manifest requested, then grant_for ignores the manifest and returns Grant::allow_all, including net. Thus an empty allowed surface and empty capability request still link the current live net import and report it granted. Runtime documentation/tests still describe fetch as a future import. This bypasses the import-level refusal documented as the capability boundary, although the runtime currently supplies no backend and an empty origin list, so actual fetches remain refused there.',
     'scope': 'No full runtime or network trial executed. Existing empty-manifest source test expects instantiation, and the current document guest imports net. Missing backend/origin controls are real and must not be reported as an executed egress bypass. The separate Grant::from_authority path exists but this bridge does not use it.',
     'diagnostics': ['script_host_source_traces.json'],
     'repair': 'Derive the actual linked grant from requested capabilities intersected with the authoritative surface decision; explicitly document any intrinsic document/log access. Update the guest/test capability contract rather than leaving a now-live import under historical future-import assumptions.'},
    {'id': 'Q371', 'title': 'Both guest build scripts leave old artifacts usable while claiming failed builds force tests to fail',
     'source': 'crates/script/app-host/build.rs:69; crates/script/document-host/build.rs:88; crates/script/app-host/tests/emit.rs:23; crates/script/document-host/tests/document_script.rs:35',
     'status': 'source-confirmed stale-artifact success path; no failed guest compilation or stale Wasm execution this pass',
     'evidence': 'Both build scripts warn and return/continue after a nonzero guest Cargo status. Their message says loading tests will fail, but neither invalidates or binds an existing guest artifact to this build. The tests use a fixed path or environment override and only check existence before loading. An older compatible artifact can therefore satisfy those tests after the current guest build failed; it is not a receipt for the current guest source.',
     'scope': 'Allowing the host library to build without test guests can be intentional. The defect is the stronger freshness/failure claim and missing test build-result binding, not a claim that all such host builds must fail. No real Cargo guest build, stale module or original integration suite executed.',
     'diagnostics': ['script_host_source_traces.json'],
     'repair': 'Carry guest-build status and source/artifact identity into tests, and fail the required guest gate explicitly when the current build failed. Preserve an intentional library-only build path. Do not delete retained evidence or unrelated shared targets as a freshness shortcut.'}]
for f in findings:
    repair = f.pop('repair'); f['verification'] = NAME
    f['repair_triage'] = {'disposition': 'documented under active read-only full-source audit', 'reason': repair}
notes['findings'].extend(findings)
rows = [
('One WIT package is a shared ABI, not shared enforcement',
 'The app and document worlds consume the same WIT source, with distinct action versus document policies.',
 ['crates/script/wit/world.wit', 'crates/script/app-host/src/lib.rs', b + 'lib.rs'],
 'ABI reuse is real. App emissions delegate decisions to ActionSink and explicitly mean accepted for lowering, not already applied. The contract cannot force an arbitrary sink to enforce its host-only vocabulary; source tests use a named StubGate, not Turnstone live policy.'),
('App emission discovery and authorization have separate lifetimes',
 'caps.granted is a vector copied at attach, while each emission delegates to the sink.',
 ['crates/script/app-host/src/lib.rs', 'crates/moot/gemot/examples/capsule-library/native.rs'],
 'The list is advisory and can become stale under sink revocation; current capsule source rechecks authority per emission. attach_bytes consumes caller-supplied exact verified bytes, a real path-race improvement. No fresh app component or signed-grant test executed here.'),
('Containment plumbing is duplicated and already diverges',
 'App has an RAII continuous Watchdog; document has run_guarded and per-call guarded_block_on threads.',
 ['crates/script/app-host/src/lib.rs', b + 'capabilities.rs', b + 'script.rs'],
 'Q369 is the concrete cumulative-versus-per-call deadline divergence. Extension runtime is explicitly unguarded with unlimited limits, despite crate-level guarded-host wording. Cancellation/panic cleanup and per-memory versus total-host quotas require separate qualification; no hostile guest or lifecycle interruption ran.'),
('AOT provenance is separate from compile-configuration compatibility',
 'load_component shares guarded engine configuration with precompile_to_cwasm.',
 [b + 'capabilities.rs', b + 'script.rs', 'crates/system/registry/src/mod_loader/loader/free_fns.rs'],
 'Compatible config is real, but Q368 tracks the unestablished trust precondition. Registry .wasm-only disk validation is not removed or ignored in this assessment; direct attach is the broader public seam.'),
('The document grant has one provider adapter but multiple decision paths',
 'Grant::from_authority maps doc/log, doc/document and doc/net through Servitor, while runtime owns an allowed capability vector.',
 [b + 'capabilities.rs', b + 'runtime.rs'],
 'The pure adapter is actual shared AuthorityProvider reuse. Direct Grant constructors and extension grant_for are independent choices; Q370 records unconditional net linking. Document permission uses Write for the combined inspect/mutate interface, not separate read-only DOM authority.'),
('Network destination parsing is locally duplicated',
 'net_host_of explicitly mirrors an embedder helper instead of consuming a common parsed request.',
 [b + 'net.rs', b + 'host.rs'],
 'Q367 executes a destination mismatch with the retained URL parser. The source gate is a hostname allowlist that strips scheme/port and accepts a wildcard apex; those are explicit current semantics, not automatically a different exploit. Backend redirect and private-network decisions are not enforced by this helper.'),
('Async signatures do not make the current fetch seam nonblocking',
 'WIT/lib prose describes fiber suspension, but ScriptHost calls blocking NetFetcher::fetch without awaiting I/O.',
 ['crates/script/wit/world.wit', b + 'lib.rs', b + 'net.rs', b + 'host.rs'],
 'The NetFetcher trait correctly documents blocking the actor thread, contradicting stronger nonblocking prose. Epoch checks bound guest instructions, not a blocked host fetch. Per-turn request count is reset by DocumentScript, not by every alternate turn driver/lifecycle path; no actual slow backend ran.'),
('DOM mutation authority is whole-instance liveness, not snapshot-issued handles',
 'inspect can return a subtree but mutations accept any is_live NodeId in the owned DOM.',
 [b + 'dom_view.rs', b + 'host.rs', 'crates/script/wit/world.wit'],
 'Subtree is a copy-size query, not a scoped grant. Only nodes in view are nameable and capability-scoped ID prose is stronger than liveness-only validation. The shared document instance still grants WholeDocument, so no cross-document access is inferred.'),
('Atomic batch preflight checks initial liveness but not all mutation semantics',
 'apply checks revision and all referenced ids before sequential LayoutDomMut calls.',
 [b + 'dom_view.rs'],
 'Unknown initial ids reject before mutation and empty batches preserve revision. Reference validity after an earlier removal, parent/type constraints, root operations, text-element behavior and revision overflow need actual Genet contract cases before promoting an atomicity defect. InsertBefore without a parent silently does nothing but still increments the revision. Non-text blocks intentionally ignore Block.text.'),
('Page ownership recovery and host allocation lie outside linear-memory containment',
 'DocumentScript owns the page DOM; detach consumes self and returns it only after successful deactivate.',
 [b + 'script.rs', b + 'host.rs', 'crates/script/app-host/src/lib.rs'],
 'A teardown error follows ? and drops the owned page instead of returning it with the error. Both hosts accumulate unbounded log strings outside guest linear memory. These source followups need lifecycle/host policy decisions; no actual page loss, memory exhaustion or host crash was executed.'),
('Guest-build and fixture discovery are duplicated independently of WIT sharing',
 'App/document build scripts repeat child environment stripping and fixed target paths; integration tests repeat path selection.',
 ['crates/script/app-host/build.rs', 'crates/script/document-host/build.rs', 'crates/script/app-host/tests/emit.rs', 'crates/script/document-host/tests/document_script.rs'],
 'Q371 tracks old-artifact freshness. App declares rerun-if-changed for world.wit; document build does not. Both strip CARGO_TARGET_DIR, retain CARGO_HOME/jobs, and expect per-guest target directories; original builds would need workspace-hygiene review before execution. No nested target was created.'),
('Rhai shares sandbox settings while preserving two deliberate execution contracts',
 'BlockEvaluator uses the standard engine; RhaiValueEngine uses raw packages and host libm maths.',
 ['crates/script/rhai/src/lib.rs', 'crates/script/rhai/src/value.rs'],
 'Dummy resolver plus disabled import/eval and depth limits are actually shared. Block max_ops0 means unbounded; value Budget::Steps0 refuses and Unbounded maps to0, matching its explicit typed contract. These distinct contracts are not duplicated permission authorities. Value eval clones globals per call and refuses duplicate registration names.'),
('Rhai determinism fixture really is shared, with a bounded comparison claim',
 'Native test and wasm example include the same rule_bits source; JS reads its expected hash/count and asserts controls.',
 ['crates/script/rhai/tests/support/rule_bits.rs', 'crates/script/rhai/tests/maths_bits.rs', 'crates/script/rhai/tests/wasm_maths_bits.mjs', 'crates/script/rhai/examples/maths_bits.rs'],
 'Unlike the lexical print-only probe, this runner fails mismatches and its controls check empty/shorter runs. It compares one10000-position workload to a recorded64-bit digest; no broad all-input proof or fresh native-versus-browser acceptance. Node is not a browser-hosted execution receipt. The runner uses fixed root target paths rather than observing configured Cargo output.'),
('Source tests distinguish mechanisms from product receipts',
 'App uses an explicit stub gate; document uses a real ScriptedDom seam with EchoFetcher; guarded tests accept any trap.',
 ['crates/script/app-host/tests/emit.rs', 'crates/script/document-host/tests/document_script.rs', 'crates/script/document-host/tests/guarded.rs', 'crates/script/document-host/tests/runtime.rs'],
 'Those are useful scoped assertions, not fresh Turnstone, real egress, specific trap-cause or hardware proof. Entire script source is complete, including the previously read Rhai scenograph_rules test. No original test suite, full Wasmtime owner/current Cargo graph or guest module ran in this pass.')]
entries = [{'id': f'A{1060 + i}', 'domain': d, 'claim': c, 'implementations': p, 'assessment': a,
            'state': 'script source complete; executed net helper separated from source-only host conclusions', 'verification': NAME}
           for i, (d, c, p, a) in enumerate(rows)]
authority['entries'].extend(entries)
completed = []
for area, count, chars in areas:
    selected = [p for p in all_script if p.startswith(area)]
    assert len(selected) == count and sum(inv['files'][p]['chars'] for p in selected) == chars
    new = [p for p in selected if p in fresh]
    item = {'prefix': area, 'files': count, 'chars': chars, 'new_files_this_pass': len(new),
            'new_chars_this_pass': sum(inv['files'][p]['chars'] for p in new), 'basis_commit': head, 'verification': NAME,
            'qualification': 'Complete source coverage, retaining prior hash-bound Rhai scenograph_rules credit. Only the complete net helper with retained url parser executed this pass; no original Rust tests or component host/guest run.'}
    completed.append(item); notes['completed_source_areas'].append(item)
notes['review_followups'].extend([
    {'verification': NAME, 'area': 'Document lifecycle and exact mutation semantics',
     'detail': 'Document detach consumes and drops its page on teardown error; initial-liveness preflight does not establish all sequential DOM semantics. Control actual Genet root/type/remove cases and decide recovery outcome before changing the API.'},
    {'verification': NAME, 'area': 'Host containment and grant owner consolidation',
     'detail': 'Q369 is a bounded per-call rearm candidate; Q370 requires the current requested/allowed import contract. Record unguarded extension runtime, blocking host I/O, unbounded logs and async cancellation cleanup separately instead of importing page containment claims wholesale.'},
    {'verification': NAME, 'area': 'Rhai typed error and value boundary details',
     'detail': 'Nested user-function wrappers may alter ValueError categorization; identifier checks are ASCII-shape validation rather than reserved-word checks. No new failing Rhai execution was run, so retain as focused candidates rather than established bugs.'}])
next_read = {'path': 'crates/servitor/Cargo.toml', 'start': 0, 'reason': 'Continue full unread Servitor authority/cascade/gate/runtime source, then remaining repository areas.'}
notes['current_area'] = 'All script source complete; net parser mismatch reproduced; AOT, epoch, runtime grant and guest freshness findings scoped'
notes['next_area'] = next_read['reason']
notes['checkpoint'] = {'classification': 'progress', 'classification_of_previous_turn': 'progress', 'goal_status': 'active; full source incomplete',
    'coverage': stats, 'finding_count': len(notes['findings']), 'authority_entries': len(authority['entries']),
    'new_receipts': [NAME, 'document_net_gate_probe_receipt.json', 'script_host_source_traces.json'],
    'audit_authored_production_edits_this_turn': False, 'next_read': next_read,
    'pending_verification': 'No owned process live. Full Wasmtime/current Cargo graph, component idle/teardown/host-I/O/race/stale-guest and original Rust suites unexecuted; previous full metadata timeout and Q352 remain qualified.',
    'diagnostic_outcomes': 'Complete unchanged net helper freshly compiled with one exact retained url2.5.8 supplier. Two audit cases pass; zero original tests. Generated executable/PDB/link outputs removed.'}
diff = subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True, text=True); assert diff.returncode == 0
with (HERE / 'script_hosts_diff_check.txt').open('x', encoding='utf-8') as f: f.write(diff.stdout + diff.stderr)
addition = f'''

### All script source complete: destination parsing, component trust and divergent guards, 2026-10-17

Read36new complete files /176,097characters at unchanged HEAD {head}. All37files /185,641characters under crates/script are now covered: app-host7 /29,051, document-host21 /97,875, Rhai8 /51,486 and shared WIT1 /7,229. Rhai scenograph_rules.rs /9,544characters was already read and remains hash-bound; it is not counted again. Repository coverage is{stats['complete_files']:,}/{stats['total_files']:,}files /{stats['read_chars']:,}/{stats['total_chars']:,}characters, with no stale/pending credit. Full audit remains active and incomplete.

Q367 is reproduced by freshly compiling the complete unchanged net helper with one exact retained url2.5.8 supplier. The raw request https://outside.test\\@allowed.test/private passes the allowed.test host gate, while the actual URL parser reads outside.test as destination; normalizing first makes the same gate refuse. Ordinary allow/deny controls work. ScriptHost source forwards the original string to its injected backend. Two audit cases, zero original tests; no Wasmtime host, production fetch backend, HTTP request or credential transfer ran. Scheme/port differences are the documented host-only policy, not an additional executed bypass.

Q368 is source-confirmed: a safe public attach path selects unsafe native-code deserialization solely by .cwasm suffix, while the SAFETY comment assumes trusted first-party serialized bytes. Exact cached Wasmtime45.0.3 contract excerpts confirm the unmodified-precompile provenance requirement; compatible config/version is not arbitrary-code validation. Registry disk discovery only accepts validated .wasm, a real mitigation for that narrower ingress. No malformed native code or exploit executed. Q369: app-host sets its epoch deadline once at attach; a continuous watchdog consumes it across idle time and successive calls, with no public rearm of the private store. DocumentScript separately rearms each guest call. This is source-confirmed with upstream deadline semantics, not a fresh timing test.

Q370: extension runtime checks requested capabilities against its separate allowed vector, then grants all imports regardless of either input. net is now live although runtime docs/tests still call fetch future work. Its absent backend and empty origin list do currently refuse actual requests; this is import-boundary/authority drift, not an executed egress bypass. Q371: both duplicated guest-build scripts warn after failure while claiming tests will fail, but leave fixed-path artifacts unbound to current source/build success. Tests can load an older compatible guest. Library-only builds may intentionally proceed; no actual failed build/stale module or original integration test ran here.

A1060-A1073 distinguish shared WIT from enforcement, accepted-for-lowering from applied actions, snapshot discovery from per-emission sink checks, RAII/per-call/unguarded containment paths, AOT compatibility from provenance, actual Servitor adapter reuse from extension policy, local host parsing from parsed request identity, async signatures from current blocking fetch, DOM liveness from scoped handles, initial preflight from complete atomic semantics, owned-page teardown/log memory from linear-memory quotas, repeated guest build/discovery, shared Rhai sandbox versus two intentional eval contracts, genuinely shared deterministic rule fixture versus its bounded recorded digest, and source tests versus fresh product evidence. Document detach dropping the owned page on teardown error, no-parent InsertBefore succeeding as a revision-only change, unbounded host logs, sequential mutation semantics, cancellation cleanup and nested Rhai error categories remain explicit source followups; no host crash, page loss, GC/teardown semantics or broader policy is chosen during audit.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}), [net helper diagnostic](../../../support/receipts/authority-success-audit/document_net_gate_probe_receipt.json), and [exact source/contract traces](../../../support/receipts/authority-success-audit/script_host_source_traces.json). Production code, prior evidence, protected hashes, shared WIP and Git index are unchanged; all prior report bytes remain an exact prefix. Generated diagnostic executable/PDB/link outputs removed. Diff check passes. No retained isolated target, Cargo home or worktree; retained evidence remains below100MB. Next: unread Servitor authority and runtime source.
'''
doc.write_bytes(prefix + addition.encode('utf-8'))
for n, value in [('review_notes.json', notes), ('authority_map.json', authority)]:
    (HERE / n).write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')
evidence = ['record_script_hosts_review.py', 'script_hosts_diff_check.txt', 'document_net_gate_probe.rs',
    'run_document_net_gate_probe.py', 'document_net_gate_probe_receipt.json', 'document_net_gate_probe_compile.log',
    'document_net_gate_probe_output.log', 'script_host_source_traces.json']
budget = sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file()); assert budget < 100_000_000
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': head,
    'kind': 'all script source complete; helper parser diagnostic and qualified source host findings; repository incomplete',
    'prior_receipt': {'path': 'mooting_illume_probes_review_receipt.json', 'sha256': sha(HERE / 'mooting_illume_probes_review_receipt.json')},
    'coverage': stats, 'completed_source_areas': completed, 'whole_current_files': fresh, 'all_completed_script_files': all_script,
    'source_sha256': {p: sha(ROOT / p) for p in fresh}, 'preserved_production_and_test_hashes': guards,
    'auxiliary_read_sha256': aux, 'runtime_input_sha256': runtime, 'shared_working_file_sha256': shared,
    'external_contract_sha256': {str(p): sha(p) for p, _ in external},
    'source_trace_sha256': {p: sha(ROOT / p) for p, _ in trace_specs},
    'new_findings': [f['id'] for f in findings], 'authority_entries': [e['id'] for e in entries],
    'audit_authored_production_edits_this_pass': False,
    'diagnostics': {'document_net_gate_probe_receipt.json': {'build_exit': 0, 'run_exit': 0, 'checks': 2, 'original_tests': 0, 'audit_cases': 2}},
    'pending_verification': notes['checkpoint']['pending_verification'], 'next_read': next_read,
    'document': {'path': old['document']['path'], 'sha256': sha(doc), 'preserved_prefix_bytes': len(prefix), 'preserved_prefix_sha256': hashlib.sha256(prefix).hexdigest()},
    'evidence_sha256': {p: sha(HERE / p) for p in evidence}, 'git_index_sha256': index_hash,
    'receipt_budget': {'cap_bytes': 100_000_000, 'retained_bytes_before_receipt': budget}, 'retained_isolation': [],
    'limits': probe['qualification'] + ' Q368-Q371 are source-confirmed paths, not fresh AOT/epoch/permission/stale-guest execution. All original suites and full Wasmtime/current Cargo graph unexecuted.'}
save(NAME, receipt)
assert sha(index) == index_hash and doc.read_bytes().startswith(prefix)
assert all(sha(ROOT / p) == h for p, h in {**guards, **receipt['source_sha256'], **aux, **runtime}.items())
assert all(sha(HERE / p) == h for p, h in receipt['evidence_sha256'].items())
assert all(sha(ROOT / p) == h for p, h in shared.items() if p != old['document']['path'])
assert subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True).returncode == 0
print(json.dumps({'receipt': NAME, 'coverage': stats, 'new_findings': receipt['new_findings'],
                  'authority_entries': receipt['authority_entries'], 'protected_sources': len({**guards, **receipt['source_sha256']}),
                  'retained_bytes': budget}, indent=2))
