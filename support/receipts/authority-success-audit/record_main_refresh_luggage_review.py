"""Record exact main reread, subsequent Luggage review, bugs and ownership seams."""
from pathlib import Path
from datetime import datetime,timezone
import difflib,hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];NAME='main_refresh_luggage_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT,text=True).strip()
def save(n,v):
    with (HERE/n).open('x',encoding='utf-8') as f:json.dump(v,f,indent=2);f.write('\n')
assert not (HERE/NAME).exists()
old=load('stickleback_completion_review_receipt.json');head=git('rev-parse','HEAD');assert head==old['basis_commit']
guards={**old['preserved_production_and_test_hashes'],**old['source_sha256']};assert len(guards)==2142
aux,runtime,shared=old['auxiliary_read_sha256'],old['runtime_input_sha256'],old['shared_working_file_sha256']
assert all(sha(ROOT/p)==h for p,h in {**guards,**aux,**runtime}.items())
assert all(sha(HERE/p)==h for p,h in old['evidence_sha256'].items())
doc=ROOT/old['document']['path'];prefix=doc.read_bytes();assert sha(doc)==old['document']['sha256']
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
index=Path(git('rev-parse','--git-path','index'))
if not index.is_absolute():index=ROOT/index
index_hash=sha(index);assert index_hash==old['git_index_sha256']
inv,cov,refresh=load('source_inventory.json'),load('read_coverage.json'),load('main_refresh_after_stickleback_receipt.json')
changed=sorted(set(refresh['inventory_changed'])|set(refresh['inventory_added']))
luggage=[p for p in inv['files'] if p.startswith('crates/system/luggage/')]
fresh=sorted(changed+luggage);assert len(changed)==27 and len(luggage)==10 and len(fresh)==37
assert sum(inv['files'][p]['chars'] for p in changed)==772041 and sum(inv['files'][p]['chars'] for p in luggage)==106957
assert not set(fresh).intersection(guards)
assert load('pending_read.json')==[] and load('main_delta_pending.json')==[]
assert all(cov[p]['through']==inv['files'][p]['chars'] and cov[p]['sha256']==sha(ROOT/p)==inv['files'][p]['sha256'] for p in fresh)
delta=load('main_delta_plan.json');before=load('before_main_refresh_source_inventory.json');oldcov=load('before_main_refresh_read_coverage.json')
assert delta['old_inventory_sha256']==sha(HERE/'before_main_refresh_source_inventory.json') and delta['old_coverage_sha256']==sha(HERE/'before_main_refresh_read_coverage.json')
assert len(delta['rows'])==14
for row in delta['rows']:
    p=row['path'];assert row['acknowledged_blocks']==len(row['blocks']) and oldcov[p]['through']==before['files'][p]['chars'] and oldcov[p]['sha256']==row['old_sha256']
    raw=subprocess.check_output(['git','show',row['old_git_commit']+':'+p],cwd=ROOT)
    if row['old_bytes_reconstruction']=='checkout_crlf':raw=raw.replace(b'\r\n',b'\n').replace(b'\n',b'\r\n')
    assert hashlib.sha256(raw).hexdigest()==row['old_sha256'] and sha(ROOT/p)==row['new_sha256']
    a=raw.decode('utf-8-sig').splitlines(keepends=True);b=(ROOT/p).read_bytes().decode('utf-8-sig').splitlines(keepends=True)
    rendered=[];same=new=0
    for tag,i,j,k,l in difflib.SequenceMatcher(None,a,b,autojunk=False).get_opcodes():
        if tag=='equal':assert a[i:j]==b[k:l];same+=len(''.join(b[k:l]))
        else:
            new+=len(''.join(b[k:l]));rendered.append(''.join([f'@@ {tag}: old lines {i+1}:{j}, new lines {k+1}:{l} @@\n',
                *[' '+x for x in a[max(0,i-3):i]],*['-'+x for x in a[i:j]],*['+'+x for x in b[k:l]],*[' '+x for x in b[l:min(len(b),l+3)]]]))
    assert ''.join(rendered)==''.join(row['blocks']) and same==row['unchanged_previously_read_chars'] and new==row['changed_new_chars'] and same+new==row['new_chars']
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==2649 and stats['read_chars']==31453530 and not stats['changed_since_inventory'] and stats['pending_chunks']==0
probe=load('main_refresh_staging_probe_v2_receipt.json');relocation=load('staging_build_relocation_receipt.json')
assert probe['exit']==0 and probe['passed_tests']==27 and probe['original_tests']==24 and probe['audit_tests']==3
assert probe['runner_sha256']==sha(HERE/'run_main_refresh_staging_probe_v2.py') and probe['probe_sha256']==sha(HERE/'main_refresh_staging_probe.rs')
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert all(sha(HERE/p)==h for p,h in probe['projections'].items())
assert relocation['source_directory_removed'] and not Path(relocation['source']).exists() and relocation['live_owners_before_move']==0 and not relocation['isolated_target_retained']
assert len(relocation['removed_outputs'])==2 and all(not Path(v['path']).exists() for v in relocation['removed_outputs'])
assert Path(relocation['destination']).joinpath('CACHEDIR.TAG').read_text().startswith('Signature: 8a477f597d28d172789f06886806bc55')
assert not list(Path(relocation['destination']).joinpath('debug/deps').glob('authority_success_refresh_staging-*.exe'))
assert all(sha(HERE/p)==v['sha256'] for p,v in probe['fixture_files'].items())
upstream=git('rev-parse','origin/main');assert upstream=='00e4442868ee1cb014364a81ccaed657b512b218'
assert git('rev-list','--left-right','--count','HEAD...origin/main').split()==['0','8']
reviewed_upstream=['.github/workflows/portable.yml','.github/workflows/rust-policy.yml','Cargo.toml','rust-toolchain.toml',
 'crates/cambium/cambium-rootstock/src/embedding.rs','crates/cambium/cambium-rootstock/src/embedding/egui_input.rs',
 'crates/cambium/cambium-rootstock/src/embedding/iced_input.rs','crates/cambium/cambium-rootstock/src/embedding/tests.rs',
 'crates/canvas/pictograph/src/canvas/scene_axes.rs','crates/canvas/pictograph/src/canvas/tests/bounded_scenes.rs',
 'crates/script/app-host/guest/rust-toolchain.toml','crates/script/document-host/guest-bomb/rust-toolchain.toml',
 'crates/script/document-host/guest/rust-toolchain.toml','ports/dramatis/package.py']
