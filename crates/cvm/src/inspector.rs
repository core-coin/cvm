use crate::cvm_impl::EVMData;
use crate::interpreter::{CallInputs, CreateInputs, Energy, InstructionResult, Interpreter};
use crate::primitives::{db::Database, Bytes, B176, B256};

use auto_impl::auto_impl;

#[cfg(feature = "std")]
pub mod customprinter;
pub mod energy;
pub mod noop;
#[cfg(feature = "serde")]
pub mod tracer_eip3155;

/// All Inspectors implementations that cvm has.
pub mod inspectors {
    #[cfg(feature = "std")]
    pub use super::customprinter::CustomPrintTracer;
    pub use super::energy::EnergyInspector;
    pub use super::noop::NoOpInspector;
    #[cfg(feature = "serde")]
    pub use super::tracer_eip3155::TracerEip3155;
}

#[auto_impl(&mut, Box)]
pub trait Inspector<DB: Database> {
    /// Called Before the interpreter is initialized.
    ///
    /// If anything other than [InstructionResult::Continue] is returned then execution of the interpreter is
    /// skipped.
    fn initialize_interp(
        &mut self,
        _interp: &mut Interpreter,
        _data: &mut EVMData<'_, DB>,
        _is_static: bool,
    ) -> InstructionResult {
        InstructionResult::Continue
    }

    /// Called on each step of the interpreter.
    ///
    /// Information about the current execution, including the memory, stack and more is available
    /// on `interp` (see [Interpreter]).
    ///
    /// # Example
    ///
    /// To get the current opcode, use `interp.current_opcode()`.
    fn step(
        &mut self,
        _interp: &mut Interpreter,
        _data: &mut EVMData<'_, DB>,
        _is_static: bool,
    ) -> InstructionResult {
        InstructionResult::Continue
    }

    /// Called when a log is emitted.
    fn log(
        &mut self,
        _cvm_data: &mut EVMData<'_, DB>,
        _address: &B176,
        _topics: &[B256],
        _data: &Bytes,
    ) {
    }

    /// Called after `step` when the instruction has been executed.
    ///
    /// InstructionResulting anything other than [InstructionResult::Continue] alters the execution of the interpreter.
    fn step_end(
        &mut self,
        _interp: &mut Interpreter,
        _data: &mut EVMData<'_, DB>,
        _is_static: bool,
        _eval: InstructionResult,
    ) -> InstructionResult {
        InstructionResult::Continue
    }

    /// Called whenever a call to a contract is about to start.
    ///
    /// InstructionResulting anything other than [InstructionResult::Continue] overrides the result of the call.
    fn call(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        _inputs: &mut CallInputs,
        _is_static: bool,
    ) -> (InstructionResult, Energy, Bytes) {
        (InstructionResult::Continue, Energy::new(0), Bytes::new())
    }

    /// Called when a call to a contract has concluded.
    ///
    /// InstructionResulting anything other than the values passed to this function (`(ret, remaining_energy,
    /// out)`) will alter the result of the call.
    fn call_end(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        _inputs: &CallInputs,
        remaining_energy: Energy,
        ret: InstructionResult,
        out: Bytes,
        _is_static: bool,
    ) -> (InstructionResult, Energy, Bytes) {
        (ret, remaining_energy, out)
    }

    /// Called when a contract is about to be created.
    ///
    /// InstructionResulting anything other than [InstructionResult::Continue] overrides the result of the creation.
    fn create(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        _inputs: &mut CreateInputs,
    ) -> (InstructionResult, Option<B176>, Energy, Bytes) {
        (
            InstructionResult::Continue,
            None,
            Energy::new(0),
            Bytes::default(),
        )
    }

    /// Called when a contract has been created.
    ///
    /// InstructionResulting anything other than the values passed to this function (`(ret, remaining_energy,
    /// address, out)`) will alter the result of the create.
    fn create_end(
        &mut self,
        _data: &mut EVMData<'_, DB>,
        _inputs: &CreateInputs,
        ret: InstructionResult,
        address: Option<B176>,
        remaining_energy: Energy,
        out: Bytes,
    ) -> (InstructionResult, Option<B176>, Energy, Bytes) {
        (ret, address, remaining_energy, out)
    }

    /// Called when a contract has been self-destructed with funds transferred to target.
    fn selfdestruct(&mut self, _contract: B176, _target: B176) {}
}
