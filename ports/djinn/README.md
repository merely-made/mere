# Djinn

Djinn is Mere's local desktop resident. It composes the owner-held parts of a
personal device: Personae authority, the SSH agent, Graphshell's local browser
and application brokers, personal sync,
[Knot's](https://github.com/merely-made/knot-editor) source/sync/evidence custody,
per-profile Castellan custody, and the shared physical blob store.

Graphshell remains the local session and admission protocol. Knot remains the
Djot editor and document authority. Djinn owns their shared process lifetime
when they use persona-held state.

## Run

```text
cargo run -p djinn --bin djinn -- --dir <personae-vault> --data-root <data-root>
```

The default configuration continues to read the existing Graphshell
application directory. That is a compatibility bridge for selected profiles,
pairing records, and content-store migrations, not an additional resident.

## Saved Gemini sites

`djinn-site` submits an ordinary saved Knot site to the running resident through
its owner-only application broker. Each selected Personae profile has one
publication, its own certificate, and independently retained content.

```text
djinn-site publish <site-directory> 1965 --resume
djinn-site status
djinn-site stop
djinn-site remove
```

The port is optional; zero asks the operating system for a free loopback port.
Status reports the bound address, snapshot digest and certificate fingerprint.
Browse `gemini://localhost:<reported-port>/` in Lagrange. The stock 1.21.1 client
passed with this hostname; its IP-literal certificate check refused the same
loopback certificate, so `127.0.0.1` is not an accepted Lagrange URL receipt.
`--resume` explicitly enables serving after a resident restart. Without it,
the saved publication starts stopped on the next run. The default listener is
restricted to `127.0.0.1`. The broker endpoint follows
`GRAPHSHELL_APP_ENDPOINT`, matching the resident's `--app-endpoint` when one is
selected.

Publishing selects saved bytes once. Later source edits become visible only
after another successful publish. Stop releases the listener and retains the
publication. Remove forgets the serving record and releases its storage lease;
status reports retained custody if cleanup needs retrying. The profile's TLS
identity remains available for its next publication. Missing or mismatched
identity files are refused during serving restoration.

The initial caller is a command-line utility. Knot's desktop controls, explicit
certificate rotation, additional protocols, public binding and governed moot
hosting remain separate integration work. Exporting Tabard's Lagrange palette
does not change Gemini page content or this listener's policy.

## Publishing

`0.0.2` is the source version of this workspace resident, not a crates.io
release. Knot Editor is pinned from its own public repository; the Graphshell
composition still uses workspace-only dependencies, so `cargo package` correctly
refuses it. A public Djinn release needs an installable package boundary and a
staged release of the Mere dependencies it exposes.

## Security boundary

Djinn holds durable authority but does not manufacture public services. Its
default personal-sync policy can use local discovery when the owner has
configured it; relays remain owner-selected transport configuration.

The following deployments are intentionally absent until a forcing consumer
defines their policy and acceptance receipt:

- a public Knot publisher;
- a Misfin receiver;
- a Gemot community host;
- a dedicated relay;
- a Secret Service daemon.

Castellan record and freshness keys are separately derived from the unlocked
Personae identity and opened per Djinn profile. Starting Secret Service also
requires a concrete persona selection and allowed-caller policy, so Djinn does
not guess either from a profile name.

## Lifecycle

Djinn uses Distillery's `lifecycle` module (the `mere-resident` crate until
2026-09-23) for the small rule shared with Distillery: close
resources in a concrete order, attempt every close, and retain every failure.
It does not share product policy, configuration, or service APIs with
Distillery.

## License

MPL-2.0
