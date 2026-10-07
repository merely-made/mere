# pelt-core

`pelt-core` is the embeddable controller of the Pelt reference browser. A
`PeltController` owns one retained document session, its engine registries,
navigation history, host-neutral input effects, target size, and frame
production. `PeltWorkspace` arranges one controller per document tile through
the shared `TileTree`, retaining inactive tabs and routing Frisket content-hole
geometry without adding a window or paint dependency.

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
