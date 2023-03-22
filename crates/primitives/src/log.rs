use crate::{bytes::Bytes, B176, B256};
use alloc::vec::Vec;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Log {
    pub address: B176,
    pub topics: Vec<B256>,
    #[cfg_attr(feature = "serde", serde(with = "crate::utilities::serde_hex_bytes"))]
    pub data: Bytes,
}
