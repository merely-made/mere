"""Exact decoder observer/math source; real retained HF tokenizer; no model."""
from pathlib import Path
import json,hashlib
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
dest=HERE/'decoder_text_math_probe.rs';assert not dest.exists()
provider=ROOT/'crates/intel/esp/src/infer/decoder/provider.rs';s=provider.read_text()
generation=s[s.index('#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize'):s.index('\nstruct StreamObserver')]
observer=s[s.index('struct StreamObserver'):s.index('\nimpl DecoderProvider {')]
decode=s[s.index('    fn decode(&self, ids:'):s.index('\n    fn prepare_generation(')]
train=ROOT/'crates/intel/esp/src/infer/decoder/train.rs';t=train.read_text()
rank=t[t.index('pub fn expected_token_rank('):t.index('\n/// Count how many cases')]
portable=ROOT/'crates/intel/esp/src/infer/provider.rs';sampler=ROOT/'crates/intel/esp/src/infer/decoder/sample.rs'
body='use std::ops::ControlFlow; use tokenizers::Tokenizer;\n'
body+='#[path='+json.dumps(portable.as_posix())+'] mod portable;\nuse portable::{InferError,GenerationRequest};\n'
body+='#[path='+json.dumps(sampler.as_posix())+'] mod sample;\n'
body+='// Fixture owns only the real tokenizer; no decoder model is compiled.\nstruct DecoderProvider { tokenizer: Tokenizer, add_special_tokens: bool }\n'
body+='impl DecoderProvider {\n'+decode+'}\n'+generation+observer+rank+'\n'+(HERE/'decoder_text_math_cases.rs').read_text()
with dest.open('x',encoding='utf-8') as f:f.write(body)
manifest={'source_sha256':{p.relative_to(ROOT).as_posix():sha(p) for p in [provider,train,portable,sampler]},'qualification':'Exact unchanged StreamObserver, DecoderGeneration, decode method, and expected_token_rank extracted with hash binding; two unchanged full source modules include original tests. Fixture DecoderProvider holds only a real tokenizers 0.20.4 tokenizer. No model, full provider, Burn, inference/session or GPU execution.'}
with (HERE/'decoder_text_math_source.json').open('x',encoding='utf-8') as f:json.dump(manifest,f,indent=2);f.write('\n')
