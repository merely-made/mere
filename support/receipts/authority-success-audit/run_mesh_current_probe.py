"""Repeat the preserved probe on updated main without overwriting cited evidence."""
from pathlib import Path
source=(Path(__file__).parent/'run_mesh_board_probe.py').read_text(encoding='utf-8')
source=source.replace('mesh_board','mesh_current')
source=source.replace("'mesh_current_probe.rs'","'mesh_board_probe.rs'")
exec(compile(source,str(__file__),'exec'))
