# Query a peer with Mere

You can ask “Who named each author of this paper?” from a laptop that has
only the query and an invitation. The other laptop opens the graph, runs the
query through Mere’s SPARQL engine, and sends the rows back. Choose WebRTC for a browser
reader, or iroh, Reticulum, or I2P for a terminal reader. Each invitation selects its own carrier.

## Open a WebRTC invitation in a browser

Only the graph owner needs Mere. In the repository root on the owner, install
the CLI and start a session. Replace the example IP with the owner's LAN
address. Choose an available UDP port.

```sh
cargo install --path crates/cli
sample=crates/graph/engine/examples/fixtures
mere serve --via webrtc --listen 192.168.1.10:43864 \
  --graph "$sample/paper-authors.trig" --allow-query "$sample/paper-authors.rq" \
  --invite paper-webrtc.html
```

When the owner sees `Ready`, privately transfer **paper-webrtc.html** to
the reader using a trusted file-sharing channel. On the reader, open the
saved file in a browser that supports WebRTC and WebTransport with certificate
pinning, such as Chrome. Use **View allowed SPARQL query** to inspect the
question, then **Run query**. If the browser asks to reach devices on the
local network, allow that connection to query the owner.

| author | namedBy |
| --- | --- |
| Ada | Library catalog |
| Ada | Publisher |
| Grace | Publisher |

The file includes the browser reader, the allowed query, a session secret,
and the fingerprint of the owner's temporary signaling certificate. Open
invitations only from an owner you trust. Each click establishes a fresh
connection. WebTransport uses the fingerprint to authenticate the owner and
exchange WebRTC connection details. The query and response travel over the
WebRTC data channel. The graph stays on the owner. The reader needs only
the HTML file and the browser.

This direct connection needs UDP reachability to the owner's signaling port
and the per-connection WebRTC port. Start with both devices on the same LAN.
A browser compatibility message means the necessary browser APIs are
unavailable. For a connection timeout, check that the owner is still serving,
the invitation is current, and the devices can reach each other over UDP.
Ctrl-C on the owner ends access. Restart with a fresh filename after changing
the graph or query, or when the temporary signaling certificate expires
after 14 days.

## Try the author query from another terminal

Install the CLI from the Mere repository root on each machine with Rust and
Cargo available. Set `sample` in each terminal where you use the example.

```sh
cargo install --path crates/cli
sample=crates/graph/engine/examples/fixtures
```

**On the graph owner**, try the question locally first.

```sh
mere query --graph "$sample/paper-authors.trig" --query "$sample/paper-authors.rq"
```

Start the service on the owner. For a first run on the same LAN, use iroh.

```sh
mere serve --via iroh --graph "$sample/paper-authors.trig" \
  --allow-query "$sample/paper-authors.rq" --invite paper.peer
```

Leave that command running. When it prints `Ready`, use a second terminal
in the same directory to send the invitation and the query. Replace
`reader-host` with the reader’s SSH hostname.

```sh
scp paper.peer crates/graph/engine/examples/fixtures/paper-authors.rq reader-host:
```

**On the reader**, run the command from the directory that received the files
(the home directory for the example above).

```sh
mere query --peer paper.peer --query paper-authors.rq
```

```json
[["Ada","Library catalog"],["Ada","Publisher"],["Grace","Publisher"]]
```

The columns follow the query’s `SELECT ?author ?namedBy`. Ada appears twice
because the library catalog and the publisher each credit her. SSH is used once
to hand over the invitation. Subsequent queries and replies use the selected carrier.

## Choose a terminal carrier

* **iroh** connects to the endpoint in the invitation through Murm’s p2mere
  adapter. The CLI uses direct connections. Start with two machines on the
  same LAN or a directly reachable peer.
* **Reticulum** uses a Reticulum link over a TCP interface. Give the owner’s
  reachable local IP address and an available TCP port. The reader learns the
  address and Reticulum destination from the invitation. For example, if the
  owner’s LAN address is `192.168.1.10`, run this on the owner.

  ```sh
  mere serve --via reticulum --listen 192.168.1.10:41863 \
    --graph "$sample/paper-authors.trig" --allow-query "$sample/paper-authors.rq" \
    --invite reticulum.peer
  ```

* **I2P** uses a running I2P router on each machine with the SAM interface
  enabled. The CLI creates a temporary published destination on the owner and
  a temporary client session on the reader. Tunnel setup can take several
  minutes. If setup times out, leave the owner and both routers running and
  retry the reader command with the same invitation. Both commands default
  to the local SAM bridge at `127.0.0.1:7656`.
  If a router uses another port, add `--sam 127.0.0.1:PORT` on that machine.

  ```sh
  mere serve --via i2p --graph "$sample/paper-authors.trig" \
    --allow-query "$sample/paper-authors.rq" --invite i2p.peer
  ```

Transfer the new invitation as above. Substitute `reticulum.peer` or
`i2p.peer` for `paper.peer` in the reader command.

## Share a query on your own graph

Choose a local RDF file and a UTF-8 SPARQL `SELECT` or `ASK` file. The
owner loads both files at startup. Each request must match the allowed query,
including whitespace (Windows and Unix line endings are treated alike).
Change either file by restarting the service and issuing a fresh invitation.

Anyone holding the invitation can run that query for the lifetime of the
service. Transfer it privately. Press Ctrl-C on the owner to end access.
Starting a new session requires a new invitation filename. The client checks
the owner’s identity before sending the invitation secret or the query.

Output is one JSON array on standard output. Cells contain Mere’s display
strings, with `null` for an unbound variable. An empty result is `[]`.
`ASK` produces one row with the string `"true"` or `"false"`. Remote query
files are limited to 64 KiB and the encoded reply to 1 MiB. Errors go to
standard error and return a nonzero exit status.
