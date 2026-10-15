"""Same-state WebGL2/WebGPU capture comparison: full-frame RGB RMS, max RGB and alpha deltas."""
import json, math, sys
from PIL import Image
def compare(a, b):
    A = Image.open(a).convert('RGBA'); B = Image.open(b).convert('RGBA')
    if A.size != B.size:
        return {"size_mismatch": [A.size, B.size]}
    sq = 0; n = 0; mrgb = 0; ma = 0
    for p, q in zip(A.getdata(), B.getdata()):
        for c in range(3):
            d = p[c] - q[c]; sq += d * d; n += 1; mrgb = max(mrgb, abs(d))
        ma = max(ma, abs(p[3] - q[3]))
    return {"rgb_rms": math.sqrt(sq / n), "max_rgb_delta": mrgb, "max_alpha_delta": ma, "size": list(A.size)}
gl, wg, contract = sys.argv[1], sys.argv[2], json.load(open(sys.argv[3]))
rows = []
for name in ["before-loss", "after-restore", "odd-resize"]:
    r = {"name": name, **compare(f"{gl}/{name}.png", f"{wg}/{name}.png")}
    r["pass"] = r.get("rgb_rms", 9e9) <= contract["rgb_rms_max"] and r.get("max_rgb_delta", 999) <= contract["max_rgb_delta"] and r.get("max_alpha_delta", 999) <= contract["max_alpha_delta"]
    rows.append(r)
print(json.dumps(rows, indent=2))
