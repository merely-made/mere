import json,runpy,subprocess,sys
from pathlib import Path
from unittest.mock import patch
target=Path(sys.argv[1]); spec=json.loads(Path(sys.argv[2]).read_text())
sys.argv=[str(target),spec.get("knot_revision", "unused")]
def injected_run(*args, **kwargs):
    text=kwargs.get("text",False)
    output=spec["stdout"] if text else spec["stdout"].encode()
    error="AUDIT injected stderr" if text else b"AUDIT injected stderr"
    return subprocess.CompletedProcess(["AUDIT injected subprocess"],spec["child_exit"],stdout=output,stderr=error)
with patch("subprocess.run",side_effect=injected_run), patch("subprocess.check_output",return_value="AUDIT injected compiler identity; no compiler execution"):
    runpy.run_path(str(target),run_name="__main__")
