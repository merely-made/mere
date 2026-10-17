"""One-shot owner-admission refresh and standalone probe audit checkpoint."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
NAME='admission_probe_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def save(n,v):(HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
assert not (HERE/NAME).exists()
old=load('trainer_identity_review_receipt.json')
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()==old['basis_commit']
preserved={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
assert all(sha(ROOT/n)==s for n,s in preserved.items())
assert load('pending_read.json')==[] and not (HERE/'pending_transition.json').exists()
whole=['crates/system/pandect/src/graph_session.rs','crates/system/pandect/src/graph_session_transaction.rs','crates/system/pandect/src/lib.rs',
 'ports/distillery/probe/.cargo/config.toml','ports/distillery/probe/Cargo.toml',
 'ports/distillery/probe/fetch-model-matrix.ps1','ports/distillery/probe/measure-remote-minilm-vram.ps1',
 'ports/distillery/probe/remote-fixture/Cargo.toml','ports/distillery/probe/remote-fixture/src/main.rs',
 'ports/distillery/probe/repros/burn_browser_embedding/.cargo/config.toml',
 'ports/distillery/probe/repros/burn_browser_embedding/Cargo.toml',
 'ports/distillery/probe/repros/burn_browser_embedding/run-repro.ps1',
 'ports/distillery/probe/repros/burn_browser_embedding/src/lib.rs',
 'ports/distillery/probe/repros/burn_browser_embedding/tests/launcher_retirement.rs',
 'ports/distillery/probe/repros/burn_browser_embedding/web/index.html',
 'ports/distillery/probe/repros/burn_browser_embedding/web/worker.js',
 'ports/distillery/probe/repros/cubek_browser_extrema/.cargo/config.toml',
 'ports/distillery/probe/repros/cubek_browser_extrema/Cargo.toml',
 'ports/distillery/probe/repros/cubek_browser_extrema/run-repro.ps1',
 'ports/distillery/probe/repros/cubek_browser_extrema/src/lib.rs',
 'ports/distillery/probe/repros/cubek_browser_extrema/web/index.html',
 'ports/distillery/probe/repros/cubek_browser_extrema/web/worker.js',
 'ports/distillery/probe/run-model-session.ps1','ports/distillery/probe/run-probe.ps1',
 'ports/distillery/probe/run-remote-minilm.ps1','ports/distillery/probe/session-fixture/Cargo.toml',
 'ports/distillery/probe/session-fixture/src/main.rs']
inv,cov=load('source_inventory.json')['files'],load('read_coverage.json')
assert len(whole)==27 and sum(inv[n]['chars'] for n in whole)==400446
assert all(cov[n]['sha256']==inv[n]['sha256']==sha(ROOT/n) and cov[n]['through']==inv[n]['chars'] for n in whole)
proof=load('review_transition_verification_receipt.json')
assert len(proof['verified_transitions'])==4 and proof['unread_old_source_refused_without_coverage_or_pending_write']
transitions={r['path']:r['certificate'] for r in proof['verified_transitions']}
assert set(transitions)=={'Cargo.toml','ports/djinn/src/bin/djinn.rs','ports/djinn/src/embedded_reservoir.rs','ports/djinn/src/resident_mere.rs'}
for n,certificate in transitions.items():
    p=load(certificate)
    assert sha(ROOT/n)==p['current_sha256']==cov[n]['sha256']==inv[n]['sha256']
    assert cov[n]['transition_evidence']==certificate and cov[n]['through']==inv[n]['chars']
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==1328 and stats['read_chars']==17693773 and not stats['changed_since_inventory'] and not stats['pending_chunks']
directory=load('model_directory_probe_receipt.json');numeric=load('probe_numeric_validation_receipt.json')
assert directory['checks']==3 and directory['exit']==0 and sha(ROOT/directory['source'])==directory['source_sha256']
assert numeric['checks']==3 and numeric['exit']==0 and numeric['generated_output_removed']
assert all(sha(ROOT/n)==s for n,s in numeric['source_sha256'].items())
assert not list(Path('C:/t/cargo-targets/mere').glob('authority-success-probe-numeric-validation.*'))
notes,authority=load('review_notes.json'),load('authority_map.json')
assert notes['findings'][-1]['id']=='Q222' and authority['entries'][-1]['id']=='A503'
findings=[
 {'id':'Q223','title':'Model fetch directory guard accepts dot paths outside its intended model leaf',
  'source':'ports/distillery/probe/fetch-model-matrix.ps1:relativeDirectory validation and Join-Path',
  'status':'exact production PowerShell guard AST counterexample reproduced; documented, not repaired',
  'evidence':'The three actual production directory assignment/guard/join statements accept /models/.. and resolve it to the Mere repository root. /models/. aliases the model root itself. The normal MiniLM child control stays strictly below models. The guard forbids separators and punctuation but does not exclude dot and dot-dot path components.',
  'scope':'Three native PowerShell guard observations only. No curl, network, directory creation, model download, file move or overwrite is executed. Caller-supplied malformed matrix paths, not an externally reachable attack or damage from the current committed matrix.',
  'diagnostic':'model_directory_probe_receipt.json',
  'repair_triage':{'disposition':'documented in active read-only full-source audit','size':'bounded leaf-path validation',
   'reason':'Reject dot components and verify the normalized absolute destination is strictly below the intended models root before creating directories or replacing files. Keep the single directory derivation instead of introducing another independently interpreted path.'}},
 {'id':'Q224','title':'Remote inference timing includes the deliberate telemetry stage hold',
  'source':'ports/distillery/probe/remote-fixture/src/main.rs:report_stage and first/recovery execution timers; measure-remote-minilm-vram.ps1:StageHoldMs',
  'status':'source-confirmed measurement defect; no new GPU execution',
  'evidence':'Both first_execution_ms and recovery_execution_ms are sampled after report_stage for their completed remote execution. report_stage sleeps for DISTILLERY_REMOTE_STAGE_HOLD_MS. The driver telemetry wrapper sets that variable from StageHoldMs, default3000. The resulting remote_execution values therefore include the deliberate three-second hold rather than ending at inference completion.',
  'scope':'Complete fixture and wrapper source read. The default direct fixture has zero hold; positive holds through the telemetry wrapper contaminate its execution durations. No actual GPU or wall-clock performance figure was measured here, and the allocator/driver release predicates are not invalidated by this timing observation.',
  'repair_triage':{'disposition':'documented in active read-only full-source audit','size':'bounded timer boundary correction',
   'reason':'Capture inference elapsed time immediately when inference returns, then emit/hold the telemetry stage. Keep stage sampling and its configured hold separate from the reported inference duration.'}},
 {'id':'Q225','title':'Session fixture max-error helper silently ignores non-finite coordinate differences',
  'source':'ports/distillery/probe/session-fixture/src/main.rs:max_abs; remote-fixture/src/main.rs:max_abs_error',
  'status':'exact extracted native numeric helper divergence reproduced; documented, not repaired',
  'evidence':'The session helper maps differences and folds with f32::max without a finite check. NaN-minus-NaN and infinity-minus-the-same-infinity are NaN and disappear during that reduction: a paired non-finite coordinate plus a finite2 difference returns Ok(2). The remote fixture helper rejects these inputs. Finite and unequal-length controls show where both agree.',
  'scope':'Three native checks using the two verbatim pure std-only functions, synthetic float values only. The full session fixture separately checks adapted and independent logits for finiteness, so this is not proof a complete model receipt passes NaNs. Its base-logit comparison and published-checkpoint audit reuse the weaker helper; actual non-finite model execution was not performed.',
  'diagnostic':'probe_numeric_validation_receipt.json',
  'repair_triage':{'disposition':'documented in active read-only full-source audit','size':'bounded shared numeric validation',
   'reason':'Reject non-finite operands before difference reduction and share the comparison contract across fixtures. Keep the independent PEFT merge arithmetic independent; duplicating a weaker validation helper does not strengthen that numerical control.'}},
]
for f in findings:f['verification']=NAME
notes['findings'].extend(findings)
q216=next(f for f in notes['findings'] if f['id']=='Q216')
q216['later_source_qualification']={'verification':NAME,'basis_commit':old['basis_commit'],
 'note':'Upstream now expects one link.surface and separately asserts the three retained source relation kinds, including after fork. This source correction addresses the historical stale assertion; no new execution of that resident test ran in this pass.'}
notes['candidates'].extend([
 {'title':'Remote MiniLM fixture labels expected model hashes without observing the supplied directory bytes',
  'source':'ports/distillery/probe/remote-fixture/src/main.rs:run_local/run_remote receipt model fields',
  'note':'Both receipt arms emit fixed model/revision/weights size/SHA256 constants while loading a caller-supplied model directory. The session fixture instead computes size/SHA256 from actual loaded bytes. Fixed first-eight numerical reference plus native comparison is real but is not an artifact-identity check. Trace ESP loader and fetch-wrapper identity guarantees before claiming a successful wrong-artifact GPU receipt; none was executed.'},
 {'title':'Browser worker error observation only wraps popErrorScope',
  'source':'ports/distillery/probe/repros/{burn_browser_embedding,cubek_browser_extrema}/web/worker.js and index.html',
  'note':'Both copied workers collect returned error scopes and awaited-call failures; their pages collect window errors/rejections. No worker-global unhandledrejection/uncapturederror/device-lost receipt handler is present here. Whether an actual detached worker/device error can coexist with success needs a browser failure injection. No browser/GPU run or planted worker error was performed.'},
 {'title':'Unlocked primary browser-probe build does not enforce the comment that selected dependency graph remains pinned',
  'source':'ports/distillery/probe/run-probe.ps1; standalone .cargo/config.toml portable-bundle comment',
  'note':'run-probe explicitly omits --locked to tolerate unused patch ordering changes and assumes selected packages are unchanged; the two minimal repro runners use --locked. It performs no before/after selected-graph comparison or resulting lock hash receipt. This is an evidence/reproducibility boundary to qualify, not proof that current successful builds selected different versions.'},
])
rows=[
 ('Replay and new-reference admission','Structural replay and current owner availability are distinct shared validators.', ['GraphValidator','GraphAdmissionValidator','GraphSession::validate'], 'Both validators run against edits before candidate installation; pure cold replay is structurally checked without inventing live referents for historical subjects. The new domain validator was reread in the prior checkpoint.'),
 ('Transaction and fork policy sharing','The detached transaction writer and whole-session copies retain the same admission policy.', ['GraphSession::detached_candidate','begin_copy_at','pending_with'], 'The new callback is cloned into staged transactions and forks. pending_with validates the current graph against itself, so already-admitted history is not reintroduced as new references. Source coverage does not mean those original tests were freshly executed.'),
 ('One owner lookup in both hosts','Daemon and embedded composition supply the actual PersonaeHost authority.', ['MereRoutes::with_profile_subject_authority','embedded_reservoir attachment','bin/djinn route composition'], 'Missing lookup refuses new profile subjects; composed lookup invokes with_profile_authority. Domain structural truth remains shared rather than duplicating profile authority. Graphshell adapter forwarding was inspected at its method only, not claimed as a new whole-file read.'),
 ('Canonical projection and retained assertions','A painted pair is not the source assertion store.', ['resident_mere shared route test','link.surface','disclosed_graph retained relations'], 'Upstream source corrects Q216 by expecting one painted link and explicitly retaining three source kinds after fork. Historical failure remains valid at its old hash; current execution is unverified.'),
 ('Verified retained source coverage','Previously read unchanged source can carry across a changed whole-file hash only with a complete proof.', ['review_transition.py','four transition certificates','verify_review_transitions.py'], 'Four fully read old sources were reconstructed at the exact Git/checkout hashes.351600 current characters are text-identical mapped spans;276 new/changed lines were emitted and acknowledged. Independent hunk application checks every current character. An unread-old-source control refuses without changing coverage.'),
 ('Shared device-scoped remote service','The standalone remote fixture composes production Distillery/RemoteSessionService.', ['run_remote','Distillery::attach_remote_sessions','MeshHost registry','RemoteSessionService mount'], 'It uses two local p2panda endpoints and actual lease-bound Burn Remote service, not a second scheduler. Manual clock and NoCustody are explicit fixture limits. This pass reads source and runs no remote model/GPU workload.'),
 ('Allocator truth versus driver truth','The Rust fixture explicitly separates active allocator release from driver allocation.', ['AllocatorSnapshot','await_allocator_baseline','physical_gpu_allocation_release','measure-remote-minilm-vram.ps1'], 'Rust records physical driver measurement false and reserved cache outside its active-allocation gate. The wrapper adds process counters plus NVIDIA PID attribution and rejects missing active samples. This separation is real; Q224 only contaminates reported timing.'),
 ('Telemetry and timing','Configured stage holds are observation plumbing and must not become inference latency.', ['report_stage','first_execution_ms','recovery_execution_ms'], 'Q224 samples elapsed time after the hold. Default telemetry wrapper adds3000ms at each affected boundary; no new GPU benchmark was run.'),
 ('Numeric validation drift','Two fixture-specific maximum-error implementations disagree on non-finite data.', ['session-fixture::max_abs','remote-fixture::max_abs_error'], 'Q225 is reproduced with exact extracted helpers; the stronger helper checks lengths and finite operands. Full session receipt also checks two logits arrays for finiteness, so helper failure is not overclaimed as integrated false success.'),
 ('Independent LoRA arithmetic','A separate CPU merger is a numerical control, while session execution uses the shared loader.', ['independent_peft_merge','ndarray B dot A','PeftLoraAdapterLoader','Eidetic model round trip'], 'The fixture hashes actual loaded bytes and compares independent-merge logits/tokens against actual bound ESP session output. Separate reference arithmetic is useful evidence, unlike repeated weaker validation logic. No real model was run in this pass.'),
 ('Copied browser receipt gates','Two standalone repros repeat the window/worker receipt machinery.', ['burn_browser_embedding web/index.html and worker.js','cubek_browser_extrema web/index.html and worker.js'], 'Window error gates, worker timeouts, popErrorScope wrapping, pinned bindgen checks and wasm cfg are repeated. Three reviewed cfg files are byte-identical today. Standalone minimal reproduction is an intentional boundary; no fake second production inference engine is inferred.'),
 ('Pinned model resolver copies','One model manifest is parsed by several local readers.', ['fetch-model-matrix.ps1::Resolve-Row','reference-fixtures.py::resolve_row','remote fixed MiniLM receipt metadata'], 'The artifact manifest is shared data but parser/field mapping is repeated and remote receipt model hashes are fixed labels. Q223 demonstrates path admission is not supplied by merely sharing that manifest. Remaining primary browser reader is still unread.'),
]
entries=[{'id':f'A{504+i:03d}','domain':d,'claim':c,'implementations':impl,'assessment':a,
 'state':'current source review; executed scope named in assessment','verification':NAME} for i,(d,c,impl,a) in enumerate(rows)]
authority['entries'].extend(entries)
notes['coverage_transition_rule']={'verification':'review_transition_verification_receipt.json',
 'rule':'Linear reading remains the default. Previously fully read sources may retain coverage only through exact Git/checkout-hash reconstruction, verified text-identical spans, complete emitted changed hunks and a separate acknowledgment. The full new text is not claimed as newly emitted; all current characters have one checked provenance.',
 'unchanged_previously_read_chars_carried_this_pass':351600,'changed_lines_inspected':276}
notes['current_area']='Upstream owner admission and projection qualifications; standalone remote/session/browser repro tools and numeric receipt boundaries'
notes['next_area']='Remaining primary Distillery browser Rust and web source, then the next unread product ownership area'
pending={**old['pending_verification'],'state':'Same live session32507 verified this turn; dependency fetching has advanced through Murm/Genet to Netrender/Cleromancy. No terminal result or executed original test yet. Poll same handle; do not restart on observation timeout.'}
notes['checkpoint']={'classification':'progress','classification_of_previous_turn':'progress','goal_status':'active; full-source objective incomplete',
 'coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),
 'new_receipts':[NAME],'production_edits_this_turn':False,
 'next_read':{'path':'ports/distillery/probe/src/lib.rs','start':0,'reason':'Five primary browser probe files remain unread; standalone fixtures/repros and seven upstream changes now have current coverage.'},
 'diagnostic_outcomes':'Three exact PowerShell directory-guard checks and three exact native numeric-helper checks pass. Timing bug is source-confirmed only. Four source transitions independently verified plus unread-source refusal control. No GPU/browser/model run.',
 'pending_verification':pending,'coverage_note':'27 whole files400446 chars emitted; four additional changed files refreshed through proven unchanged prior reads plus276 changed lines. No stale acknowledged hashes or pending chunks.'}
doc=ROOT/old['document']['path'];original=doc.read_bytes();assert hashlib.sha256(original).hexdigest()==old['document']['sha256']
addition=f'''

### Owner admission refresh and probe evidence boundaries, 2026-10-17

Current coverage is {stats['complete_files']:,} of {stats['total_files']:,} files and {stats['read_chars']:,} of {stats['total_chars']:,} current characters. This pass emits27 whole files400446 characters, including complete current Pandect session/transaction/library sources and24 new standalone probe, launcher and fixture sources. All seven upstream-invalidated source paths now have current coverage. The full Mere review remains incomplete; five primary Distillery browser Rust/web files remain unread, and other product areas are still outstanding.

Four changed sources were refreshed through verified retained coverage rather than claiming they were emitted whole again: Cargo.toml and Djinn's daemon, embedded reservoir and resident_mere. Each old file had already been fully acknowledged. Its exact Git/checkout bytes were reconstructed at the prior hash,351600 current characters were proved identical to previously read spans, and276 changed lines were emitted with deleted lines and context, inspected and separately acknowledged. An independent verifier applies every emitted hunk against both full sources, proves every current character has exactly one coverage source, and confirms an unread-old-source attempt refuses without modifying coverage. The original linear reader is unchanged. The four immutable transition certificates and review_transition_verification_receipt.json make this coverage method explicit; it is not grep, compiler reach or an inferred whole-file read.

The admission seam is shared. Structural GraphValidator and new-reference GraphAdmissionValidator both check edit candidates. Detached transaction copies and whole-session forks retain both callbacks. Historical replay uses structural validation; current owner lookup is needed for newly introduced profile ids. Both daemon and embedded compositions supply PersonaeHost to MereRoutes, and an absent lookup refuses new profile subjects. The forwarding adapter method was inspected but is not counted as a whole receiving-host source read. Original upstream refusal tests inspect graph/history/revision/storage; they were read, not newly executed. Upstream also corrects Q216's stale projection assertion: one painted link.surface is expected, and three retained source relation kinds are separately checked, including after fork. This is a current source correction; its historical failed gate is not erased or converted into a fresh green runtime receipt.

Q223: fetch-model-matrix.ps1's purported safe model-directory guard accepts /models/.. and resolves it to the repository root, while /models/. aliases models itself. The ordinary child control stays strictly below models. The exact three production AST statements were evaluated natively with synthetic paths. No curl, download, directory creation, move or overwrite ran. This is malformed caller-supplied matrix input, not a claim of an externally reachable attack or existing matrix damage. Reject dot components and check the normalized absolute destination before creating or replacing files.

Q224: the remote fixture captures first/recovery inference elapsed time after report_stage has slept for the configured telemetry hold. The driver wrapper defaults that hold to3000ms, so both reported remote_execution values include the deliberate hold. This follows directly from the complete fixture/wrapper source and does not require a new GPU benchmark. The default direct fixture has zero hold. Capture elapsed time before emitting/holding the completed stage. This timing defect does not invalidate allocator/driver release gates by itself.

Q225: two maximum-error helpers have drifted. The session fixture folds differences with f32::max and silently discards NaN coordinates, including infinity minus itself. The remote helper refuses non-finite operands. Exact verbatim extracted helpers compiled natively reproduce the divergence; finite and unequal-length controls pass. The full session receipt separately checks adapted and independent logits for finiteness, so a complete model false-pass is not proved. Its base-logit and published-checkpoint comparisons reuse the weaker helper. Share a strict numeric-validation contract while retaining the independent CPU PEFT merge arithmetic as a useful separate numerical control. No actual model, GPU or browser was run.

Authority entries A504-A515 distinguish real shared service composition from copied evidence plumbing. The remote fixture mounts actual Distillery/RemoteSessionService and uses two local endpoints, manual time and explicit NoCustody. Active CubeCL allocator release is separated from physical driver measurement: the Rust receipt records physical measurement false, and the wrapper adds process counters/NVIDIA PID attribution and rejects missing active samples. The LoRA session fixture actually hashes its loaded bytes, round-trips through Eidetic and calls the shared ESP adapter loader, comparing a separate CPU merge. In contrast, remote MiniLM receipt fields use fixed expected model/revision/hash/size labels for a caller-supplied directory; the fetch/loader identity guarantee needs further tracing before claiming a successful wrong-artifact GPU receipt.

The two standalone browser repros copy page-error gates, worker timeout/error-scope wrappers and launcher policy. Three reviewed wasm cfg files are byte-identical at this snapshot. A self-contained minimal reproducer is an intentional boundary; copied receipt validation can still drift independently. Window error handling is not a worker-global unhandled-error/device-lost gate, but no browser failure injection was performed, so that stays a candidate. The primary probe runner also omits --locked while assuming selected package identity is unchanged, unlike the two locked repro runners; there is no selected-graph before/after proof in that script. No actual dependency drift is asserted from that source fact alone.

No production or original test source was edited. Shared WIP and the prior canonical document prefix remain byte-preserved. The numeric diagnostic's native executable/PDB/import outputs were removed, and the directory diagnostic created no model files. No isolated target, Cargo home or worktree was created. The existing original FLoRA gate is still live at session32507, now fetching Netrender/Cleromancy after Murm/Genet; it has no terminal result or executed test count yet. Continue polling that same handle before retrying. The [owner/probe checkpoint](../../../support/receipts/authority-success-audit/{NAME}) records the six new native checks, source-only timing defect, coverage proofs and remaining verification.
'''
doc.write_bytes(original+addition.encode('utf-8'));save('review_notes.json',notes);save('authority_map.json',authority)
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
(HERE/'admission_probe_diff_check.txt').write_text(diff.stdout+diff.stderr,encoding='utf-8');assert diff.returncode==0
evidence=['record_admission_probe_review.py','review_transition.py','verify_review_transitions.py',
 'review_transition_verification_receipt.json','review_transition_unread_control.log','model_directory_probe.ps1','model_directory_probe_receipt.json',
 'run_probe_numeric_validation.py','probe_numeric_validation.rs','probe_numeric_validation_compile.log','probe_numeric_validation_output.log',
 'probe_numeric_validation_receipt.json','admission_probe_diff_check.txt']+list(transitions.values())
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':old['basis_commit'],
 'kind':'Owner admission requalification and standalone probe review; full Mere objective incomplete',
 'production_edits_this_pass':False,'coverage':stats,'whole_current_files':whole,'whole_current_chars':400446,
 'current_files_refreshed_with_verified_prior_read_spans':transitions,'verified_prior_read_unchanged_chars':351600,'changed_lines_inspected':276,
 'source_sha256':{n:sha(ROOT/n) for n in whole+list(transitions)},
 'trace_sha256':{'ports/graphshell/src/mere_host.rs':sha(ROOT/'ports/graphshell/src/mere_host.rs')},
 'preserved_production_and_test_hashes':preserved,
 'prior_receipt':{'path':'trainer_identity_review_receipt.json','sha256':sha(HERE/'trainer_identity_review_receipt.json')},
 'new_findings':[f['id'] for f in findings],'authority_entries':[e['id'] for e in entries],
 'native_observations_and_controls':6,'source_only_new_finding':'Q224 timing boundary; no GPU measurement',
 'original_tests':'No new original tests executed yet; original FLoRA feature gate remains fetching dependencies',
 'pending_verification':pending,'next_read':notes['checkpoint']['next_read'],
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(original),'preserved_prefix_sha256':hashlib.sha256(original).hexdigest()},
 'evidence_sha256':{n:sha(HERE/n) for n in evidence},'probe_outputs_removed':True,'retained_isolation':[],
 'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},
 'limits':'No full source completion. No model/GPU/browser/native-headed/installed/physical gate run. Numeric helper observation is not an integrated NaN false-pass; model metadata/worker-global errors/unlocked dependency-graph stability remain candidates. Four refreshed files explicitly retain proven prior reads rather than claiming full re-emission.'}
save(NAME,receipt)
print(json.dumps({'receipt':NAME,'coverage':stats,'whole_files':len(whole),'whole_chars':400446,
 'verified_transition_files':len(transitions),'new_findings':receipt['new_findings'],
 'authority_entries':receipt['authority_entries'],'guard_paths':len(preserved),'retained_bytes':budget},indent=2))
