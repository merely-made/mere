import json,runpy,subprocess,sys
from pathlib import Path
from unittest.mock import patch
target=Path(sys.argv[1]); spec=json.loads(Path(sys.argv[2]).read_text())
sys.argv=[str(target),spec.get("knot_revision", "unused")]
completed=subprocess.CompletedProcess(["AUDIT injected subprocess"],spec["child_exit"],stdout=spec["stdout"],stderr="AUDIT injected stderr")
with patch("subprocess.run",return_value=completed), patch("subprocess.check_output",return_value="AUDIT injected compiler identity; no compiler execution"):
    runpy.run_path(str(target),run_name="__main__")
