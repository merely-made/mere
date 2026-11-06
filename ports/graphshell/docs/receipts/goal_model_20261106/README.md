# Goal surface with the model assessor, headed native receipt (2026-11-06)

Eidetic's S3 `ModelGoalAssessor` (S1 plus all-MiniLM-L6-v2) drives the goal
surface's bounded Fleece traversal behind an opt-in "Use the model" toggle, on
the native proof host. Off by default; the ordinary lexical path is unchanged
when the toggle is off or the feature is not built.

## Source cohort
- Eidetic rev 86e78dd37914fd61f8ca336bf0ab2da716356fca (Mere's pin; not changed). ESP (distillery) rev a235cbdbca187aa7507739b3f6cf9a2f7846fcc0.
- Features: `--no-default-features --features goal-surface-proof,goal-surface-model`, bin `goal-surface-host --receipt DIR --size 1280x920 --model`, env `GRAPHSHELL_GOAL_MODEL_DIR`.
- Host: Apple M4 (Metal), macOS, shared Cambium winit host, 1280x920 logical (2560x1840 captured). Debug build.

## Model (verified by SHA-256 before load; never downloaded)
all-MiniLM-L6-v2, ESP `bert-f32`, CPU (Burn NdArray), folder `/Users/markik/Code/testing/eidetic/models/all-MiniLM-L6-v2`:
- config.json 953f9c0d463486b10a6871cc2fd59f223b2c70184f49815e7efbcab5d8908b41
- tokenizer.json be50c3628f2bf5bb5e3a7f17b1f74611b2561a3a27eeab05e5aa30f411572037
- model.safetensors 53aa51172d142c89d9012cce15ae4d6cc0ca6895895114379cacb4fab128d9db

Threshold: `match_similarity_milli` = 220, from `recommended_match_similarity_milli(space)` (Eidetic's MiniLM calibration, Or rule);
the generic default 500 is wrong for MiniLM. Rule `Or`. Load on the worker thread: 1787 ms (includes hashing 91 MB).

## Result (qualification.json)
Goal "preserve my sourdough while I am on holiday" (shares no term with any page), start Bread notes index, 6 lookups, stop Exhausted.
- Toggle off (stage-1): 0 of 5 matched; "No retained source met the goal."
- Toggle on (stage-3/4): 2 of 5 matched, both found only by the model: Baking while travelling (similarity 0.425) and Starter care (0.311);
  each shows "Lexical: not a match, coverage 0%", the similarity and threshold, and "Decided by: model (the lexical rule found no match)".
  Oven spring 0.191, Flour types 0.130, Bread notes index 0.153 are below 0.220 ("neither"). The explanation shows model vs lexical selection:
  at the fourth pick the model took the unretained Hotel kitchens (0.203) where the lexical rule would have taken Starter care.
  The run summary line reports "lexical rule alone would have matched 0 of the sources read; the combined rule matched 2".
- `traversal_receipt.model` carries the full `ModelGoalRecord` (envelope, settings, per-selection scores, per-assessment similarity and rule); `traversal_receipt.goal` is S1's record unchanged.

Precision caveat from Eidetic's calibration (0.67): a low threshold also passes some unrelated text; here Starter care (0.311) is a reasonable but not exact answer.

## Captures (large; not committed)
/Users/markik/Code/testing/eidetic/goal_model_20261106/wide/stage-{0..4}.png, hashes in captures.sha256. All read by eye:
0 empty surface with the toggle (off); 1 lexical run, no matches; 2 model ready (status line: checksums verified, threshold 0.220; stale lexical results still shown); 3 model run results; 4 scrolled walk and run summary.

## Not qualified
Physical pointer/keyboard (clicks dispatched via DOM callbacks); release-mode timing; other platforms; GPU/wgpu embedding; browser/wasm (the feature is excluded from web builds);
real user content (synthetic fixture); models other than the pinned MiniLM; precision on real corpora; cancelling a model run in flight.
