# Batch 74: production native window recovery

Audit date: 2026-10-12. Incremental judgment of the production controlled
owned-device destruction gate, its preserved strict negative and corrected
acceptance under the existing renderer limits. This supersedes the named
batch 73 records; performance evidence remains separately scoped.
The original D2 snapshot and inherited findings are unchanged.

## cambium_docs/technical_architecture/2026-10-11_shared_device_recovery_owner.md

- disposition: current
- status line: "Implemented; native controlled-loss and current-frame qualification passed." — accurate: yes
- claims checked: 10 — holds: 10, stale: 0, unverifiable: 0

### Stale claims

The formerly open one-window surface recovery gate has executed. Physical
driver reset, multiple native windows and asynchronous browser recovery remain open.

### Contradictions

None. A new observed generation is not relabeled as a measured factory count.
Queue presentation is not physical compositor acknowledgement.

### Recommended action

Keep arbitrary mid-frame loss, browser async recovery and OS accessibility
qualification separate. Preserve existing pixel predicates in subsequent runs.

### Notes

The nine previously judged architecture and healthy-presentation groups hold.
The added group is actual production Device::destroy, callback observation,
normal owner/surface recreation, preserved runner/sources/authority and new
successful presentation. Source-reviewed frame-local drop ordering is separate
from this after-collected-baseline injection. Native r2 reports 56 presentations,
four captures, no timeout and pixels within unchanged limits.

## cambium_docs/testing/receipts/2026-10-12_flattening_and_device_recovery/README.md

- disposition: current
- status line: "Controlled native surface recovery" — accurate: yes
- claims checked: 13 — holds: 13, stale: 0, unverifiable: 0

### Stale claims

The one-window post-destruction queue-presentation gate is no longer pending.
Older healthy, occluded and strict-negative evidence retains its original scope.

### Contradictions

No exact whole-window equality is claimed. The new fixture initially required
zero difference, exceeding the established acceptance: r1 changed 30 pixels by
RGB 1, alpha 0. The strict negative is preserved. Final r2 uses the existing
stricter interior RGB 3 / alpha 2 / mean 1.0 limits uniformly, without the looser
edge/neighbor allowances. This does not change the renderer acceptance thresholds.

### Recommended action

Retain both rounds and their binary/source/scenario identities. Use the
presentation-bound producer rectangles to delimit displayed pixel evidence;
do not infer complete hidden source pixels from staging identities.

### Notes

The prior twelve receipt groups remain source-qualified. The new controlled
window group is checked against the actual scenario, native r1/r2 command/exit
logs, branded presentation metadata, RGBA comparisons and four r2 PNGs.
Installed reason Destroyed corresponds to generation 1, followed by generation 2
in the same owner and native host. Before/after captured sequences are 26/40.
Both named preview generations change to 2 and their stamps increase; old core
weak identity is released, runner and source identities stay intact, and the
scenario verifies full authored state, focus/selection/zoom and continued input/resize.
R2 changes 26 whole-window pixels by at most RGB 1, alpha 0, mean
0.000001376065. Reader visible-region difference is one pixel by RGB 1;
Stylesheet visible-region bytes are identical. Source staging and displayed
region comparisons are distinct. Six actual CPU controls reject over-limit
RGB/alpha/mean and unarmed/windowless destruction. Captures were visually
reviewed; Root stopped or observed finite expiry of its exact owned inhibitors
and explicitly returned runtime. Borrowed CPU cache leases were separate.
No physical reset, multiple-window, complete hidden-source or OS AT acceptance
is inferred. The source qualifier's full MPL notice is restored without changing
its metadata logic.

## DOC_README.md

- disposition: current
- status line: "Controlled owned-device destruction also recovers one native surface" — accurate: yes
- claims checked: 3 — holds: 3, stale: 0, unverifiable: 0

### Stale claims

The former one-window post-destruction pending phrase is updated to executed
qualification. Browser, OS and physical-reset gates remain open.

### Contradictions

None in these bounded entries; unrelated index claims are not requalified.

### Recommended action

Keep exact source/proof identities and open consumer gates visible when
integrating subsequent parallel work.

### Notes

Recovery-owner, production receipt and timing paths resolve. The recovery entry
matches the measured scope; the original 53-frame healthy gate and historical
captures remain valid, and performance conclusions are unchanged.
