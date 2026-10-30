//! Isolated source consumer: no Mere renderer/browser or unrelated port devdeps.
#[path = "../../../ports/moot/src/coop.rs"]
pub mod coop;
#[path = "../../../ports/moot/src/ocapn_coop.rs"]
pub mod ocapn_coop;
#[cfg(test)]
mod tests;
