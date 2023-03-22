#![cfg_attr(not(feature = "std"), no_std)]

pub mod bits;
pub mod bytecode;
pub mod db;
pub mod env;
pub mod log;
pub mod precompile;
pub mod result;
pub mod specification;
pub mod state;
pub mod utilities;

extern crate alloc;

pub use bits::{B176, B256};
pub use bytes;
pub use bytes::Bytes;
pub use hex;
pub use hex_literal;

/// The address type consists of the last 20 bytes of the hash of the core blockchain account, 
/// with the network prefix and checksum pre-appended.
pub type Address = B176;

/// Hash, in Core usually Sha-3.
pub type Hash = B256;

pub use bitvec;
pub use bytecode::*;
pub use env::*;
pub use hashbrown::{hash_map, HashMap};
pub use log::Log;
pub use precompile::*;
pub use result::*;
pub use ruint;
pub use ruint::aliases::U256;
pub use ruint::uint;
pub use specification::*;
pub use state::*;
pub use utilities::*;
