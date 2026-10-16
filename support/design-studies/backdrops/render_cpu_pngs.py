#!/usr/bin/env python3
# Copyright 2026 Mark Alan Boykin
# SPDX-License-Identifier: MPL-2.0
"""Convert backdrop_receipt's RGBA output to PNG without external packages."""
import json
from pathlib import Path
import struct
import sys
import zlib


def chunk(kind, data):
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))


def main():
    output = Path(sys.argv[1])
    receipt = json.loads((output / "receipt.json").read_text())
    for frame in receipt["frames"]:
        width, height = frame["width"], frame["height"]
        pixels = (output / (frame["scene"] + ".rgba")).read_bytes()
        if len(pixels) != width * height * 4:
            raise ValueError("unexpected RGBA dimensions")
        rows = b"".join(b"\0" + pixels[y * width * 4:(y + 1) * width * 4] for y in range(height))
        png = b"\x89PNG\r\n\x1a\n"
        png += chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
        png += chunk(b"IDAT", zlib.compress(rows, 9)) + chunk(b"IEND", b"")
        (output / (frame["scene"] + ".png")).write_bytes(png)


if __name__ == "__main__":
    main()
