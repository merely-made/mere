"""One-shot primary browser-probe and Eidetic model/schema/storage checkpoint."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
NAME='browser_eidetic_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def save(n,v):(HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
assert not (HERE/NAME).exists()
old=load('admission_probe_review_receipt.json')
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()==old['basis_commit']
preserved={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
assert all(sha(ROOT/n)==s for n,s in preserved.items())
assert load('pending_read.json')==[] and not (HERE/'pending_transition.json').exists()
whole=['ports/distillery/probe/src/lib.rs','ports/distillery/probe/web/index.html',
 'ports/distillery/probe/web/probe.js','ports/distillery/probe/web/styles.css','ports/distillery/probe/web/worker.js',
 'crates/eidetic/eidetic-core/Cargo.toml','crates/eidetic/eidetic-core/src/lib.rs',
 'crates/eidetic/eidetic-core/src/models/adapter.rs','crates/eidetic/eidetic-core/src/models/inference.rs',
 'crates/eidetic/eidetic-core/src/models/library.rs','crates/eidetic/eidetic-core/src/models/mod.rs',
 'crates/eidetic/eidetic-core/src/models/recall.rs','crates/eidetic/eidetic-core/src/models/training.rs',
 'crates/eidetic/eidetic-core/src/codicil.rs','crates/eidetic/eidetic-core/src/clip_profile.rs',
 'crates/eidetic/eidetic-core/src/content_store.rs','crates/eidetic/eidetic-core/src/image_content.rs',
 'crates/eidetic/eidetic-core/src/pack.rs','crates/eidetic/eidetic-core/src/bundle.rs',
 'crates/eidetic/eidetic-core/src/manifest/tests.rs','crates/eidetic/eidetic-core/src/schema_def.rs',
 'crates/eidetic/eidetic-core/src/schema_def/validators.rs','crates/eidetic/eidetic-core/src/schema_def/tests.rs']
inv,cov=load('source_inventory.json')['files'],load('read_coverage.json')
assert len(whole)==23 and sum(inv[n]['chars'] for n in whole)==254394
assert all(cov[n]['sha256']==inv[n]['sha256']==sha(ROOT/n) and cov[n]['through']==inv[n]['chars'] for n in whole)
distillery=[n for n in inv if n.startswith('ports/distillery/')]
assert all(cov.get(n,{}).get('sha256')==inv[n]['sha256']==sha(ROOT/n) and cov[n]['through']==inv[n]['chars'] for n in distillery)
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==1351 and stats['read_chars']==17948167 and not stats['changed_since_inventory'] and not stats['pending_chunks']
browser=load('browser_probe_flow_receipt.json');assert browser['checks']==6 and browser['exit']==0
assert sha(ROOT/browser['source'])==browser['source_sha256']
probes={n:load(n) for n in ['legacy_corpus_probe_receipt.json','content_cache_probe_receipt.json',
 'pack_signature_order_probe_receipt.json','manifest_cache_probe_receipt.json','builtin_schema_probe_receipt.json']}
for n,p in probes.items():
    assert p['test_exit']==0 and p['generated_output_removed']
    assert all(sha(ROOT/k)==s for k,s in p['source_sha256'].items())
assert sum(p['tests'] for p in probes.values())==16
for stem in ['legacy-corpus','content-cache','pack-signature-order','manifest-cache','builtin-schema']:
    assert not list(Path('C:/t/cargo-targets/mere').glob('authority-success-'+stem+'-probe.*'))
assert load('source_extension_recheck.json')['paths']==[]
notes,authority=load('review_notes.json'),load('authority_map.json')
assert notes['findings'][-1]['id']=='Q225' and authority['entries'][-1]['id']=='A515'
findings=[
 {'id':'Q226','title':'Manual browser-probe termination leaves its UI submission unresolved and controls disabled',
  'source':'ports/distillery/probe/web/probe.js:runWorker, terminateActive and submit handler',
  'status':'actual page orchestration reproduced in controlled native Node VM; documented, not repaired',
  'evidence':'The UI submission disables run controls and awaits runSuite/runWorker. terminateActive returns true, calls terminate and clears global worker state, but never calls the runWorker promise resolver/rejector. In the bounded VM observation the fake worker is terminated and page state is terminated, while submission remains pending and run controls stay disabled. A worker-error control settles the same flow and reenables controls.',
  'scope':'Complete actual page JavaScript, synthetic DOM/fetch/worker messages. The pending observation is30ms plus the source proof that manual termination calls neither finish nor a timeout. No real browser, worker, GPU, model or storage was exercised.',
  'diagnostic':'browser_probe_flow_receipt.json',
  'repair_triage':{'disposition':'documented in active read-only audit','size':'bounded cancellation settlement',
   'reason':'Route manual termination through the same owned finish/rejection boundary as normal result/error and state-driven cancellation; clear the active owner once and allow finally to reenable controls. Do not create a second independent cancellation state machine.'}},
 {'id':'Q227','title':'Embedding row success ignores reported within-worker and across-worker repeatability failures',
  'source':'ports/distillery/probe/web/probe.js:runConfiguredRowUngated and runDecoderUngated',
  'status':'actual page receipt gate counterexample reproduced in controlled native Node VM; documented, not repaired',
  'evidence':'Synthetic worker-contract reports have valid dimensions/norm/reference/integrity but false cold/warm repeat_outputs_match and false matches_prior_worker. The actual embedding row concludes row_passed=true and output_repeated_across_workers=false. The decoder counterpart explicitly gates on these repeatability facts. Integrity-failure and ordinary-consistent controls show the embedding gate is selective, not universally passing.',
  'scope':'Native execution of complete page code with controlled worker reports. No claim that an actual GPU produced nondeterministic values or a real model receipt was generated.',
  'diagnostic':'browser_probe_flow_receipt.json',
  'repair_triage':{'disposition':'documented in active read-only audit','size':'bounded shared row predicate',
   'reason':'Make row success require the repeatability facts the row purports to qualify, and share common cold/warm checks between embedding and decoder while keeping workload-specific numeric/stream/cancel checks separate.'}},
 {'id':'Q228','title':'Browser matrix can claim pinned model size and success despite contradictory observed component sizes',
  'source':'ports/distillery/probe/web/probe.js:runConfiguredRowUngated/runMatrix; probe/src/lib.rs:ProbeConfig/execute',
  'status':'actual page matrix gate accepts contradictory controlled worker reports; documented, not repaired',
  'evidence':'The page accepts cold/warm reports whose observed weights size is1 byte while the configured artifact advertises90000000 bytes. all_configured_rows_passed is true and largest_successful_weight_bytes is90000000. The gate never compares component_sizes with expected_artifacts, and ProbeConfig does not carry the pinned artifact sizes/SHA256 values into acquisition. Cold/warm hashes prove storage round-trip identity, not agreement with the configured external pin.',
  'scope':'Actual full page source in native Node VM, synthetic worker-contract messages only. A one-byte real model was not loaded and no GPU success with a wrong model is claimed. This proves the receipt predicate/size conclusion gap and the missing pin comparison in reviewed source.',
  'diagnostic':'browser_probe_flow_receipt.json',
  'repair_triage':{'disposition':'document artifact identity repair rather than widen audit beyond one hour','size':'shared pin and observed-artifact contract',
   'reason':'Carry the producer-owned artifact pin to acquisition/validation, compare actual source bytes and sizes before claiming that row, and derive measured size conclusions from observed evidence. Reuse the same pin semantics across fetch/reference/browser readers; cold-to-warm equality alone cannot establish publisher/model identity.'}},
 {'id':'Q229','title':'Legacy training-corpus reader bypasses the current corpus invariants',
  'source':'crates/eidetic/eidetic-core/src/models/training.rs:load_training_corpus and LegacyTrainingCorpus',
  'status':'public native legacy/current reader divergence reproduced; documented, not repaired',
  'evidence':'The v1 path deserializes a legacy shape and directly maps fields into TrainingCorpus without validate. Public load returns empty, overlapping, duplicated and reverse-ordered partitions that fail the shared current validator. Current v2 serialization refuses all equivalent cases; valid v1/v2 records agree and validate.',
  'scope':'Synthetic memory-backed artifacts and retained exact Cargo-built libraries; reviewed sources unchanged through merge. Distillery calls corpus.validate before loading cases, so actual training/evaluation leakage is not claimed.',
  'diagnostic':'legacy_corpus_probe_receipt.json',
  'repair_triage':{'disposition':'documented in active read-only audit','size':'bounded legacy conversion validation',
   'reason':'Run the shared corpus validator after legacy conversion, preserving valid historical vocabulary and refusing malformed legacy partitions at the same artifact boundary. Keep the trainer recheck as defense at its consuming boundary.'}},
 {'id':'Q230','title':'Successful URL-cache replacement with no MIME retains the preceding response content type',
  'source':'crates/eidetic/eidetic-core/src/content_store.rs:save_content/load_content',
  'status':'public native cache replacement counterexample reproduced; documented, not repaired',
  'evidence':'After saving HTML, a successful replacement with new body bytes and content_type=None returns the new bytes plus old text/html. save_content overwrites body but only writes MIME for Some and never deletes the old MIME key. Fresh None and explicit text/plain replacement controls behave correctly. This contradicts the documented None round-trip on replacement.',
  'scope':'Synthetic URL/body/MIME in MemoryBackend only. This durable-cache defect is distinct from Q192 in PageLoad request observation; no browser rendering, live network, credential or installed cache was exercised.',
  'diagnostic':'content_cache_probe_receipt.json',
  'repair_triage':{'disposition':'documented in active read-only audit','size':'bounded body/metadata replacement',
   'reason':'Replace body and MIME as one shared backend batch, deleting the MIME key when the new response has none. Muniment apply already supplies the common batch seam; do not retain metadata from a different body or add another cache layer.'}},
 {'id':'Q231','title':'Pack verification verdict changes when the same valid and malformed signatures are reordered',
  'source':'crates/eidetic/eidetic-core/src/pack.rs:verify_pack',
  'status':'native public signature-order counterexample reproduced; documented, not repaired',
  'evidence':'The same signature multiset is Trusted with a valid binding first and Broken with a malformed binding first. verify_pack returns on the first personae binding despite its documentation describing whether some binding verifies. Single valid, malformed, unsigned and tampered-manifest controls retain their expected distinctions.',
  'scope':'Synthetic Ed25519 key/manifest and actual sign_pack/verify_pack. No signature forgery, installed pack, executable payload, grant or visible-review bypass is claimed. The replacement mixed-signature policy is not chosen by this audit.',
  'diagnostic':'pack_signature_order_probe_receipt.json',
  'repair_triage':{'disposition':'document mixed-signature policy decision','size':'trust verdict semantics and compatibility',
   'reason':'Choose and document whether one valid author binding suffices or any claimed invalid author binding rejects the envelope, then evaluate the complete set consistently. Keep signature verification distinct from local installation review and grants.'}},
 {'id':'Q232','title':'Manifest resolution writes a canonical cache it cannot reuse for remote-only or differently keyed sources',
  'source':'crates/eidetic/eidetic-core/src/manifest.rs:resolve_blob; manifest/tests.rs cache test',
  'status':'native actual acquisition/cache/reopen counterexample reproduced; documented, not repaired',
  'evidence':'A synthetic fetcher supplies valid bytes through actual resolve_blob, which caches them under blob:<hash>. Taking that fetcher offline makes the next resolution fail despite those valid cached bytes remaining present, for remote-only and nonmatching-Local manifests. Adding the canonical Local source makes the warm read work without another fetch. Corrupt declared cache bytes are still rejected.',
  'scope':'Actual resolver and MemoryBackend with an in-process canned fetcher, no network. Normal save_typed manifests already declare the canonical Local key and are the working control. The original cache test only checks that a cache key was written, not that it can be read on a subsequent offline resolve.',
  'diagnostic':'manifest_cache_probe_receipt.json',
  'repair_triage':{'disposition':'documented in active read-only audit','size':'bounded shared cache lookup/source normalization',
   'reason':'Make canonical cached identity usable on later resolution or explicitly require/adopt a canonical Local source when populating that cache. Preserve hash verification and the sealed-blob boundary; a successful cache write is not proof of a reusable cache.'}},
 {'id':'Q233','title':'Built-in adapter and opaque payload schemas do not conform to the shared declared validator',
  'source':'crates/eidetic/eidetic-core/src/models/{adapter.rs,mod.rs}; schema_def.rs:MereNativeFieldSpec/validate_payload',
  'status':'native exact built-in schema versus real typed binding counterexamples reproduced; documented, not repaired',
  'evidence':'Each extracted canonical definition hashes to the runtime public schema reference. A valid ModelAdapterManifest cannot validate because its mere-native body uses unsupported number/integer and nullable-union type forms; the observed failure is unknown variant number. OpaqueBlob serializes raw bytes, but its definition requires a JSON object with a bytes-string field and rejects those bytes. Current TrainingCorpus validates against its own exact definition. Meta bootstrap leaves the adapter definition absent, and the documented tolerant unknown-schema path skips rather than proving validity.',
  'scope':'Public actual typed bindings and common validator, exact schema constants, synthetic payloads and a fresh memory-store control. Retained Cargo-built libraries; reviewed sources unchanged. Actual ESP loader domain checks are separate, so failed real model loading, GPU execution or installed artifact migration is not claimed.',
  'diagnostic':'builtin_schema_probe_receipt.json',
  'repair_triage':{'disposition':'document schema compatibility repair; do not widen audit beyond one hour','size':'shared schema/encoding authority and legacy identities',
   'reason':'Make declared schema format/encoding and Rust binding agree through one supported contract. Preserve old immutable schema references via explicit revisions/legacy readers or deliberate validator compatibility. Merely relabeling a format or patching canonical constants without migration can invalidate retained artifacts or silently weaken validation.'}},
]
for f in findings:f['verification']=NAME
notes['findings'].extend(findings)
notes['candidates'].append({'title':'Raw image namespace lacks the typed resolver integrity check',
 'source':'crates/eidetic/eidetic-core/src/image_content.rs:load_image_bytes; crates/system/pandect/src/image_store.rs:load_image',
 'note':'Image bytes are stored at a content-hash key but load_image_bytes returns the raw backend value without recomputing the digest; Pandect forwards it. This differs from manifest resolution and remains a separate content-addressed namespace. Trace the caller corruption/decoding/trust contract before asserting a current false integrity success. No corrupted-image load probe or rendering trial ran.'})
rows=[
 ('Primary browser model execution','The probe composes shared IndexedDB, Eidetic and ESP instead of defining another inference engine.', ['worker::execute','ModelLibrary::save_model_with_components/resolve_components','ESP Bert/DecoderProvider'], 'Cold acquisition, stored resolution and actual model execution use the shared owners. The native page diagnostic injects worker reports and therefore is not a model/GPU/browser acceptance run.'),
 ('Completion versus qualified success','Worker finished means its report is complete, not that every conclusion passed.', ['post_state finished','ExecutionReceipt fields','page row predicate'], 'This distinction is real, but the page must consume the report facts correctly; Q227/Q228 show fields the embedding success predicate ignores.'),
 ('Two cancellation implementations','State-driven cancellation settles through finish, while the manual path bypasses it.', ['runWorker cancelAtState','terminateActive','submit finally'], 'Q226 demonstrates the unresolved manual UI submission against a working error-settlement control. One owned settlement boundary should serve both.'),
 ('Duplicated workload receipt gates','Embedding and decoder separately define cold/warm success.', ['runConfiguredRowUngated','runDecoderUngated'], 'The decoder checks repeats and prior-worker identity; embedding records them but does not gate them. Q227 is concrete divergence in copied qualification logic, not a second inference engine.'),
 ('External pin versus local round trip','Configured artifact metadata is not the worker acquisition authority.', ['resolveRow','ProbeConfig','expected_artifacts','runMatrix largest_successful_weight_bytes'], 'Q228 shows a matrix can label expected bytes as successful while worker sizes disagree. Source pin parsers are copied across JS/Python/PowerShell, and pin sizes/SHA256 do not reach the runtime worker config.'),
 ('Warm stored identity','Warm runs require hashes in execute and use NoFetcher to resolve stored components.', ['RunMode::Warm','source_hashes/resolved_hashes','ModelLibrary resolve'], 'Although the early config validator only requires manifest_id, execute explicitly rejects absent expected_hashes. A successful warm report is not based on a vacuous absent-hash comparison; that candidate is ruled out.'),
 ('Measurement limits stated in source','Copy ladder and page adapter samples are deliberately bounded evidence.', ['copy_ladder notes','browserAdapter note','memorySnapshot unknown','gpu_memory_release_measured false'], 'Structural eager-copy accounting is not peak memory telemetry, and the main-page adapter is not asserted to identify Burn worker hardware. Reading these paths does not qualify current browser/GPU behavior.'),
 ('Real shared storage floor','Eidetic Store is Muniment Backend reexported, with compatibility names rather than a second trait.', ['lib.rs Store alias','MemoryBackend reuse','engram compatibility module'], 'The old floor duplication is actually removed. Arbitrary SaveBlob remains mutable by contract; the floor alone cannot enforce the higher-level R0 envelope/history claims.'),
 ('Composite inventory consolidation not adopted','The generic Bundle API is not the common model/pack inventory in this checkout.', ['bundle.rs','models::ModelManifest/ModelLibrary','PackManifest parts','bundle_consumer_scan.log'], 'The crates/ports Rust name scan finds Eidetic Bundle implementation/tests/reexports and unrelated Bundle names, with no other matching Eidetic consumer. Reviewed model and pack paths use separate shapes while sharing leaf storage. External published consumers are not surveyed, so this is an unadopted in-repo API, not proof it is safe to delete.'),
 ('Payload identity and metadata envelopes','Codicil and BlobManifest both carry classification around payload-hash identity.', ['Codicil::id/verify_integrity','BlobManifest','TypedPayload save/load'], 'Codicil integrity covers payload bytes and valid time bounds, not a complete metadata-envelope identity. Q221 remains the concrete schema/metadata alias finding. Do not infer all classification or authorization from a bare payload hash.'),
 ('Legacy corpus validation drift','Current corpus serialization/deserialization shares its validator, while the legacy conversion skips it.', ['TrainingCorpus TypedPayload implementation','LegacyTrainingCorpus','load_training_corpus'], 'Q229 is reproduced on four malformed partition classes. The reviewed TrainerResource explicitly revalidates before loading cases, which prevents overclaiming actual training leakage.'),
 ('Durable URL-cache metadata','The cache stores body and MIME at separate keys.', ['content_store::save_content/load_content','PageLoad Q192 comparison'], 'Q230 leaves old MIME on successful None replacement. This is a distinct owner/implementation defect from in-memory request metadata; fixing one does not establish the other.'),
 ('Signature verdict and installation authority','Pack Trusted explicitly means signature-only evidence.', ['sign_pack/verify_pack','PackVerdict','requested_scopes'], 'Q231 exposes first-binding order dependence. No grant is minted here and the source expressly retains local review; do not report a signature forgery or install bypass.'),
 ('Cache existence versus cache consumption','The resolver and its cache test use different done-conditions.', ['resolve_blob cache write','declared Local lookup','resolve_falls_through_local_then_iroh_then_https'], 'Q232 demonstrates successful cache population without usable offline reopen. Canonical Local manifests are a positive control and actual byte integrity checks remain real.'),
 ('Built-in schema authority drift','Private canonical schema declarations disagree with the central native schema grammar/encoding.', ['ModelAdapterManifest schema payload','OpaqueBlob raw binding','MereNativeFieldSpec'], 'Q233 checks exact declared hashes and actual typed payloads. TrainingCorpus is a conforming control. Private runtime validators can keep model paths working while portable schema validation fails, so successful local tests do not prove shared schema authority.'),
 ('Tolerant schema reads are skips','Unknown-schema success does not prove payload conformance.', ['validate_against_schema missing definition branch','bootstrap_meta_schema','bootstrap_enables_strict_schema_resolution test'], 'This tolerance is explicitly documented and tested, rather than an undisclosed implementation fallback. Bootstrap seeds only meta schema; its test verifies that one definition, not universal availability of built-in schemas.'),
 ('Format-specific validation remains bounded','Schema dispatch has three closed enum formats with explicit capability limits.', ['MereNativeValidator','JsonSchemaValidator feature gate','JsonLdValidator'], 'Unavailable JSON Schema validation returns an error. JSON-LD checks nominal types and required non-null properties without full SHACL; the overview still calls it parse-only. The trait/enum is extendable by code changes, not a demonstrated runtime validator registry.'),
]
entries=[{'id':f'A{516+i:03d}','domain':d,'claim':c,'implementations':impl,'assessment':a,
 'state':'current source review; executed scope named in assessment','verification':NAME} for i,(d,c,impl,a) in enumerate(rows)]
authority['entries'].extend(entries)
terminal=load('flora_gate_terminal.json') if (HERE/'flora_gate_terminal.json').exists() else None
if terminal:
    pending=None;gate_note=f"The postmerge original FLoRA gate is terminal: exit{terminal['exit']}, {terminal['passed']} passed and {terminal['failed']} failed. Its exact scope is recorded in flora_gate_terminal.json."
else:
    pending={**old['pending_verification'],'state':'Same live session32507 was polled again; network fetch is complete and the current dependency graph is compiling, including Eidetic/ESP/Pandect and Gemot. No terminal result was observed. Poll the same handle; do not start another gate on an observation timeout.'}
    gate_note='The original postmerge FLoRA gate remains live at session32507. It has finished fetching pinned dependencies and is compiling the current graph; no terminal result was observed. Continue polling that same handle rather than starting another retry.'
notes['current_area']='Completed primary Distillery probe; Eidetic models, corpus, raw cache, pack/bundle, schema declarations and validators'
notes['next_area']='Eidetic sealing/retention and remaining browsing/import source, followed by other unread repository areas'
notes['checkpoint']={'classification':'progress','classification_of_previous_turn':'progress','goal_status':'active; full-source objective incomplete',
 'coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),
 'new_receipts':[NAME],'production_edits_this_turn':False,
 'next_read':{'path':'crates/eidetic/eidetic-core/src/seal.rs','start':0,'reason':'Follow envelope/cache identity into sealing and retention; browsing/import and other repository source remain unread.'},
 'diagnostic_outcomes':'6 complete-page Node VM checks and16 native Rust tests pass. Current probe source/model/schema/corpus/cache/pack behaviors have bounded receipts, with no real model/GPU/browser run. All diagnostic native outputs removed.',
 'pending_verification':pending,'coverage_note':'23 additional whole current files254394 chars; all inventoried ports/distillery source now complete; no stale hashes or pending chunks.'}
doc=ROOT/old['document']['path'];original=doc.read_bytes();assert hashlib.sha256(original).hexdigest()==old['document']['sha256']
addition=f'''

### Primary browser probe and Eidetic authority review, 2026-10-17

This pass reads23 additional whole files254394 characters. Coverage is {stats['complete_files']:,} of {stats['total_files']:,} files and {stats['read_chars']:,} of {stats['total_chars']:,} current characters, without stale acknowledged hashes or pending chunks. All inventoried source beneath ports/distillery is now read. That is not the whole Mere objective: Eidetic sealing/retention/browsing/import and other product source remain outstanding. A separate path inventory found none of the additional common code-extension candidates checked; this does not replace the source denominator or add scan results as reading coverage.

Q226-Q228 are reproduced by evaluating the complete actual browser page JavaScript in a native Node VM with controlled DOM/fetch/Worker messages. Manual termination returns true and kills the synthetic worker but never settles the pending UI submission, leaving controls disabled. The worker-error control settles and reenables them. Embedding row success ignores false within-worker and across-worker repeatability fields, whereas decoder success explicitly includes them; an integrity-failure control rejects and an ordinary row passes. Finally, a controlled worker report with1 observed weight byte can produce a successful matrix conclusion advertising90000000 configured bytes as its largest successful model. No real browser, worker, model, GPU or network was exercised, and a one-byte real model was not loaded. These are actual consumer-orchestration/predicate contradictions under controlled worker contracts. ProbeConfig does not carry the external artifact sizes/SHA256 pins into acquisition. Cold-to-warm content equality is real stored identity, not evidence of agreement with the advertised model pin.

Q229: the v1 training corpus reader converts legacy fields without the current validator. Public native loading accepts empty, overlapping, duplicated and reverse-ordered partitions that current v2 serialization rejects. Valid v1/v2 controls agree. The real Distillery trainer independently calls corpus.validate before loading cases, so this does not claim an actual training/evaluation leak. Apply the same domain validation after legacy conversion rather than maintaining a weaker compatibility lane.

Q230: successful durable URL-cache replacement with content_type=None leaves the preceding response's MIME key intact. Native public save/load returns the new bytes plus old text/html; fresh-None and explicit-type replacement controls work. This is distinct from Q192's PageLoad request-observation defect. Replace body/MIME through the existing shared batch seam and delete absent replacement metadata. No live fetch, rendering or installed cache was touched.

Q231: pack verification returns on the first personae binding. A valid-plus-malformed signature multiset is Trusted or Broken solely by order, contrary to the documented existential verification description. Single valid/malformed/unsigned and tampered-manifest controls still behave correctly. All signing is with a synthetic fixture key. This is a verdict-contract problem, not a signature forgery, grant mint or installation-review bypass. Document the mixed-signature policy before changing it; the audit does not select whether one valid binding suffices or any invalid claim rejects the envelope.

Q232: actual manifest resolution can write valid canonical cache bytes yet fail its next offline resolve. This occurs for a remote-only source list or a noncanonical declared Local key, because the resolver never consults the canonical key it just populated. A canonical Local source makes the warm read succeed without another fetch, and corrupt declared cache bytes still fail integrity. The original cache test establishes key existence, not cache reuse. The native diagnostic uses a synthetic in-process fetcher, not a real network. Normal save_typed manifests already declare the canonical Local key and remain a working control.

Q233: built-in portable schema declarations have diverged from the shared schema validator and typed bindings. Exact canonical schema bytes are checked against runtime public references. A domain-valid ModelAdapterManifest is rejected because its declared mere-native body contains unsupported number/integer and nullable-union field forms; the observed first error is unknown variant number. OpaqueBlob produces raw bytes while its definition requires a JSON object carrying a bytes-string field, so its actual payload also fails. Current TrainingCorpus validates against its own definition. Meta bootstrap only seeds the meta-schema; a missing adapter definition still takes the explicitly documented tolerant skip path and can return Ok without proving validity. This is a schema/encoding compatibility repair with retained identities, not a safe in-place canonical-constant rewrite. No failed real model load or installed migration is claimed: ESP's local adapter domain checks are a separate current path.

Authority entries A516-A532 distinguish actual sharing from consolidation prose. The primary browser worker really uses IndexedDB/Muniment, Eidetic ModelLibrary and ESP providers. Its finished state means the report exists; the page owns qualification and must consume its facts correctly. Structural copy ladders and page adapter/memory samples explicitly disclaim peak GPU-memory and worker-hardware identity. Warm execute explicitly requires expected hashes despite the earlier shallow config check; a vacuous missing-hash success was ruled out.

Eidetic Store is actually a Muniment Backend reexport, and old Engram vocabulary is a compatibility alias rather than another engine. Conversely, the generic Bundle inventory remains unadopted by the reviewed model and pack paths. A crates/ports Rust name scan finds its implementation, self-tests and reexports plus unrelated Bundle names, with no other matching Eidetic consumer. ModelLibrary resolves its own config/weights/tokenizer shape, and PackManifest has another parts inventory while sharing the same leaf store. This is an in-repo consolidation gap; external published consumers were not surveyed, so removal is not justified. Codicil and BlobManifest wrap payload-hash identity with classification metadata; their higher-level temporal/authorization claims cannot be inferred from the mutable floor or bare content hash alone. Raw image loads are another namespace without the typed resolver's hash check; the consuming corruption/decoding contract remains a candidate to trace.

The central schema dispatcher has three code-defined formats, not a demonstrated runtime registry. Optional JSON Schema validation refuses when unavailable. JSON-LD now checks nominal types and required non-null properties while correctly stopping short of SHACL/vocabulary validation; its older overview still calls it parse-only. Unknown-schema tolerance is explicit and tested, so its Ok result must not be reported as conformance. Private runtime validators can succeed while portable schema definitions fail, which is the concrete shared-authority drift in Q233.

All6 Node checks and16 native Rust tests passed. The Rust diagnostics use exact retained premerge Cargo-built libraries with unchanged reviewed source; their scope is recorded per receipt, and the current postmerge gate remains separate. All diagnostic executables/PDB/import outputs were removed. Shared WIP and the canonical document prefix remain byte-preserved. No production/original-test edits or isolated target, Cargo home or worktree were created. {gate_note} The [browser/Eidetic checkpoint](../../../support/receipts/authority-success-audit/{NAME}) binds these findings, source coverage, duplicate-implementation map and verification limits.
'''
doc.write_bytes(original+addition.encode('utf-8'));save('review_notes.json',notes);save('authority_map.json',authority)
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
(HERE/'browser_eidetic_diff_check.txt').write_text(diff.stdout+diff.stderr,encoding='utf-8');assert diff.returncode==0
evidence=['record_browser_eidetic_review.py','browser_probe_flow_probe.mjs','browser_probe_flow_receipt.json',
 'bundle_consumer_scan.log','source_extension_recheck.json','builtin_adapter_schema.json','builtin_opaque_schema.json','builtin_training_schema.json',
 'browser_eidetic_diff_check.txt']
for stem in ['legacy_corpus','content_cache','pack_signature_order','manifest_cache','builtin_schema']:
    evidence.extend([stem+'_probe.rs','run_'+stem+'_probe.py',stem+'_probe_compile.log',stem+'_probe_output.log',stem+'_probe_receipt.json'])
if terminal:evidence.extend(['flora_gate_terminal.json','trainer_flora_fetched_unit_tests.log'])
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':old['basis_commit'],
 'kind':'Completed Distillery source and reviewed Eidetic artifact/schema/validation owners; full Mere objective incomplete',
 'production_edits_this_pass':False,'coverage':stats,'whole_current_files':whole,'whole_current_chars':254394,
 'source_sha256':{n:sha(ROOT/n) for n in whole},'trace_sha256':{n:sha(ROOT/n) for n in ['ports/distillery/src/trainer.rs','crates/eidetic/eidetic-core/src/manifest.rs','crates/system/pandect/src/image_store.rs']},
 'preserved_production_and_test_hashes':preserved,
 'prior_receipt':{'path':'admission_probe_review_receipt.json','sha256':sha(HERE/'admission_probe_review_receipt.json')},
 'new_findings':[f['id'] for f in findings],'authority_entries':[e['id'] for e in entries],
 'all_inventoried_distillery_source_complete':True,'distillery_source_files':len(distillery),
 'native_Node_VM_checks':6,'native_Rust_tests':16,'retained_library_basis':'trainer_flora_probe_build.jsonl exact artifacts before authorized merge; relevant reviewed sources unchanged',
 'pending_verification':pending,'original_flora_gate_terminal':terminal,'next_read':notes['checkpoint']['next_read'],
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(original),'preserved_prefix_sha256':hashlib.sha256(original).hexdigest()},
 'evidence_sha256':{n:sha(HERE/n) for n in evidence},'probe_outputs_removed':True,'retained_isolation':[],
 'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},
 'limits':'Full Mere source review incomplete. Node reports are controlled worker-contract inputs, not real browser/model/GPU executions. Corpus leakage is mitigated by trainer revalidation. Signature ordering is not forgery or install bypass. Schema incompatibility does not prove actual ESP load failure. No installed/user store, physical peer or native-headed trial.'}
save(NAME,receipt)
print(json.dumps({'receipt':NAME,'coverage':stats,'whole_files':len(whole),'whole_chars':254394,
 'new_findings':receipt['new_findings'],'authority_entries':receipt['authority_entries'],
 'distillery_source_complete':True,'distillery_source_files':len(distillery),'guard_paths':len(preserved),'retained_bytes':budget},indent=2))
