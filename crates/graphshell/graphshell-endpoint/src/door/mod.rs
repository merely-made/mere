//! Graphshell adaptation of the independent Djinn application door.
//! Projection clients and provider endpoints use Djinn's selected native client.
#[cfg(all(feature = "profile", any(unix, windows), not(target_arch = "wasm32")))]
pub mod profile;
#[cfg(all(
    feature = "projection-door",
    any(unix, windows),
    not(target_arch = "wasm32")
))]
pub mod projection;
#[cfg(feature = "projection-offer")]
pub mod projection_offer;
#[cfg(all(
    feature = "projection-provider",
    any(unix, windows),
    not(target_arch = "wasm32")
))]
pub mod projection_provider;
#[cfg(all(
    feature = "projection-door",
    any(unix, windows),
    not(target_arch = "wasm32")
))]
pub mod resident_mere_proxy;
