# pelt-core

`pelt-core` is the embeddable controller of the Pelt reference browser.

- A `PeltController` owns one retained document session, its engine
  registries, navigation history, host-neutral input effects, target size,
  and frame production.
- A `PeltContent` is one routed piece of browsing content: a document lane
  (a `PeltController`) or a surface lane (a live web-engine producer), behind
  one command, input, frame and routing API. Every new load is routed through
  the shared `PeltRegistries`:
  - A document lane changes engine as its addresses and response media types
    require. History reopens each entry with the engine it was shown with.
  - A load that routes to a surface swaps lanes, and a surface asked for a
    document address swaps back. History does not cross lanes.
- `PeltWorkspace` arranges one `PeltContent` per document tile through the
  shared `TileTree`, retaining inactive tabs and routing Frisket content-hole
  geometry, without adding a window or paint dependency.

A host with its own arrangement holds `PeltContent` directly, keyed however
it likes.

Concrete engines receive resource policy when the caller registers them.
The controller receives a caller-owned clock and returns the engine's generic
frame type. Window creation, wgpu device and queue ownership, rasterization,
and presentation remain with the embedding host. This lets standalone Pelt,
Tabard previews, and focused test hosts drive the same controller without
depending on winit or a paint backend.

## Loading

A controller loads top-level documents in one of two modes. In the engine
mode, the original one, the document engine fetches through the fetcher it
was registered with, synchronously inside its spawn. In the host mode
(`PeltControllerConfig::with_host_loading`), the host's transport fetches:

- The controller queues `PeltLoadCommand`s (`Fetch` and `Cancel`), drained
  with `take_load_commands`.
- The host feeds back `accept_progress` and `accept_outcome`, in the
  `mere-fetch` actor's vocabulary.
- Every answer is gated on its exact request, so a superseded or stopped
  transfer never surfaces.
- Reload mints a new request for the same entry, and Stop cancels the exact
  request and keeps the current page.
- A streamed prefix opens the document. Later fragments replace its body in
  place where the host installed a body replacer.
- Input, client identity and certificate changes stay typed in
  `PeltDocumentState::Awaiting`, for the host's own conversation.
- Downloads come back in the host effect.

The engine only ever receives held bodies. The load itself is `page-load`'s
sans-IO `PageLoad`, which a host with its own model can also own directly.

## History

A controller keeps history in one of two modes. In the linear mode, the
original one, links and GET forms push entries, and Back and Forward
traverse them. In the host mode (`PeltControllerConfig::with_host_history`,
or `PeltRegistries::with_host_history` for routed content), the host keeps
history:

- The controller holds only its current entry.
- A link or GET form comes back as a `PeltNavigationRequest`, with its cause
  and the modifiers held, and nothing loads.
- Back and Forward on a document come back unhandled.
- The host loads the entry it chose with `open`, which replaces the current
  entry, routes like any new load, and opens a held body directly even when
  the host's transport fetches.

Turnstone's history is its graph: a link opens or mints a node.
`PeltContent::open` does the same in either lane. A web surface keeps its own
engine history in both modes.
