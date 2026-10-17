from pathlib import Path
p=Path(__file__).resolve().parent
assert not (p/'search_boundaries_probe_v2.rs').exists() and not (p/'run_search_boundaries_probe_v2.py').exists()
(p/'search_boundaries_probe_v2.rs').write_text((p/'search_boundaries_probe.rs').read_text().replace('../../../../crates/', '../../../crates/'),encoding='utf-8')
s=(p/'run_search_boundaries_probe.py').read_text().replace("NAME='search_boundaries_probe'", "NAME='search_boundaries_probe_v2'")
(p/'run_search_boundaries_probe_v2.py').write_text(s,encoding='utf-8')
