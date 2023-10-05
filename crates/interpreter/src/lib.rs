#![cfg_attr(not(feature = "std"), no_std)]

pub mod energy;
mod host;
pub mod inner_models;
pub mod instruction_result;
mod instructions;
mod interpreter;

extern crate alloc;
extern crate core;

pub(crate) const USE_ENERGY: bool = !cfg!(feature = "no_energy_measuring");

// Reexport primary types.
pub use energy::Energy;
pub use host::{DummyHost, Host};
pub use inner_models::*;
pub use instruction_result::InstructionResult;
pub use instructions::opcode::{self, OpCode, OPCODE_JUMPMAP};
pub use interpreter::*;
pub use interpreter::{BytecodeLocked, Contract, Interpreter, Memory, Stack};

pub use revm_primitives as primitives;
