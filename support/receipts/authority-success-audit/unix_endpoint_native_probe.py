"""Native Linux primitive evidence for source-observed Rust cleanup branches.

This does not execute the Rust/Tokio functions or launch an installed resident.
It checks the current source branch and demonstrates its connect/unlink policy
using the same native Unix socket and filesystem operations in Python.
"""
from pathlib import Path
import errno
import hashlib
import json
import socket
import stat
import tempfile

ROOT = Path(__file__).resolve().parents[3]
paths = [
    "ports/djinn/src/bin/djinn.rs",
    "crates/graphshell/graphshell-endpoint/src/door/local_endpoint.rs",
]
sources = {}
for path, name in zip(paths, ["prepare_unix_agent_endpoint", "prepare_unix_endpoint"]):
    raw = (ROOT / path).read_bytes()
    text = raw.decode("utf-8-sig")
    start = text.index("async fn " + name)
    brace = text.index("{", start)
    # Both helper bodies contain no braces in string literals. Count the
    # actual body rather than crediting unrelated matches elsewhere in file.
    depth, end = 1, brace + 1
    while depth:
        depth += (text[end] == "{") - (text[end] == "}")
        end += 1
    body = text[start:end]
    assert "tokio::net::UnixStream::connect(endpoint).await" in body
    assert "std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused" in body
    assert "std::fs::remove_file(endpoint)" in body
    assert "symlink_metadata" not in body and "file_type" not in body
    sources[path] = {
        "sha256": hashlib.sha256(raw).hexdigest(),
        "helper_sha256": hashlib.sha256(body.encode()).hexdigest(),
        "helper": name,
    }


def prepare_policy(path):
    """Model only the source's connect/refusal/unlink decision, explicitly."""
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as probe:
        try:
            probe.connect(str(path))
        except OSError as error:
            if error.errno not in (errno.ENOENT, errno.ECONNREFUSED):
                raise
            observed_errno = error.errno
        else:
            raise OSError(errno.EADDRINUSE, "live listener refused")
    try:
        path.unlink()
    except FileNotFoundError:
        pass
    return observed_errno


observations = {}
with tempfile.TemporaryDirectory(prefix="mere-unix-endpoint-probe-") as name:
    root = Path(name).resolve()
    (root / "GENERATED-AUDIT-FIXTURE").write_text("Only generated endpoint probe data.\n")
    regular = root / "regular"
    regular.write_bytes(b"generated ordinary-file canary")
    assert stat.S_ISREG(regular.lstat().st_mode)
    observed = prepare_policy(regular)
    assert observed == errno.ECONNREFUSED and not regular.exists()
    observations["regular_file"] = {"connect_errno": observed, "removed_by_policy": True}

    missing = root / "missing"
    assert prepare_policy(missing) == errno.ENOENT
    observations["missing"] = "accepted without a file"

    stale = root / "stale.sock"
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as previous:
        previous.bind(str(stale))
    assert stat.S_ISSOCK(stale.lstat().st_mode)
    assert prepare_policy(stale) == errno.ECONNREFUSED
    assert not stale.exists()
    observations["stale_socket"] = "removed as intended"

    live = root / "live.sock"
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as listener:
        listener.bind(str(live))
        listener.listen(1)
        try:
            prepare_policy(live)
        except OSError as error:
            assert error.errno == errno.EADDRINUSE
        else:
            raise AssertionError("live listener was not refused")
        assert live.exists()
    observations["live_socket"] = "refused and preserved"
    # Every socket is now closed before TemporaryDirectory removes generated
    # fixture data. No product path is used for a cleanup experiment.

print(json.dumps({
    "kind": "native Linux primitives and source branch checks; not Rust API execution",
    "sources": sources,
    "observations": observations,
    "fixture_removed": not root.exists(),
}, indent=2))