upstream_files=git('diff','--name-only',head,upstream).splitlines()
remote={'basis_commit':head,'upstream_commit':upstream,'ahead':0,'behind':8,'integration_exit':1,
 'integration_error':'git merge --ff-only origin/main refused: local changes to design_docs/mere_docs/implementation_strategy/2026-10-07_scenograph_editor_plan.md would be overwritten',
 'changed_files':upstream_files,'reviewed_diffs':{p:{'diff_sha256':hashlib.sha256(subprocess.check_output(['git','diff','--unified=3',head,upstream,'--',p],cwd=ROOT)).hexdigest(),
 'upstream_git_bytes_sha256':hashlib.sha256(subprocess.check_output(['git','show',upstream+':'+p],cwd=ROOT)).hexdigest()} for p in reviewed_upstream},
 'remaining_diffs':[p for p in upstream_files if p not in reviewed_upstream],
 'qualification':'Fourteen exact upstream diffs were emitted/read; not all38files, not a completed upstream source snapshot, not local read credit. No stash/revert/commit/merge of shared WIP. Continue remote diff review read-only while preserving the current checkout/report.'}
save('upstream_after_inference_review_receipt.json',remote)
notes,authority=load('review_notes.json'),load('authority_map.json');assert notes['findings'][-1]['id']=='Q381' and authority['entries'][-1]['id']=='A1131'
findings=[
 {'id':'Q382','title':'The independent architecture logit-error subcheck silently discards NaN mismatches',
  'source':'crates/intel/esp/src/infer/decoder/architecture_tests.rs:195-202',
  'status':'reproduced exact comparison expression in a fresh CPU graph; no whole architecture model execution',
  'evidence':'The exact iter/zip/absolute-error/f32::max expression reports0for [NaN,2] versus [1,2], and also for an all-NaN actual row. Finite exact-match control gives0and finite mismatch[9,2] gives8and fails tolerance. f32::max ignores a NaN operand, so its 0seed suppresses every non-finite mismatch. Later token-ID assertions are separate and were not claimed to pass for a model producing these rows.',
  'scope':'Projected unchanged expression only; full independent reference fixture/model/Burn/GPU unexecuted. Related Q348 covers the separate expected-token rank evaluator; this is a new reference-comparison acceptance check with the same missing output-validity boundary.',
  'diagnostics':['main_refresh_staging_probe_v2_receipt.json'],'verification':NAME,
  'repair_triage':{'disposition':'bounded numerical acceptance repair candidate','reason':'Reject non-finite actual/reference rows and require exact lengths before tolerance comparison; share explicit numeric-validity checks where appropriate. Do not infer reference agreement from a NaN-suppressing maximum.'}},
 {'id':'Q383','title':'Reloaded staged update authenticity is replaced by a digest stored beside the mutable artifact',
  'source':'crates/system/luggage/src/staging.rs:35-47,68-102,164-166;crates/system/luggage/src/install.rs:190-192',
  'status':'reproduced unchanged production StagedUpdate load/take_verified methods over real filesystem and BLAKE3',
  'evidence':'An intact audit-owned staged payload/record pair reloads and passes. Changing only the artifact is refused. Changing its digest in neighbouring staged.json as well makes take_verified return the substituted unsigned bytes. The record retains no publisher key, signature or signed-manifest commitment. install_staged only invokes this check before passing bytes to install; that native handoff was source reviewed and was not executed.',
  'scope':'Requires write access to both stage files. Scoped unchanged methods included before cfg(test) over an audit Update facade whose installer panics and was never called. Initial directory fixture was synthetic rather than a genuine signed download. No remote ingress exploit, privilege escalation, actual installer or user file was exercised. The observed failure directly contradicts the module assurance about another process reaching the staged file across restart.',
  'diagnostics':['main_refresh_staging_probe_v2_receipt.json'],'verification':NAME,
  'repair_triage':{'disposition':'document high-effect restart trust migration; broader than a one-hour audit repair',
    'reason':'Re-establish publisher authorization using caller-trusted key policy and retained signed evidence at apply time. An editable record cannot be its own trust root. Bind target, version, format and artifact identity together, define old-stage migration and restart durability, and preserve the single-file tamper control. Do not silently choose a new publisher trust policy.'}},
 {'id':'Q384','title':'Discarding a staged update reports success when both deletions fail',
  'source':'crates/system/luggage/src/staging.rs:113-117',
  'status':'reproduced unchanged production discard method with audit-owned filesystem failure fixtures',
  'evidence':'After loading a real staged-record object, both known audit files are replaced with empty directories so remove_file fails. discard returns Ok while both paths remain. The fixture creates only small audit-owned files/directories; the result does not claim a still-loadable offer or real permission failure.',
  'scope':'Exact method over real filesystem, no mocked removal API and no user files. NotFound and other removal errors are all currently discarded.',
  'diagnostics':['main_refresh_staging_probe_v2_receipt.json'],'verification':NAME,
  'repair_triage':{'disposition':'bounded false-success repair candidate','reason':'Treat already-absent files as idempotent success and propagate other cleanup failures. Report partial cleanup truthfully and retain enough state to retry.'}},
 {'id':'Q385','title':'Windows MSI update handoff exits successfully even when neither launcher starts',
  'source':'crates/system/luggage/src/install.rs:321-349',
  'status':'source-confirmed Windows failure path; no native installer process execution',
  'evidence':'If starting PowerShell fails, WiX/MSI handoff attempts msiexec directly but discards that spawn Result. The branch then unconditionally calls process::exit(0). Thus the caller terminates successfully even when the fallback also failed before starting any installer. A successful launcher spawn also does not prove installer completion; this finding is the narrower both-launches-fail branch.',
  'scope':'Full platform source read. No installer, PowerShell replacement, failed real update or process exit experiment ran. NSIS uses a different panic-on-spawn-failure branch and is not claimed to share this exact false0 path.',
  'verification':NAME,'repair_triage':{'disposition':'bounded launch-result repair candidate; platform transaction work remains documented',
    'reason':'Preserve and surface failure when both launch paths fail, and exit only after an accepted handoff. Broader completion receipts, rollback and platform replacement semantics require explicit design and should not delay the remaining source audit.'}}
]
notes['findings'].extend(findings)
d='crates/intel/esp/src/infer/';l='crates/system/luggage/src/'
rows=[
 ('Declared decoder architecture now drives more of the actual kernel','DecoderConfig carries head width, window pattern, biases, QK norms, scaling and sparse-layer selection.',[d+'decoder/config.rs',d+'decoder/attention.rs',d+'decoder/layer.rs'],'Dimension/refusal checks precede ordinary parsed execution. Query width now reaches Q/O shapes and LoRA dimensions. Public typed config constructors remain caller-controlled; no complete device admission or every-family support is inferred. Sixteen unchanged original config tests ran.'),
 ('Tensor-name ownership is shared between checkpoint extraction and loader admission','layer_core/all_tensor_names and expert_tensor_names describe exactly consumed artifacts.',[d+'decoder/loader.rs',d+'decoder/checkpoint.rs'],'This is a real ownership consolidation rather than a label: import and execution consult the same grammar. Exact name checks reject extra expert/core weights and shape extraction checks required tensors. Legacy rotary buffers remain deliberately ignored; their stored values are not authenticated execution inputs.'),
 ('Model-library rows now own layer and expert lookup together','ResolvedInferenceRow::artifact_source supplies exact artifact identities and limits while the host lends only a reader.',[d+'library.rs',d+'layer_library.rs','ports/distillery/src/inference.rs','ports/distillery/src/inference_stage.rs'],'Distillery removes its local layer-only lookup closure and preserves read_expert when cloning sources. Cancellation wraps both reads at the stage boundary. LayerWeightIndex, FormattedLayerWeightIndex and ExpertWeightIndex are versioned composed artifact profiles, not three independent durable stores. Actual store locking remains host-owned and should not span tensor execution.'),
 ('Resident, streamed and staged kernels are placements under shared math','MoE expert selection is shared; each source path supplies resident or individually verified streamed weights.',[d+'decoder/moe.rs',d+'decoder/model.rs',d+'decoder/stage.rs',d+'decoder/streaming.rs'],'Routing rejects non-finite mass, bounds each encoded artifact and syncs before releasing streamed weights. Shared row sharding bounds an individual f32 matrix; it is not a total-process or total-GPU memory claim. Stage reserve now includes expert bank and hybrid dense width. No real kernel or memory test ran this pass.'),
 ('New inference support does not imply supported training or browser routes','Three adapter/training entrances explicitly reject MoE and the sharded source refuses Wasm.',[d+'decoder/lora.rs',d+'decoder/train.rs',d+'decoder/train_autodiff.rs',d+'decoder/model.rs'],'These are enforced stop lines, not merely planned exclusions. F32 expert execution is distinct from formatted dense paths and inference routing is intentionally nondifferentiable. Qualified model support should name the actual route and device.'),
 ('Template end-of-turn state now converges in the decoder provider','with_template_eos merges a known tokenizer token into one provider EOS list used by both generation routes.',[d+'decoder/provider.rs',d+'library.rs'],'This reduces an EOS policy split. Q349 Unicode streaming prefix loss remains in unchanged code, and Q347 sampler/Q348 expected-rank issues are not repaired by this migration. Q217 actor-disconnect handling likewise receives no change in the Distillery delta; none of those older diagnostics was rerun.'),
 ('Numerical acceptance checks still have multiple incomplete implementations','The new independent reference tolerance uses a NaN-suppressing maximum, separately from rank and generation acceptance.',[d+'decoder/architecture_tests.rs',d+'decoder/train.rs'],'Q382 reproduces this comparator only. Shared source math does not provide a single numerical validity owner. Do not turn a passing helper or projected expression into a whole real-model/GPU test claim.'),
 ('Fixture provenance and physical readiness remain separate','New fixtures compare cached native routes to full-recompute Transformers and include actual selected expert read counts.',['crates/intel/esp/tests/architecture_rows.rs','crates/intel/esp/tests/architecture_moe_real.rs','crates/intel/esp/tests/reference_architectures.py','crates/intel/esp/tests/mistral_library.rs','crates/intel/esp/tests/measure_architecture_gate.py'],'Source reviewed, no real model execution. Large physical tests are ignored, use explicit work roots and retain scoped device/reference dtype. Mistral/MoE row caches trust saved inputs-to-row association; exact cached provenance deserves an independent recheck before stronger pinned-artifact claims. Memory wrapper labels whole-GPU readings and sampled RSS correctly; it is not per-process VRAM proof.'),
 ('Release identity has one type while verification remains a native capability','ReleaseRefV1 is two opaque lookup identities and explicitly grants no authorization.',[l+'release.rs',l+'lib.rs','crates/system/luggage/Cargo.toml'],'Core consumers can name/compare releases but cannot call a public verification entry point. The manifest feature commentary and one library paragraph still say the browser can verify, contradicting the more precise admission that verification is private/native. Keep ownership of trust lookup distinct from a type-only core and transport carriage.'),
 ('Manifest and artifact authenticity have real distinct native checks','Updater verifies exact manifest bytes before parsing; download verifies optional BLAKE3 and the artifact minisign signature.',[l+'updater.rs',l+'signing.rs',l+'install.rs'],'Shared minisign helper is real reuse. Explicit unsigned-manifest opt-out changes guarantees. HTTP detached-signature requests do not inherit custom feed headers and build the URL by appending .sig to the full serialized URL; authenticated/query feeds need a bounded local HTTP case. No signing, download or live-feed execution ran.'),
 ('Restart staging creates a second weaker release authority','StagedRecord holds version, format, digest and target path but loses signed publisher evidence.',[l+'staging.rs',l+'install.rs'],'Q383 shows edited metadata self-certifies an edited payload after reload. install_staged does not bind that record back to the Update signature/version/format/target. Atomic Backend-like trust does not exist here; stage writes also omit file/directory sync, so durable means retained across normal close rather than power-loss proof.'),
 ('Cleanup and load wording exceed some filesystem outcomes','discard ignores every removal error while load only converts missing paths or invalid JSON into absence.',[l+'staging.rs'],'Q384 is an executed false cleanup success. Despite its unreadable-directory documentation, load propagates record I/O errors; bad JSON is a different branch. Write/rename staging replacement and partial record publication need failure cases before a crash-consistency claim.'),
 ('Installer handoff and replacement are not an applied-update receipt','Windows detaches or exits, Linux writes after moving the old AppImage, and macOS has separate direct/admin moves.',[l+'install.rs'],'Q385 identifies the ignored fallback launch failure. Document macOS rollback/backup lifetime, archive path/structure and cross-filesystem replacement review as high-effect platform followup; no installer is run and no broad data-loss exploit is asserted. The source itself calls staged-swap a planned divergence.'),
 ('Manifest authoring repeats grammar instead of using the parsed release contract','The CLI merges loose JSON maps and string arguments, writes bytes and separately instructs signing.',[l+'bin/luggage-manifest.rs',l+'release.rs',l+'signing.rs'],'Nonempty signature input, format/version/URL strings and existing signature-file presence do not prove a valid newly signed release. Artifact .sig text is copied while the manifest-signature fetch has a normalizer. Corrupt/unreadable prior JSON can be replaced without preserving the old platform map. Treat producer validation, signature normalization and atomic publication as bounded followups, not executed user export loss.'),
 ('Upstream ownership migrations are observed separately from the checkout','Fourteen diffs at origin/main00e44428 distinguish keyboard-produced text/paste, explicit presentation purpose and canonical Resource tags.',remote['reviewed_diffs'].keys(),'These are upstream-only reads; local HEAD7c718105 remains eight commits behind because fast-forward would overwrite the shared report. Other upstream resource/document/link changes remain unreviewed. Do not grant local current-source credit or report these changes installed.'),
 ('Main rereading reuses only exact previously reviewed code','Six changed files and seven new files were wholly read; fourteen other changed files combine verified prior blocks with every changed hunk.',changed,'Fourteen reconstructed old Git blobs match their old complete read SHA256, every equal block matches the new text and every changed block is ACKed. This is a composed current-source review, not a claim every old unchanged line was re-emitted. Luggage adds ten whole files. Full repository remains incomplete.'),
 ('Diagnostic and build lifetimes remain qualified evidence','A fresh offline CPU graph replaces missing old build/debug suppliers and runs24original plus3audit tests.',['support/receipts/authority-success-audit/main_refresh_staging_probe.rs'],'Initial retained-supplier setup stopped before compilation and is preserved. Scoped staging methods use an Update facade that cannot install. Cargo build-dir overrode intended target and generated a local cache; after terminal completion, marker/owner checks moved it to the approved shared build directory and deleted only the audit test exe/PDB. No isolated target retained; reusable supplier cache is owned by this active audit.')
]
entries=[{'id':f'A{1132+i}','domain':domain,'claim':claim,'implementations':list(paths),'assessment':assessment,
 'state':'main reread and Luggage source complete; full repository audit active','verification':NAME} for i,(domain,claim,paths,assessment) in enumerate(rows)]
