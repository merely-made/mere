# Local ontology packs

A product ships a default declared lens and the subject vocabularies it offers.
The loader compiles those TOML documents with Eidetic and admits the selection
with Mere's local pack loader. The resulting profile includes the exact pins
and the terms the product can use when reading or presenting data.

A product can also pass a directory chosen by its operator. Put the replacement
lens in `lens.toml`. Its `select` list can name bundled subjects and additional
vocabulary packs in the directory. A local vocabulary with the same pack ID
takes the place of the bundled copy. Two local files declaring the same ID are
ambiguous and return an error. Changing the selection does not edit the
underlying records. Load this directory only from local configuration.

[Web Annotation](packs/web-annotation.toml) and [citation](packs/citation.toml)
are shared by multiple products. Product-specific terms can live with the
product that presents them.

[Interfaces derived from linked records](application-views.md) maps the shared
operations to writing, research, divination and music, with the evidence each
application needs for its own actions.

## Saved views

A saved view names its owner scope, the source IDs the user chose, a question,
and the exact lens and vocabulary pins. It can ask for members of a class or
for statements pointing into or out of a record. Each answer carries its
original subject, predicate and object, and the source ID and revision that
supplied them. Independent sources can make the same assertion.

On open, the host checks the active owner scope and profile. It then resolves
each selected ID through its ordinary read permissions. A removed or withheld
source has a null revision. An available source with zero matches retains
its revision. A definition contains no document bytes, access grants or cached
answers. A changed vocabulary requires an explicit new selection.

The `mere-view` executable provides a second reader for local RDF exports.
Supply the lens directory, the owner scope and the current files explicitly.
For example, with a lens selecting the bundled citation vocabulary:

```sh
mere-view define ./lens library 'Citations to the score' incoming urn:score drafts notes > citations.json
mere-view run ./lens library citations.json drafts https://example.org/drafts/ draft.ttl
```

Here `drafts` is available and `notes` is unavailable. To include the notes
on a later run, add their source ID, base IRI and RDF file. To withdraw them,
omit that binding. The reader handles one file at a time. A file revision
includes the RDF format, base IRI and content digest. For JSON-LD it also
includes the offline context URLs and document digests used by the reader.
A change to the base or a context can change what a statement means. A malformed supplied file produces
an error. Each file contributes named resource relationships from its default
graph. The CLI is a reader of files the operator supplies and has no independent
access to another application's private captures.
