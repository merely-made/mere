# Packaging and releasing Dramatis

The app uses Hocket's `cargo-packager` → `luggage-manifest` → signed Luggage
feed path. Its package contains `dramatis-ui`, license notices and public build
information. Djinn remains the existing user-scoped resident. Install/remove
of this app must preserve that resident and its vault.
The resident needs the quarantine API from Mere `ec1e65ca` or a descendant,
with its default `cxf` feature. `release-info.json` declares that source floor;
Djinn's `0.0.2` package version spans builds before and after those routes.

Run these commands from Mere's root with Python 3.11+ and cargo-packager.
On Windows, builds reuse `C:/t/cargo-targets/mere`; other hosts must set
`CARGO_TARGET_DIR` to their existing approved Mere target. One compiler job is
the default; increase `--jobs` only after checking other build owners and
available memory. The package script never cleans the shared target.

## Build and sign on each native host

Commit the compiled source and version in `ports/dramatis/Cargo.toml` first.
Set `CARGO_PACKAGER_SIGN_PRIVATE_KEY` to the path of the dedicated Dramatis
release key outside every repository. Set its password through
`CARGO_PACKAGER_SIGN_PRIVATE_KEY_PASSWORD`; keep both out of configuration,
logs and source. Qualification uses disposable keys, which are not release
keys and are removed after the receipt.

```sh
python ports/dramatis/package.py pack --jobs 1
```

The driver chooses NSIS on Windows, `.app` on macOS and AppImage on Linux.
`--format` can make that choice explicit; cross-host packing is refused.
The `beforePackagingCommand` hook performs a locked build and checks the
binary's `--version` and `--build-info` against the current package source,
workspace manifests and Cargo.lock. A source/lock change during the build or
a stale binary refuses packing. Version validation avoids cleaning other
lanes' artifacts. The packer receives only the explicit `dramatis-ui` binary.

Output is marker-owned generated data under the shared target:
`dramatis-packaging/release/<format>/`. It includes the signed installer or
bundle and `luggage.json` / `luggage.json.sig`. The driver uses the existing
Luggage CLI to hash the artifact and assemble the manifest, then signs the
manifest last. Any later merge/edit requires signing it again.

For a public release, provide the artifact's final HTTPS URL:

```sh
python ports/dramatis/package.py pack --jobs 1 \
  --artifact-url https://github.com/merely-made/mere/releases/download/dramatis-v0.0.1/dramatis-ui_0.0.1_x64-setup.exe
```

Use a product-qualified tag such as `dramatis-v0.0.1`, a dedicated Dramatis
publisher key and a stable **Dramatis-only** manifest URL. The repository's
global latest release may belong to another Mere product. Local qualification
defaults to a directory feed with `file://` artifact URLs.

Each host produces its artifact; collect its platform entry in one manifest
using `luggage-manifest`. Match the version across entries, preserve the
artifact signatures/digests, and sign the combined manifest **after** the
last host is added. For macOS the feed points at the packer's signed
`.app.tar.gz`, not the `.app` directory. Set the real host's code-signing
identity with `--macos-signing-identity` or `DRAMATIS_MACOS_SIGNING_IDENTITY`;
notarization uses cargo-packager's Apple environment/keychain settings.
Windows Authenticode and
macOS Gatekeeper trust are separate from Luggage's release signatures.

On Linux, a host without libfuse2 may need `APPIMAGE_EXTRACT_AND_RUN=1` to run
packaging tools; use `NO_STRIP=1` only when that host's linuxdeploy strip
fails against its system libraries. These are host qualifications, not
universal application defaults.

## Local packaging qualification

```sh
python support/dramatis_package_receipts.py
```

This Windows-only gate uses the warm debug profile, an ephemeral signing key,
an extracted NSIS package and an absent synthetic Djinn endpoint. It validates
the extracted binary's source/version, runs its native window, and drives
Luggage's real signed-manifest, signed-artifact and durable staging paths.
Unsigned/modified manifests, a different valid publisher key, bad artifact
signatures and modified artifact/staged bytes must be rejected. Controls
restore every feed byte; the signing keys and extraction are removed.

`--profile dev` belongs to this gate. It establishes the packaging path, not
an optimized, OS-signed, installed or publicly released build. Packaging does
not add automatic self-update policy or change Djinn custody/consent. Luggage
feeds are ready for the existing suite update policy to consume; app update
controls and resident installation remain separate work.

See `support/receipts/dramatis-packaging/README.md` for qualified results.
Fedora/ThinkPad, Intel macOS and ARM macOS qualification stays pending until
the hosts are reachable. No remote build is queued by this script.
