"""Assemble exact source modules, not copied index/provider algorithms."""
from pathlib import Path
import json,hashlib
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
dest=HERE/'embedding_core_probe.rs';assert not dest.exists()
modules=['index','provider','sparse','lexical','stub','search','affinity']
body='mod embed {\n'
sources={}
for name in modules:
 p=ROOT/('crates/intel/esp/src/embed/'+name+'.rs')
 sources[p.relative_to(ROOT).as_posix()]=hashlib.sha256(p.read_bytes()).hexdigest()
 body+='#[path = '+json.dumps(p.as_posix())+'] pub mod '+name+';\n'
body+='pub use index::VectorIndex; pub use provider::{EmbeddingProvider, SimilarityMetric}; pub use sparse::{SparseVector, SparseIndex}; pub use lexical::LexicalEmbeddingProvider; pub use stub::StubEmbeddingProvider;\n}\n'
body+=(HERE/'embedding_core_cases.rs').read_text()
dest.write_text(body,encoding='utf-8')
with (HERE/'embedding_core_source_modules.json').open('x',encoding='utf-8') as f:
 f.write(json.dumps({'source_sha256':sources,'qualification':'Seven unchanged current embedding source modules compiled together in their original embed namespace, including their original unit tests. No whole ESP infer/Burn/actor/model-session build.'},indent=2)+'\n')