authority['entries'].extend(entries)
delta_paths={r['path'] for r in delta['rows']}
completed=[{'prefix':p,'files':1,'chars':inv['files'][p]['chars'],'basis_commit':head,'verification':NAME,
 'method':'hash-proven prior full read plus every changed hunk' if p in delta_paths else 'whole current file emitted and acknowledged',
 'qualification':'No stale or partial prior read is transferred. Upstream-only diffs are separate and grant no checkout coverage.'} for p in fresh]
notes['completed_source_areas'].extend(completed)
notes['review_followups'].extend({'area':area,'detail':detail,'verification':NAME} for area,detail in [
 ('Unfinished release trust migration','Q383 is a high-effect restart authentication issue; retain publisher evidence and caller-owned trust without inventing policy. Platform replacement/rollback/archive validation and durable-stage migration can exceed an hour and remain documented.'),
 ('Bounded success-path fixes','Q382 finite/shape checks, Q384 cleanup errors and Q385 failed launch reporting are bounded candidates. No production edits in this read-only audit.'),
 ('Upstream integration and source','Shared audit-report edits prevent fast-forward. Preserve them and continue the remaining24upstream diffs through Git objects; do not stash shared WIP or claim current origin is installed. Then integrate safely when the report collision is resolved.'),
 ('Ownership migration ledger','Retain actual artifact_source/tensor-name/EOS consolidation alongside unresolved numerical acceptance, staged-release trust and installer completion authorities. Do not treat file splits or a shared crate name as completed ownership migration.')])
