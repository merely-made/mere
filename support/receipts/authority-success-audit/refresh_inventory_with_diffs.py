"""Preserve the existing inventory rules and include source-bearing .diff files."""
import refresh_inventory as existing

reader = existing.reader
previous = reader.is_source
def is_source(path):
    return previous(path) or path.suffix.lower() == '.diff'
reader.is_source = is_source
if __name__ == '__main__':
    reader.inventory()
