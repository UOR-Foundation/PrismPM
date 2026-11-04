"""Reproduce the carried portable-session fix from the exact upstream archive.

Usage: python3 scripts/rebuild-hologram-session-source.py ORIGINAL_ARCHIVE OUTPUT
The original archive is SHA-256 pinned; patch applies with no fuzz, then every
archive member retains its original metadata and order. Only patched file bytes
and their lengths change. No network or compiler is involved.
"""
import hashlib
import io
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile

BASE_SHA256 = "caf5c34ef2b21d58c1aa12acf81cb13ace1adaffb3c69a641f54f490ed61cf66"
PATCH = Path(__file__).resolve().parents[1] / "crates/prismpm/vendor/hologram-live-session-bounds.patch"


def rebuild(original, output):
    data = Path(original).read_bytes()
    if hashlib.sha256(data).hexdigest() != BASE_SHA256:
        raise ValueError("not the pinned upstream source archive")
    with tempfile.TemporaryDirectory() as work:
        with tarfile.open(fileobj=io.BytesIO(data)) as source:
            source.extractall(work, filter="data")
            subprocess.run(["patch", "--batch", "--fuzz=0", "-p1"],
                           input=PATCH.read_bytes(), cwd=work, check=True)
            with tarfile.open(output, "w", format=tarfile.GNU_FORMAT) as target:
                for member in source.getmembers():
                    if member.isfile():
                        content = (Path(work) / member.name).read_bytes()
                        member.size = len(content)
                        target.addfile(member, io.BytesIO(content))
                    else:
                        target.addfile(member)
    print(hashlib.sha256(Path(output).read_bytes()).hexdigest())


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit(__doc__)
    rebuild(*sys.argv[1:])
