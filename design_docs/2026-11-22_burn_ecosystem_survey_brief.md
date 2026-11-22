# Burn ecosystem survey: what outside Burn, CubeCL and wgpu work fits the stack

**Date:** 2026-11-22
**Status:** research brief with two rulings (§8, 2026-11-22). Nothing is adopted or scheduled. Four
projects were read at source (§3 to §6); everything in §7 is known from
crates.io and GitHub metadata only and is marked unread.
**Home:** Mere holds this because it spans Distillery (`esp`), Woodshed, Hocket
and Graphshell. Any adoption belongs in the receiving repo's own plan.
Woodshed's `design_docs/2026-07-11_audio_material_analysis_plan.md` owns the
audio-to-material benchmark; this brief supplies it candidates.

## 1. Origin

Mark, 2026-11-20, on [Watershed](https://github.com/edarbieto/watershed):

> i can see something like this being of utility for hocket

Mark, 2026-11-22:

> i wonder if there are other burn projects like this that would be relevant to the stack, like watershed, bunsen, potential wgpu suppliers, etc.

> i don't mind adopting things that are useful and interesting and synergize well with the stack even if they've been abandoned

So candidates are judged on usefulness, fit and license. Last activity is
reported as a fact that sizes the port, not as a verdict.

## 2. The finding that sorts everything: the Burn 0.22 device API

`esp` is on Burn `=0.22.1` and writes `Tensor<2>` and `&Device` with no backend
type parameter. Watershed and bunsen are on 0.22.1 and are written the same
way. Projects on Burn 0.20 and 0.21 are written as `Model<B: Backend>`.

That splits the field into two port sizes:

- **Same cohort (0.22.x):** Watershed, bunsen, maolan-pitch. Code lifts close to
  as-is.
- **Older cohort (0.18 to 0.21):** demucs-rs, voxtral-mini-realtime-rs, kord,
  WhisperForge, wav2vec-burn, voxcpm-rs. Each needs its `B: Backend` signatures
  rewritten to the device API, and anything touching CubeCL internals rewritten
  again.

Distillery is also partly on `0.23.0-pre.2`, so anything lifted is a port into
the family's tree, not a dependency row.

## 3. Watershed (read at `1046832`, MIT OR Apache-2.0, Burn 0.22.1)

A practice player that separates a song into six stems with BS-RoFormer.

- **`crates/roformer`** (about 1,600 lines): the model, f16 weights, and chunked
  separation with progress and cancel callbacks. It reads PyTorch checkpoints
  without running pickle code, and safetensors.
- **It carries its own CubeCL kernels** (softmax, head split and merge, a
  hand-picked matmul strategy table) because Burn's generic attention was slow
  on an RTX 3050. Each has a plain-Burn fallback, which is what the CPU tests
  run. The kernels bind `cubek =0.3.1` and `CubeTensor` internals, and the
  strategy thresholds were tuned on two machines.
- **Parity is tested without a GPU:** a tiny random-weight fixture matches
  PyTorch per layer to 1e-3, STFT and chunking to 1e-4. Real-model figures in
  its docs: f32 masks within 2.8e-6, f16 about 3e-3.
- **Its stated cost:** about 79 s for a 4:39 song on an M5 Pro, about 109 s on
  an RTX 3050, 1.2 to 1.6 GB of GPU memory in f16. A GPU is required.
- **The weights declare no license.** Watershed records `license = "unknown"`,
  pins the Hugging Face URL to a commit with a sha256, downloads on first use
  and never bundles. The family would have to do the same.
- **`crates/dsp`** is CPU-only and uses no Burn: spectral-flux tempo with
  autocorrelation, a beat tracker for performances not played to a click
  (Viterbi tempo curve, then Ellis 2007 dynamic programming, with user anchors),
  Krumhansl-Kessler key estimation, and triad chord estimation with a
  bass-stem root vote and a Viterbi path. Tests are synthetic clicks and ramps,
  not real recordings.
- **C dependencies are avoidable.** `signalsmith-stretch` (C++) is only in the
  player crate; R33 already struck it. `vorbis_rs` bundles libvorbis and is in
  `dsp`; drop that file. The downloader pulls `ring` through rustls.

**Fit:** the strongest lift in this survey. The model goes to `esp`; the tempo,
key and chord estimators go to Woodshed as baseline observers for the
audio-to-material benchmark; Hocket gets "import a mix as tracks" through its
project worker. Hocket tracks are mono, so stems need a downmix or a stereo
decision.

## 4. bunsen (read at `d6ce5ea`, MIT OR Apache-2.0, Burn 0.22.1)

A single-maintainer suite of about 190,000 lines, version 0.43. The esp
consolidation plan already names its group optimizer as a borrow. It is much
larger than that:

- **Speech with timing.** Parakeet TDT returns `TimedWord { text, start, end }`
  in seconds. Its Voxtral port returns word starts to within 80 ms. There are
  also Whisper, Qwen3-ASR, Silero voice activity detection, Sortformer
  diarization, Qwen3-TTS and ZONOS3. A stream driver handles windowing, voice
  activity and where to cut between words.
- **A data-driven model catalog.** YAML rows give each build a contract (the
  loader's resource and argument slots), resources with sha256, size and
  format, sources, an origin URL and a license. The cache path includes the
  hash. Upstream checkpoints load directly; key renaming happens in Rust.
- **Readers** for safetensors, PyTorch, NeMo archives, MLX and a zstd archive
  format, in pure Rust.
- **Group optimizer:** parameter groups keyed by module path, each with its own
  optimizer, learning-rate selector and clipping, checked to partition the
  module's parameters.
- **Limits:** no WebAssembly target; native tokenizers use Oniguruma (C), which
  `esp` already accepts natively; it owns its own cache, device choice and
  KV-cache policy, which overlap `esp` and Eidetic's model library.

**Fit:** the answer for timed transcripts, ahead of the standalone Voxtral port.
Harvest by kit, Parakeet first, since `timed-text` wants word times. The catalog
row shape is worth comparing against `esp`'s model and template rows, mainly for
its per-build license and origin fields.

## 5. voxtral-mini-realtime-rs (read at `305f936`, Apache-2.0, Burn 0.20)

- No timestamps in its API, and no carried state between chunks; each chunk is
  decoded on its own and the texts are joined.
- The f32 path is backend-generic but needs 9.2 GB. The usable path is Q4:
  703 MB and faster than real time, but tied to wgpu and CubeCL 0.9 internals
  through a custom fused dequantize-and-multiply WGSL kernel. Q4 word error is
  8.49% against Mistral's reported 4.90% at full precision.
- It vendors a patched `cubecl-wgpu` to keep `cubek-reduce` inside WebGPU's 256
  invocations per workgroup. Distillery carries its own `cubek-reduce` patch;
  whether both answer the same fault is unchecked.

**Fit:** not a lift. Worth reading for two things `esp` does not have: Q4
execution on the GPU, and its browser loading (512 MB shards, two-phase load,
asynchronous readback), which bear on `esp`'s wasm lane and layer streaming.

## 6. demucs-rs (read at `9f156d0`, Apache-2.0, Burn 0.20.1)

- HTDemucs four-stem, six-stem and fine-tuned variants in about 4,600 lines of
  core, pure Rust, generic over the backend, with a CPU feature.
- Thin verification: no parity fixtures in Rust, one ignored test, and
  comparison against PyTorch is a manual Python script with no recorded
  numbers. No timing figures.
- Weights come from a third party's Hugging Face mirror with no hash check and
  no license statement. Meta's Demucs repository is MIT; whether that covers
  the checkpoints needs checking at the source.

**Fit:** a second separation backend, and the one that runs without a GPU. It
needs the 0.22 port plus the parity tests it lacks. Watershed first.

## 7. Wider field (unread; crates.io reverse dependencies of `burn` and `cubecl`)

| Project | What | License | Burn | Might serve |
|---|---|---|---|---|
| [maolan-pitch](https://github.com/maolan/pitch) | Neural pitch detection (FCPE) and pitch-correcting resynthesis | BSD-2-Clause | 0.22.1 | Woodshed vocal pitch |
| [kord](https://github.com/twitchax/kord) | Music theory library; trains and runs chord and note recognition from audio | MIT | 0.20 | Woodshed chord catalog and benchmark |
| [WhisperForge](https://github.com/bevsxyz/WhisperForge) | Whisper with streaming, quantization, and an alignment crate that writes subtitles | MIT | 0.21 | `timed-text` |
| [wav2vec-burn](https://github.com/privacyresearchgroup/wav2vec-burn) | wav2vec 2.0 | MIT | 0.20 | Forced alignment |
| [fluxion](https://github.com/matteospanio/fluxion) | Lowers a DSP graph to CubeCL and CPU SIMD | MIT | CubeCL 0.10 | Woodshed's DSP graph |
| [dsp-kitchen](https://github.com/neuro-kitchen/dsp-kitchen) | Multi-channel DSP kernels in CubeCL | Apache-2.0 | CubeCL 0.11 | Filter and resampling kernels |
| [gpu-fft](https://github.com/eugenehp/gpu-fft) | FFT in CubeCL | MIT | CubeCL 0.9 | STFT on the GPU |
| [gloss](https://github.com/Meshcapade/gloss) `wgpu-burn-global-device`, `wgpu-burn-interop` | Read 2026-11-22 at `e0f7ebf`: 37 lines registering a host wgpu device with CubeCL, 82 lines copying a tensor's buffer into a new wgpu buffer | MIT | 0.18 | Nothing: Conatus `ResidentClient` already registers the host device and lends planes without a copy |
| [manifolds-rs](https://github.com/GregorLueg/manifolds-rs), [node2vec-rs](https://github.com/GregorLueg/node2vec-rs) | UMAP, t-SNE, PaCMAP, ForceAtlas2; node embeddings | MIT | 0.22 | Graphshell layouts and projections |
| [harper](https://github.com/automattic/harper) `harper-pos-utils` | Part-of-speech tagging for a grammar checker | Apache-2.0 | 0.19 | Knot |
| [brush](https://github.com/ArthurBrussee/brush) | Gaussian splat training and viewing | Apache-2.0 | n/a | Isocosm, if splats enter |
| [burn-onnx](https://github.com/tracel-ai/burn-onnx) | ONNX to native Burn code | Apache-2.0 | n/a | Any model with no Rust port; bunsen uses it for parity references |
| [stable-diffusion-burn-wgpu](https://github.com/neka-nat/stable-diffusion-burn-wgpu) | Stable Diffusion; untouched since 2023 | MIT | 0.11 | Image generation, a long port |

**Video (unread; metadata only).** These are image models run over frames:

| Project | What | License | Burn |
|---|---|---|---|
| [av-denoise](https://github.com/ChillFish8/av-denoise) | Non-local-means video denoising kernels, as a crate | Apache-2.0 | CubeCL 0.11 |
| [senmei](https://github.com/senmei-app/senmei) | Upscaling, denoising and frame interpolation; media through FFmpeg | Apache-2.0 | unchecked |
| [vid2transparent](https://github.com/zwh20081/vid2transparent) | Video matting with MatAnyone2 and SAM 2; its description says non-commercial | MIT (code) | unchecked |
| [mlvc-rs](https://github.com/Lyamc/mlvc-rs) | Decoder for a learned video codec | MIT | unchecked |
| `gst-plugin-burn` (gst-plugins-rs) | A GStreamer element that runs Burn | MPL-2.0 AND (MIT OR Apache-2.0) | 0.21 |

The stack's only video decode today is Genet's media component, on GStreamer.

Neighbor, not a Burn project: [Maolan](https://github.com/maolan/maolan) is a
Rust DAW (BSD-2-Clause) with its own engine, out-of-process plugin host, CLAP
runtime and a clean-room ASIO host. Hocket is deliberately not a DAW, but its
host audio work covers the same ground.

**Excluded by license:** `kingwingfly/voice` and `rust-asr` (GPL-3.0),
`imazen/jxl-gpu` (AGPL-3.0). **No license declared:** `tracel-ai/burn-lm`,
`shigedangao/brioche`, `ninjawarrior1337/maizou`. Whisper-burn (MIT, 2024) is
superseded by bunsen's Whisper on the same Burn version.

## 8. Rulings and open questions

Ruled by Mark, 2026-11-22:

- **`esp` gets an `audio` namespace** beside `infer` and `embed`:

  > we can have an `audio` namespace.

- **Weights are fetched, never bundled.** The family ships no weights or models
  it did not make:

  > i think it's ok for people to fetch weights and I don't want to bundle weights, models, etc. that i don't make myself

  So a checkpoint with no declared license (BS-RoFormer SW) is admissible as a
  hash-pinned row the person fetches. A row should still record the license or
  its absence, and the origin.

Open, nothing scheduled:

1. Does Watershed's kernel set come along, or only the plain-Burn path until
   measured on the family's machines?
2. bunsen by kit, or as a dependency behind an `esp` feature? It tracks Burn
   releases closely, which cuts both ways under the track-main rule.
3. Is the `cubek-reduce` patch in Distillery the same fault Voxtral patched in
   `cubecl-wgpu`?
4. Further `esp` namespaces, discussed 2026-11-22. A namespace follows a
   contract shape, not a model family.
   - **`image`: agreed by Mark.** He would "almost consider that part of
     knot", since Knot documents embed images. Proposed and not yet ruled:
     execution stays in `esp`, as Knot's embedding features already do, with
     Knot the first host and owner of what an image means in a document;
     Isocosm, Inker and Graphshell are likely second hosts.
   - **Video folds into `image`.** Per-frame work is image work. What video
     adds is state carried across frames, which is a session shape inside
     `image`. Decode, encode and pacing are not model execution. Learned
     codecs are out of scope until something wants one.
   - **Text labeling** (text in; spans and tags out). Mark expects it to reach
     several products, the Cosmograph games among them. That argues for its
     own namespace sharing the encoder in `embed`; the name is open. In a
     game a label is a proposal one authority commits to the record; peers
     read it and do not recompute it.
   - **Policy: proposed dropped.** Mark asked whether it is Gemot's. Gemot's
     policy is governance and must stay legible. The one learned policy in the
     stack, Cosmograph's lineage net, is integer so every peer replays it, and
     belongs in the game. Gemot's role with models is the FLORA split: it
     governs membership, weights and adoption; Distillery transforms.
