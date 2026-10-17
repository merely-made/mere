# tabard

Tabard is Mere's shared theme-authoring library, consumed by platform crates
and application ports.

A tabard is the garment that displays a household's livery. This library is where
a household's livery gets authored: seeds in, liveries out. It composes what
already interlocks: [tinct](https://crates.io/crates/tinct) derives the full
palette from a few seed colours, [illume](https://crates.io/crates/illume)
emits syntax spans that the derived syntax palette colours, and palette-aware
icons can recolour at render time.

Tabard lives in `crates/system/tabard`: the shared theme model, palettes and
persistence are platform capabilities. Dependencies continue to flow from ports
into crates, with no exception for Tabard.

Tabard authors portable theme artifacts. Livery consumes stylesheets Tabard
emits. Pelt has an optional named preview receipt for those artifacts, but does
not own their model.
Tinct remains the small, serde-only derivation crate. Illume continues to name
syntax spans rather than decide their appearance.

## Current artifact

The first implementation is deliberately portable and library-only:

- Theme owns an id, a name and tinct::Seeds, plus the registry's derivation
  settings (source, high contrast, harmony, per-mode sheets), then derives
  Tinct's normal-contrast base palette. It is also the registry's authored
  theme, which was a separate ThemeDef until 2026-09-24.
- The theme module holds the host theme model, moved from Mere's registry on
  2026-09-24: ThemeTokenSet and its ThemeRegistry, seed derivation, custom
  modes, chrome colours, edge-style tokens, and the theme a lens carries.
- theme::choice holds the user's theme choice (a theme id and an optional
  mode) and where it is kept: the store seam, an in-memory store, and an
  atomic JSON file store that still reads Pelt's one-line appearance files.
- The smolweb module holds the smolweb document palettes, which Nematic's views
  and Mere's document lanes each defined until 2026-09-24: SmolwebTheme,
  SmolwebPalette, and the palette choice, including the per-site tint.
- Theme::design_tokens emits a typed DTCG 2025.10 color document. Every token
  has an explicit color type and a structured sRGB value, while the name,
  seeds, and derivation choice live under org.merely.tabard in $extensions.
- Theme::css_custom_properties emits a deterministic :root stylesheet with the
  same palette as --tabard-color-* custom properties. Livery can consume it
  as an ordinary author sheet. Success and danger are published gated to
  4.5:1 on bg and surface (the raw seeds stay in provenance and in the
  Lagrange export). After the sixteen base roles it emits Tinct's
  contrast-gated roles (accent, focus, border, border-strong, category-1 to
  category-5, from tinct::derive_gated_roles); the DTCG document carries them
  under color.gated.
- Theme::lagrange_palette_txt emits a deterministic `palette.txt` for
  Lagrange's application UI. It writes the documented `# Dark` and `# Light`
  sections in Lagrange's neutral, accent, and status-label order using
  `#RRGGBB` values. The return value carries diagnostics for Tabard roles
  which collapse into one Lagrange label or have no representation. This
  exporter does not change Lagrange page themes, which are selected separately
  from a site palette seed.

  The emitted label list follows the [v1.21.1 help vocabulary](https://raw.githubusercontent.com/skyjake/lagrange/v1.21.1/res/about/help.gmi).
  That release's stock [`loadPalette_Color` table](https://raw.githubusercontent.com/skyjake/lagrange/v1.21.1/src/color.c)
  omits the documented `yellow` and `magenta` labels, so the artifact retains
  those lines for documented-shape compatibility and reports a
  `StockVersionIgnored` diagnostic for each mode. A headed load receipt for a
  specific Lagrange build is required before claiming that all emitted labels
  are applied.

Its documented projection is explicit: dark neutrals are `bg`, `surface`,
`surface-2`, `surface-hover`, `text`; light neutrals are `text`, `text-dim`,
`surface-2`, `surface-hover`, `surface`. The remaining labels map as
`brown`/`orange` dim/bright variants of `primary`, and `teal`/`cyan` as
dim/bright variants of `secondary`. `red=danger` and `green=success` are used
only when their authored hues pass the exporter’s red/green semantic checks;
otherwise Lagrange’s documented defaults are emitted. Lagrange’s `yellow`,
`magenta`, and `blue` reserved colors likewise use its documented defaults.
The repeated accent mappings, defaults, omitted Tabard roles, and discarded
alpha are returned as diagnostics rather than hidden in the artifact.
- Pelt's optional `tabard-preview` receipt maps those generic properties onto
  Pelt-owned Chrome roles. It proves the shell recolors while the focused
  document, session history, tabs, and content aperture remain held. It is not
  a persistent appearance setting and does not recolor document content.
- Pelt's optional `tabard-reader-preview` receipt maps the same portable
  palette onto Reader's existing host palette. It proves a Fleece article
  keeps its held source and route-restoration behavior while Pelt supplies the
  Reader colors. Fleece and `genet-documents` do not depend on Tabard.

Recorded 2026-08-28: both named headed Windows consumer receipts passed at
960x640. `tabard-preview` completed after three redraws with compositor digest
`d0affd3746b03554`; `tabard-reader-preview` completed after nine redraws with
digest `ea505825544747b9`. The latter held a `genet.reader` Fleece article
beside a `genet.livery` neighbor and retained the Reader inspector's lineage.
These receipts validate consumer seams, not persistence or a platform theme
policy.

Theme::design_tokens carries Tinct's syntax palette as a color.syntax group,
one token per syntax role, and Theme::css_custom_properties emits the same
roles as --tabard-syntax-* properties.

Icon policy, imports, a DTCG resolver and a Geopard exporter are not here yet;
the host theme model and its persistence arrived with the theme module.

## Authoring workspace

`workshop::ThemeDraft` provides the first host-independent workshop model:
open a user theme or fork any theme into an unregistered draft, apply typed
edits, undo/redo, discard, and explicitly commit to the in-memory registry.
It protects built-ins and refuses save conflicts. Canonical-mode previews
use the existing theme derivation; authored stylesheet previews are returned
as sheets, and unresolved custom calculators are reported explicitly.

Draft previews never change the active theme. Registry commits do not write
files. `library::ThemeLibraryStore` loads and saves a versioned authored-theme
library separately from the host's appearance choice. It validates user
definitions, refuses stale/busy writers and replaces the file atomically;
failed writes leave its loaded snapshot intact.

The reusable [Cambium workshop](../../cambium/tabard-workshop) composes that draft and
library into seed/harmony controls, mode previews, history and explicit Save,
Discard and Reopen actions. Its [desktop host](../../../ports/tabard/desktop)
mounts the same surface. Hosts still own applying appearance; stylesheet
editing, custom calculator previews and mode-aware export parity remain
follow-ups in the theme modes plan.

## License

MPL-2.0 (see the repository `LICENSE`)

## Authored typography styles

`Theme::typography` carries optional explicit Interface, Reading, Heading, Label
and Code styles through `ThemeFontStyle`. The role enum and family validation
reuse the published `typography` contract. Application-owned ordered family
requests remain separate on `ThemeChoice.fonts` and take precedence over theme
family defaults. Each role can request a family, CSS-pixel size and unitless leading.
Empty preferences preserve host defaults. Validation rejects blank/control
character family names and nonfinite or out-of-range metrics before save or
export. Named families emit safely quoted CSS plus an explicit generic fallback;
`missing_families` reports availability without rewriting the definition.

CSS exports include `--tabard-font-{role}`, `--tabard-font-{role}-size` and `-line-height`
only when authored. `Typography::role_css` lets each consumer supply its own
semantic selectors. `css_property_declarations` and `role_declarations` also
scope these same declarations to isolated retained specimens without changing
the surrounding editor. Hosts retain layout, installed-face discovery and font
measurement authority; this model carries no font bytes. Authored mode sheets
remain exact and follow the font variable definitions.

Libraries containing font preferences use version 2; color-only writes preserve
version 1. Portable font-bearing exports use `{ "version": 2, "theme": ... }`.
The new reader accepts existing color-only files, while old hosts refuse the
new version rather than silently losing fonts. Theme drafts, forks, history,
conflict checks and library persistence retain the role map. Typed DTCG tokens
remain color-only; explicit-mode provenance retains the complete definition.
