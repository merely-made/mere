"""Use the preserved current build closure, without replacing any cited gate."""
from pathlib import Path
source=(Path(__file__).parent/'run_mesh_board_probe.py').read_text(encoding='utf-8')
source=source.replace('mesh_board','mesh_execution')
source=source.replace("'mesh_execution_build.jsonl'","'mesh_current_build.jsonl'")
source=source.replace("'tests':9","'tests':7")
source=source.replace("'scope':'Nine native public-API observations with synthetic signed operations and real MemoryBackend MeshStore. Direct fold order dependence; mutable parsed address versus cached signed bytes; completed lease output and attested device loss through actual store checkpoint acceptance; over-grant completion selected by actual board. No network, live peer divergence, forged wire signatures, user files, production edits, redb or durable user state.'", "'scope':'Seven native public-API observations: worker policy bypasses, final-await cancellation, unbound delayed exact identity, weaker local verification verdict, catch-up predecessor failure and device omission. CPU, synthetic signatures, real MemoryBackend and built-in delayed adapter; no probe network, redb, user files, GPU, production edits or shared-source modifications.'")
exec(compile(source,str(__file__),'exec'))
