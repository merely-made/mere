//! Payloads: one version byte, then CBOR (D6), at most [`MAX_PAYLOAD`] bytes
//! (D13). Both sides check the cap.

use crate::Failure;
use serde::{Serialize, de::DeserializeOwned};

pub const VERSION: u8 = 1;
/// The browser carrier's native-message cap, the tightest carrier's.
pub const MAX_PAYLOAD: usize = 1024 * 1024;

pub fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, Failure> {
    let mut bytes = vec![VERSION];
    ciborium::into_writer(value, &mut bytes).map_err(|e| Failure::Malformed(e.to_string()))?;
    check(&bytes)?;
    Ok(bytes)
}

pub fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, Failure> {
    check(bytes)?;
    match bytes.split_first() {
        Some((&VERSION, body)) => {
            ciborium::from_reader(body).map_err(|e| Failure::Malformed(e.to_string()))
        },
        Some((version, _)) => Err(Failure::Malformed(format!("payload version {version}"))),
        None => Err(Failure::Malformed("empty payload".into())),
    }
}

pub fn check(bytes: &[u8]) -> Result<(), Failure> {
    if bytes.len() > MAX_PAYLOAD {
        return Err(Failure::TooLarge(bytes.len() as u64));
    }
    Ok(())
}
