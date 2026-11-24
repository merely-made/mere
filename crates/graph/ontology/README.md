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
