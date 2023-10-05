//! EnergyIspector. Helper Inspector to calculte energy for others.
//!
use crate::interpreter::{CallInputs, CreateInputs, Energy, InstructionResult};
use crate::primitives::{db::Database, Bytes, B176};
use crate::{evm_impl::EVMData, Inspector};

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default)]
pub struct EnergyInspector {
    energy_remaining: u64,
    last_energy_cost: u64,
}

impl EnergyInspector {
    pub fn energy_remaining(&self) -> u64 {
        self.energy_remaining
    }

    pub fn last_energy_cost(&self) -> u64 {
        self.last_energy_cost
    }
}

impl<DB: Database> Inspector<DB> for EnergyInspector {
    #[cfg(not(feature = "no_energy_measuring"))]
    fn initialize_interp(
        &mut self,
        interp: &mut crate::interpreter::Interpreter,
        _data: &mut EVMData<'_, DB>,
        _is_static: bool,
    ) -> InstructionResult {
        self.energy_remaining = interp.energy.limit();
        InstructionResult::Continue
    }

    // get opcode by calling `interp.contract.opcode(interp.program_counter())`.
    // all other information can be obtained from interp.

    #[cfg(not(feature = "no_energy_measuring"))]
    fn step(
        &mut self,
        _interp: &mut crate::interpreter::Interpreter,
        _data: &mut EVMData<'_, DB>,
        _is_static: bool,
    ) -> InstructionResult {
        InstructionResult::Continue
    }

    #[cfg(not(feature = "no_energy_measuring"))]
    fn step_end(
        &mut self,
        interp: &mut crate::interpreter::Interpreter,
        _data: &mut EVMData<'_, DB>,
        _is_static: bool,
        _eval: InstructionResult,
    ) -> InstructionResult {
        let last_energy = self.energy_remaining;
        self.energy_remaining = interp.energy.remaining();
        if last_energy > self.energy_remaining {
            self.last_energy_cost = last_energy - self.energy_remaining;
        } else {
            self.last_energy_cost = 0;
        }
        InstructionResult::Continue
    }

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
}

#[cfg(test)]
mod tests {
    use crate::db::BenchmarkDB;
    use crate::interpreter::{
        opcode, CallInputs, CreateInputs, Energy, InstructionResult, Interpreter, OpCode,
    };
    use crate::primitives::{
        hex_literal::hex, Bytecode, Bytes, ResultAndState, TransactTo, B176, B256,
    };
    use crate::{inspectors::EnergyInspector, Database, EVMData, Inspector};

    #[derive(Default, Debug)]
    struct StackInspector {
        pc: usize,
        energy_inspector: EnergyInspector,
        energy_remaining_steps: Vec<(usize, u64)>,
    }

    impl<DB: Database> Inspector<DB> for StackInspector {
        fn initialize_interp(
            &mut self,
            interp: &mut Interpreter,
            data: &mut EVMData<'_, DB>,
            is_static: bool,
        ) -> InstructionResult {
            self.energy_inspector
                .initialize_interp(interp, data, is_static);
            InstructionResult::Continue
        }

        fn step(
            &mut self,
            interp: &mut Interpreter,
            data: &mut EVMData<'_, DB>,
            is_static: bool,
        ) -> InstructionResult {
            self.pc = interp.program_counter();
            self.energy_inspector.step(interp, data, is_static);
            InstructionResult::Continue
        }

        fn log(
            &mut self,
            evm_data: &mut EVMData<'_, DB>,
            address: &B176,
            topics: &[B256],
            data: &Bytes,
        ) {
            self.energy_inspector.log(evm_data, address, topics, data);
        }

        fn step_end(
            &mut self,
            interp: &mut Interpreter,
            data: &mut EVMData<'_, DB>,
            is_static: bool,
            eval: InstructionResult,
        ) -> InstructionResult {
            self.energy_inspector
                .step_end(interp, data, is_static, eval);
            self.energy_remaining_steps
                .push((self.pc, self.energy_inspector.energy_remaining()));
            eval
        }

        fn call(
            &mut self,
            data: &mut EVMData<'_, DB>,
            call: &mut CallInputs,
            is_static: bool,
        ) -> (InstructionResult, Energy, Bytes) {
            self.energy_inspector.call(data, call, is_static);

            (
                InstructionResult::Continue,
                Energy::new(call.energy_limit),
                Bytes::new(),
            )
        }

        fn call_end(
            &mut self,
            data: &mut EVMData<'_, DB>,
            inputs: &CallInputs,
            remaining_energy: Energy,
            ret: InstructionResult,
            out: Bytes,
            is_static: bool,
        ) -> (InstructionResult, Energy, Bytes) {
            self.energy_inspector.call_end(
                data,
                inputs,
                remaining_energy,
                ret,
                out.clone(),
                is_static,
            );
            (ret, remaining_energy, out)
        }

        fn create(
            &mut self,
            data: &mut EVMData<'_, DB>,
            call: &mut CreateInputs,
        ) -> (InstructionResult, Option<B176>, Energy, Bytes) {
            self.energy_inspector.create(data, call);

            (
                InstructionResult::Continue,
                None,
                Energy::new(call.energy_limit),
                Bytes::new(),
            )
        }

        fn create_end(
            &mut self,
            data: &mut EVMData<'_, DB>,
            inputs: &CreateInputs,
            status: InstructionResult,
            address: Option<B176>,
            energy: Energy,
            retdata: Bytes,
        ) -> (InstructionResult, Option<B176>, Energy, Bytes) {
            self.energy_inspector.create_end(
                data,
                inputs,
                status,
                address,
                energy,
                retdata.clone(),
            );
            (status, address, energy, retdata)
        }
    }

    #[test]
    fn test_energy_inspector() {
        let contract_data: Bytes = Bytes::from(vec![
            opcode::PUSH1,
            0x1,
            opcode::PUSH1,
            0xb,
            opcode::JUMPI,
            opcode::PUSH1,
            0x1,
            opcode::PUSH1,
            0x1,
            opcode::PUSH1,
            0x1,
            opcode::JUMPDEST,
            opcode::STOP,
        ]);
        let bytecode = Bytecode::new_raw(contract_data);

        let mut evm = crate::new();
        evm.database(BenchmarkDB::new_bytecode(bytecode.clone()));
        evm.env.tx.caller = B176(hex!("10000000000000000000000000000000000000000000"));
        evm.env.tx.transact_to =
            TransactTo::Call(B176(hex!("00000000000000000000000000000000000000000000")));
        evm.env.tx.energy_limit = 21100;

        let mut inspector = StackInspector::default();
        let ResultAndState { result, state } = evm.inspect(&mut inspector).unwrap();
        println!("{result:?} {state:?} {inspector:?}");

        for (pc, energy) in inspector.energy_remaining_steps {
            println!(
                "{pc} {} {energy:?}",
                OpCode::try_from_u8(bytecode.bytes()[pc]).unwrap().as_str(),
            );
        }
    }
}
