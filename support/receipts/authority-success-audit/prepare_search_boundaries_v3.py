from pathlib import Path
p=Path(__file__).resolve().parent
assert not (p/'search_boundaries_probe_v3.rs').exists() and not (p/'run_search_boundaries_probe_v3.py').exists()
(p/'search_boundaries_probe_v3.rs').write_bytes((p/'search_boundaries_probe_v2.rs').read_bytes())
s=(p/'run_search_boundaries_probe_v2.py').read_text().replace("NAME='search_boundaries_probe_v2'", "NAME='search_boundaries_probe_v3'")
s=s.replace("(fixture/'.audit-generated').write_text('mere authority audit disposable search fixtures\\n',encoding='utf-8')", "(fixture/'.audit-generated').write_bytes(b'mere authority audit disposable search fixtures\\n')")
assert s!=(p/'run_search_boundaries_probe_v2.py').read_text()
(p/'run_search_boundaries_probe_v3.py').write_text(s,encoding='utf-8')