next_read={'path':'crates/system/registry/Cargo.toml','start':0,'reason':'Continue the full inventory with Registry diagnostics/state ownership; also finish the24remaining upstream diffs read-only while main fast-forward is report-blocked.'}
notes['current_area']='All20changed main sources and7new files reviewed; ten Luggage files read; four findings recorded'
notes['next_area']=next_read['reason']
notes['checkpoint']={'classification':'progress','classification_of_previous_turn':'progress','goal_status':'active; full repository source incomplete',
 'coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),
 'new_receipts':[NAME,'main_refresh_staging_probe_v2_receipt.json','upstream_after_inference_review_receipt.json','staging_build_relocation_receipt.json'],
 'audit_authored_production_edits_this_turn':False,'next_read':next_read,
 'pending_verification':'No owned process live. Full repository,24upstream diffs, native Luggage pipeline/signing/installer and real architecture/GPU fixtures remain unexecuted or unread. Earlier qualified findings remain historical; old retained build/debug suppliers are absent. Main remains8behind origin due shared-report overlap.'}
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True);assert diff.returncode==0
with (HERE/'main_refresh_luggage_diff_check.txt').open('x',encoding='utf-8') as f:f.write(diff.stdout+diff.stderr)
addition=f'''

### Main reread, inference ownership, and completed Luggage source, 2026-10-17

At checkout HEAD {head}, all20externally changed sources and7new inference files /772,041current characters are now reviewed. Six changed files plus every new file were emitted wholly; fourteen other changed files use a composed review: exact old Git bytes reproduce an earlier complete read hash, each unchanged block is equal to current bytes, and every changed hunk is emitted and ACKed. These fourteen files contain542,964previously read unchanged characters plus49,864new characters. The receipt verifies reconstruction and complete hunk coverage rather than calling a diff scan a whole new read. Luggage then adds10whole files /106,957characters. Coverage is{stats['complete_files']:,}/{stats['total_files']:,}files /{stats['read_chars']:,}/{stats['total_chars']:,}characters, zero stale/pending credit. Full review remains active and incomplete.

Q382: the new architecture reference max-error subcheck reports0for mixed/all-NaN actual logits because f32::max suppresses NaN differences. Exact-match and finite-mismatch controls pass. This is an exact expression projection; no complete architecture/model/GPU test is claimed to pass invalid output. Existing Q348 rank, Q349 Unicode streaming, Q347 sampling and Q217 actor-disconnect findings are not fixed by these ownership changes and were not rerun.

Q383: unchanged production StagedUpdate load/take_verified accepts substituted unsigned bytes after both staged-artifact and the digest in neighbouring staged.json are changed. Intact control succeeds and artifact-only tampering is refused. The record loses publisher signature/key/manifest evidence across restart and becomes its own integrity authority. Requires write access to both stage files; no remote exploit, privilege escalation or initial genuinely signed download is claimed. install_staged's unchecked native handoff is source reviewed only. Retaining authenticated publisher evidence and defining old-stage migration is a high-effect design issue documented under the hour rule, not repaired here.

Q384: unchanged discard returnsOkafter both remove_file calls fail against audit-owned empty directories; both paths remain. Q385: source-only Windows MSI handoff ignores fallback msiexec spawn failure and unconditionally exits0after PowerShell spawn failure, even when no launcher starts. No OS installer/process-exit experiment ran. Broader rollback, archive structure/path validation, cross-filesystem install and staged durability work remain documented, without inventing destruction or publisher policy.

A1132-A1148 track actual consolidation: DecoderConfig drives declared math, checkpoint/loader share tensor-name grammar, ResolvedInferenceRow::artifact_source now owns both core-layer and expert identities, Distillery supplies only store access, and template EOS reaches both loops through one provider list. Explicit F32/native/inference-only guards stay distinct from training/browser support. Resident/streamed/staged paths share kernels but their resource/placement receipts are different. Unfinished authority includes independent numerical validity checks, editable restart staging, installer handoff versus completion, CLI loose manifest grammar/signature normalization, and authenticated/query HTTP signature retrieval. Core ReleaseRefV1 remains naming-only; prose still claiming browser verification exceeds its private/native verifier API.

Fresh offline CPU gate passes27tests:24unchanged original config/portable-provider/release-core tests plus3audit tests. Staging production methods are included unchanged before cfg(test), over an audit Update facade whose install panics and is never invoked. First retained-supplier setup failed before compilation because the old build/debug rlibs had disappeared; terminal failure is preserved. The fresh graph is small and offline, not a full Luggage/ESP native graph. A local Cargo build-dir setting redirected its generated cache despite CARGO_TARGET_DIR. After terminal completion, Cargo marker and live-owner checks moved only generated output to C:/t/cargo-targets/mere/build and removed the audit test executable/PDB. No local/isolated target remains; the187,458,309byte reusable shared supplier cache is owned by this active audit for later scoped gates. Small final staging fixtures remain as evidence. Future gates must explicitly set build.build-dir too.

Fetch finds origin/main {upstream}, eight commits ahead. git merge --ff-only refuses because it would overwrite the shared scenograph editor/audit report; no stash,revert,commit or production changes were made. Fourteen upstream-only diffs were read: keyboard text/paste identity, explicit producer presentation purpose, canonical Resource tags and their tests, toolchain and published welding pin. This grants no checkout read credit. Twenty-four other upstream diffs, including document/fetch/link ownership work, remain queued; do not call origin installed or completely reviewed.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/{NAME}), [current composed reread](../../../support/receipts/authority-success-audit/main_delta_plan.json), [CPU gate](../../../support/receipts/authority-success-audit/main_refresh_staging_probe_v2_receipt.json), [upstream status](../../../support/receipts/authority-success-audit/upstream_after_inference_review_receipt.json), and [generated-output relocation](../../../support/receipts/authority-success-audit/staging_build_relocation_receipt.json). Previous report bytes remain an exact prefix. Protected source, runtime, auxiliary inputs, unrelated WIP, prior evidence and Git index match. Diff check passes; receipt evidence is below100MB. Next: Registry source and remaining upstream diffs, preserving current report/WIP.
'''
doc.write_bytes(prefix+addition.encode('utf-8'))
for n,v in [('review_notes.json',notes),('authority_map.json',authority)]: (HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
evidence=['record_main_refresh_luggage_review.py','main_refresh_luggage_diff_check.txt','read_main_delta.py','main_delta_plan.json',
 'main_refresh_staging_probe.rs','main_refresh_staging_production.rs','main_refresh_comparison_projection.rs',
 'run_main_refresh_staging_probe.py','run_main_refresh_staging_probe_v2.py','main_refresh_staging_initial_setup_failure.json',
 'main_refresh_staging_probe_v2_receipt.json','main_refresh_staging_probe_v2_output.log','cpu-staging-runtime/Cargo.toml','cpu-staging-runtime/Cargo.lock',
 'relocate_staging_build.ps1','staging_build_relocation_receipt.json','upstream_after_inference_review_receipt.json',*probe['fixture_files']]
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':head,'upstream_commit':upstream,
 'kind':'twenty changed plus seven new inference files reviewed; ten whole Luggage source files; repository incomplete',
 'prior_receipt':{'path':'stickleback_completion_review_receipt.json','sha256':sha(HERE/'stickleback_completion_review_receipt.json')},
 'coverage':stats,'completed_source_areas':completed,'whole_current_files':[p for p in fresh if p not in delta_paths],
 'composed_review_files':sorted(delta_paths),'composed_review_qualification':delta['qualification'],
 'source_sha256':{p:sha(ROOT/p) for p in fresh},'preserved_production_and_test_hashes':guards,
 'resolved_previous_stale_files':list(old['superseded_reviewed_source_hashes_pending_reread']),
 'auxiliary_read_sha256':aux,'runtime_input_sha256':runtime,'shared_working_file_sha256':shared,
 'new_findings':[f['id'] for f in findings],'authority_entries':[e['id'] for e in entries],
 'audit_authored_production_edits_this_pass':False,'diagnostics':{'passed_tests':27,'original_tests':24,'audit_tests':3,'qualification':probe['qualification']},
 'prior_suppliers':'Old build/debug dependency family absent at attempted reuse; historical receipts unchanged, not freshly revalidated or rerun.',
 'pending_verification':notes['checkpoint']['pending_verification'],'next_read':next_read,
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},
 'evidence_sha256':{p:sha(HERE/p) for p in evidence},'git_index_sha256':index_hash,
 'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},'retained_isolation':[],
 'shared_generated_cache':{'path':relocation['destination'],'bytes_after_probe_binary_removal':relocation['cache_bytes'],'owner':'active full-source authority/success audit','reason':'Fresh reusable small CPU supplier graph for later scoped diagnostics; approved stable shared Cargo target, not isolated.'}}
save(NAME,receipt)
assert sha(index)==index_hash and doc.read_bytes().startswith(prefix)
assert all(sha(ROOT/p)==h for p,h in {**guards,**receipt['source_sha256'],**aux,**runtime}.items())
assert all(sha(HERE/p)==h for p,h in receipt['evidence_sha256'].items())
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
assert subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True).returncode==0
print(json.dumps({'receipt':NAME,'coverage':stats,'new_findings':receipt['new_findings'],'authority_entries':receipt['authority_entries'],
 'original_tests':24,'audit_tests':3,'protected_sources':len({**guards,**receipt['source_sha256']}),'retained_receipt_bytes':budget,
 'upstream_relation':'0ahead/8behind;14of38diffs read','next_read':next_read},indent=2))
