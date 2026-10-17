"""Keep the original reader intact while including executable extension manifests.

JSON evidence and fixture data stay excluded. These two JSON files are actual
browser install/permission/entry-point configuration, not receipt data.
Use this wrapper for future full source-inventory refreshes.
"""
import read_coverage as reader
EXTRA_PATHS={
    'ports/graphshell/web/extension/manifest.chromium.json',
    'ports/graphshell/web/extension/manifest.firefox.json',
}
base=reader.is_source
def is_source(path):
    return base(path) or path.relative_to(reader.ROOT).as_posix() in EXTRA_PATHS
reader.is_source=is_source
if __name__=='__main__':reader.inventory()
