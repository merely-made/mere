# Mere

Mere is a Rust library for working with linked data. It keeps a graph of
resources such as pages, papers, and people. A connection in that graph can
express a claim as simple as “this paper cites that paper.” Mere can keep who
made the claim, where it came from, and whether it was later withdrawn.

Suppose a library catalog names Ada as the author of a paper, and a
publisher credits both Ada and Grace. An application using Mere can keep
both accounts, ask which sources credit Grace, and show the reader the
evidence for each answer. It can read and write the graph in standard
linked-data formats such as JSON-LD and Turtle. [Engine API](crates/graph/engine/src/lib.rs)

## Mere in the Turnstone browser

In Turnstone, open a page and choose **Save source and keep** in the
Inspector. Turnstone decides which edition of the page to save. Graphshell
supplies Cambium, the shared interface toolkit that draws the Inspector’s
button and delivers the click. When the saved page contains structured
claims, Mere can record them with a link to the exact edition and the block
that supplied them.

If the publisher later changes the author list at the same address, the
earlier capture can still tell you what the page said when you saved it.
To try the graphical interface, start with [Turnstone’s browser
instructions](https://github.com/merely-made/turnstone#build-and-run).
For the implementation, see the [Turnstone save scenario](https://github.com/merely-made/turnstone/blob/main/scenarios/r1_source_keep_seed.scn),
[Graphshell toolkit](https://github.com/merely-made/graphshell), and
[Mere’s saved page claims](https://github.com/merely-made/mere/blob/main/crates/system/pandect/src/graph_session/page_claims.rs).

## Who named each author?

The terminal inspector can answer the question from the catalog example above.
The [sample graph](crates/graph/engine/examples/fixtures/paper-authors.trig)
records each author credit and the organization that made it. From the
repository root, run the inspector with the included [SPARQL query](crates/graph/engine/examples/fixtures/paper-authors.rq).

```sh
sample=crates/graph/engine/examples/fixtures
cargo run -p mere --features query --example inspect -- \
  "$sample/paper-authors.trig" --query "$(cat "$sample/paper-authors.rq")"
```

The inspector prints a summary of the graph, followed by these rows.

```text
query (3 rows): author | namedBy
  Ada | Library catalog
  Ada | Publisher
  Grace | Publisher
```

Ada appears twice because two sources credit her. The query follows each
credit to its recorded source, then looks up both names. To ask about your
own paper, use its identifier in place of `ex:paper` and supply a graph with
the same attribution and name fields. If a credit has no recorded source, this query returns
no row for that credit.

## Across devices

If a colleague sends you this graph, running the query on your copy gives
the same answer. The publisher still gets credit for naming Grace even when
your colleague’s laptop delivers the file. The terminal inspector reads a
local file. For live graph sharing, Turnstone has a **Co-op** view where the
owner can invite a contact and grant access to a particular graph. In a
test on two Linux desktops, Turnstone used Djinn, the local background
service on each device, and a direct iroh connection. The guest edited the
shared graph and the owner’s copy retained the guest’s attribution. [Two-device sharing example](https://github.com/merely-made/turnstone/blob/main/design_docs/archive_docs/receipts/2026-10-28_graphshell_host_plan_receipts.md#2026-11-20-n2-native-co-op-on-t-pc-and-s-pc)

## Development

The [workspace manifest](Cargo.toml) lists the engine packages and dependencies.
The [toolchain file](rust-toolchain.toml) selects Rust for the checkout.
The repository records resolved dependency versions in Git notes. Use
`house resolved show <commit>` to inspect a recorded resolution.
For work across sibling checkouts, see the [local Cargo configuration
example](.cargo/config.toml.example). The [commit hooks](.pre-commit-config.yaml)
list the repository checks.

## License

[MPL-2.0](LICENSE)
