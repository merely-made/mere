import json,runpy,sys
from pathlib import Path
from unittest.mock import patch
target=Path(sys.argv[1]);root=Path(sys.argv[2]);action=sys.argv[3];output=Path(sys.argv[4]);files=json.loads(Path(sys.argv[5]).read_text())
def git_output(command,*args,**kwargs):
    if "ls-files" in command:return ("\0".join(files)+"\0").encode()
    if "rev-parse" in command:return b"AUDIT_INERT_SOURCE_FIXTURE_NO_REAL_GIT_COMMIT\n"
    raise AssertionError(command)
sys.argv=[str(target),action,"--repo",str(root),"--output",str(output),*sys.argv[6:]]
with patch("subprocess.check_output",side_effect=git_output):runpy.run_path(str(target),run_name="__main__")
