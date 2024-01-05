#![cfg_attr(not(feature = "std"), no_std)]

mod cvm;
mod cvm_impl;
pub mod db;
mod inspector;
mod journaled_state;

#[cfg(all(feature = "with-serde", not(feature = "serde")))]
compile_error!("`with-serde` feature has been renamed to `serde`.");

pub(crate) const USE_ENERGY: bool = !cfg!(feature = "no_energy_measuring");
pub type DummyStateDB = InMemoryDB;

pub use cvm::{cvm_inner, new, EVM};
pub use cvm_impl::EVMData;
pub use db::{Database, DatabaseCommit, InMemoryDB};
pub use journaled_state::{JournalEntry, JournaledState};

extern crate alloc;

/// reexport `cvm_precompiles`
pub use cvm_precompile as precompile;

// reexport `cvm_interpreter`
pub use cvm_interpreter as interpreter;

// reexport `cvm_primitives`
pub use cvm_interpreter::primitives;

/// Reexport Inspector implementations
pub use inspector::inspectors;
pub use inspector::Inspector;
