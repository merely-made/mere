//! The C0 contract's native side, shared by every binding route.
//!
//! [`Provider`] is what a provider implements, keyed by a session handle the
//! binding maps to its resource. [`Api`] is what a consumer binding exposes;
//! [`script::run`] drives one and returns a transcript the routes compare.

pub mod payload;
pub mod provider;
pub mod script;

use serde::{Deserialize, Serialize};

/// Why a call returned no answer (the WIT `failure`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Failure {
    Revoked,
    Expired,
    TooLarge(u64),
    Refused(String),
    Malformed(String),
}

/// A projection advanced (the WIT `notice`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notice {
    pub epoch: u64,
    pub revision: u64,
}

/// A snapshot request without its session, which the resource holds.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectionAsk {
    pub version: chirograph::ProtocolVersion,
    pub score: sceno::Score,
}

/// An intent without its session.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IntentCall {
    pub target: sceno::InstanceId,
    pub observed_epoch: chirograph::SceneEpoch,
    pub observed_revision: chirograph::Revision,
    pub intent: String,
    pub payload: Vec<u8>,
}

pub type Reply = Result<Vec<u8>, Failure>;

/// A provider of projection sessions. `session` is the binding's handle for
/// one opened session; dropping it is [`Provider::close`].
pub trait Provider {
    fn discover(&mut self) -> Reply;
    fn open(&mut self, request: &[u8]) -> Result<(u64, Vec<u8>), Failure>;
    fn snapshot(&mut self, session: u64, ask: &[u8]) -> Reply;
    fn fetch(&mut self, session: u64, address: &[u8]) -> Reply;
    fn chunk(&mut self, session: u64, address: &[u8], offset: u64, length: u32) -> Reply;
    fn resume(&mut self, session: u64, epoch: u64, revision: u64) -> Reply;
    fn intent(&mut self, session: u64, call: &[u8]) -> Reply;
    fn suspend(&mut self, session: u64) -> Result<(), Failure>;
    fn take_notice(&mut self, session: u64) -> Option<Notice>;
    fn wait_notice(&mut self, session: u64) -> Result<Notice, Failure>;
    fn close(&mut self, session: u64);
}

/// What a consumer binding exposes: the same calls through its own session
/// type, whose close is the resource's drop.
pub trait Api {
    type Session;
    fn discover(&mut self) -> Reply;
    fn open(&mut self, request: Vec<u8>) -> Result<(Self::Session, Vec<u8>), Failure>;
    fn snapshot(&mut self, s: &Self::Session, ask: Vec<u8>) -> Reply;
    fn fetch(&mut self, s: &Self::Session, address: Vec<u8>) -> Reply;
    fn chunk(&mut self, s: &Self::Session, address: Vec<u8>, offset: u64, length: u32) -> Reply;
    fn resume(&mut self, s: &Self::Session, epoch: u64, revision: u64) -> Reply;
    fn intent(&mut self, s: &Self::Session, call: Vec<u8>) -> Reply;
    fn suspend(&mut self, s: &Self::Session) -> Result<(), Failure>;
    fn take_notice(&mut self, s: &Self::Session) -> Option<Notice>;
    fn wait_notice(&mut self, s: &Self::Session) -> Result<Notice, Failure>;
    fn close(&mut self, s: Self::Session);
}
