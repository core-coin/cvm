//! Inspector that support tracing of EIP-3155 https://eips.ethereum.org/EIPS/eip-3155

use crate::inspectors::EnergyInspector;
use crate::interpreter::{CallInputs, CreateInputs, Energy, InstructionResult};
use crate::primitives::{db::Database, hex, Bytes, B176};
use crate::{cvm_impl::EVMData, Inspector};
use cvm_interpreter::primitives::U256;
use cvm_interpreter::{opcode, Interpreter, Memory, Stack};
use serde_json::json;
use std::io::Write;

pub struct TracerEip3155 {
    output: Box<dyn Write>,
    energy_inspector: EnergyInspector,
    #[allow(dead_code)]
    trace_mem: bool,
    #[allow(dead_code)]
    trace_return_data: bool,

    stack: Stack,
    pc: usize,
    opcode: u8,
    energy: u64,
    mem_size: usize,
    #[allow(dead_code)]
    memory: Option<Memory>,
    skip: bool,
}

impl TracerEip3155 {
    pub fn new(output: Box<dyn Write>, trace_mem: bool, trace_return_data: bool) -> Self {
        Self {
            output,
            energy_inspector: EnergyInspector::default(),
            trace_mem,
            trace_return_data,
            stack: Stack::new(),
            pc: 0,
            opcode: 0,
            energy: 0,
            mem_size: 0,
            memory: None,
            skip: false,
        }
    }
}

impl<DB: Database> Inspector<DB> for TracerEip3155 {
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

    // get opcode by calling `interp.contract.opcode(interp.program_counter())`.
    // all other information can be obtained from interp.
    fn step(
        &mut self,
        interp: &mut Interpreter,
        data: &mut EVMData<'_, DB>,
        is_static: bool,
    ) -> InstructionResult {
        self.energy_inspector.step(interp, data, is_static);
        self.stack = interp.stack.clone();
        self.pc = interp.program_counter();
        self.opcode = interp.current_opcode();
        self.mem_size = interp.memory.len();
        self.energy = self.energy_inspector.energy_remaining();
        //
        InstructionResult::Continue
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
        if self.skip {
            self.skip = false;
            return InstructionResult::Continue;
        };

        self.print_log_line(data.journaled_state.depth());
        InstructionResult::Continue
    }

    fn call(
        &mut self,
        data: &mut EVMData<'_, DB>,
        _inputs: &mut CallInputs,
        _is_static: bool,
    ) -> (InstructionResult, Energy, Bytes) {
        self.print_log_line(data.journaled_state.depth());
        (InstructionResult::Continue, Energy::new(0), Bytes::new())
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
        self.energy_inspector
            .call_end(data, inputs, remaining_energy, ret, out.clone(), is_static);
        // self.log_step(interp, data, is_static, eval);
        self.skip = true;
        if data.journaled_state.depth() == 0 {
            let log_line = json!({
                //stateroot
                "output": format!("{out:?}"),
                "energyUser": format!("0x{:x}", self.energy_inspector.energy_remaining()),
                //time
                //fork
            });

            writeln!(
                self.output,
                "{:?}",
                serde_json::to_string(&log_line).unwrap()
            )
            .expect("If output fails we can ignore the logging");
        }
        (ret, remaining_energy, out)
    }

    fn create_end(
        &mut self,
        data: &mut EVMData<'_, DB>,
        inputs: &CreateInputs,
        ret: InstructionResult,
        address: Option<B176>,
        remaining_energy: Energy,
        out: Bytes,
    ) -> (InstructionResult, Option<B176>, Energy, Bytes) {
        self.energy_inspector
            .create_end(data, inputs, ret, address, remaining_energy, out.clone());
        (ret, address, remaining_energy, out)
    }
}

impl TracerEip3155 {
    fn print_log_line(&mut self, depth: u64) {
        let short_stack: Vec<String> = self.stack.data().iter().map(|&b| short_hex(b)).collect();
        let log_line = json!({
            "pc": self.pc,
            "op": self.opcode,
            "energy": format!("0x{:x}", self.energy),
            "energyCost": format!("0x{:x}", self.energy_inspector.last_energy_cost()),
            //memory?
            "memSize": self.mem_size,
            "stack": short_stack,
            "depth": depth,
            //returnData
            //refund
            "opName": opcode::OPCODE_JUMPMAP[self.opcode as usize],
            //error
            //storage
            //returnStack
        });

        writeln!(self.output, "{}", serde_json::to_string(&log_line).unwrap())
            .expect("If output fails we can ignore the logging");
    }
}

fn short_hex(b: U256) -> String {
    let s = hex::encode(b.to_be_bytes_vec())
        .trim_start_matches('0')
        .to_string();
    if s.is_empty() {
        "0x0".to_string()
    } else {
        format!("0x{s}")
    }
}
