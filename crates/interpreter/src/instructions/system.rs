use crate::{
    energy,
    interpreter::Interpreter,
    primitives::{sha3 as hash_sha3, Spec, SpecId::*, B256, SHA3_EMPTY, U256},
    Host, InstructionResult,
};

pub fn sha3(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    pop!(interpreter, from, len);
    let len = as_usize_or_fail!(interpreter, len, InstructionResult::InvalidOperandOOG);
    energy_or_fail!(interpreter, energy::sha3_cost(len as u64));
    let hash = if len == 0 {
        SHA3_EMPTY
    } else {
        let from = as_usize_or_fail!(interpreter, from, InstructionResult::InvalidOperandOOG);
        memory_resize!(interpreter, from, len);
        hash_sha3(interpreter.memory.get_slice(from, len))
    };

    push_b256!(interpreter, hash);
}

pub fn address(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    energy!(interpreter, energy::BASE);
    push_b256!(interpreter, B256::from(interpreter.contract.address));
}

pub fn caller(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    energy!(interpreter, energy::BASE);
    push_b256!(interpreter, B256::from(interpreter.contract.caller));
}

pub fn codesize(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    energy!(interpreter, energy::BASE);
    push!(interpreter, U256::from(interpreter.contract.bytecode.len()));
}

pub fn codecopy(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    pop!(interpreter, memory_offset, code_offset, len);
    let len = as_usize_or_fail!(interpreter, len, InstructionResult::InvalidOperandOOG);
    energy_or_fail!(interpreter, energy::verylowcopy_cost(len as u64));
    if len == 0 {
        return;
    }
    let memory_offset = as_usize_or_fail!(
        interpreter,
        memory_offset,
        InstructionResult::InvalidOperandOOG
    );
    let code_offset = as_usize_saturated!(code_offset);
    memory_resize!(interpreter, memory_offset, len);

    // Safety: set_data is unsafe function and memory_resize ensures us that it is safe to call it
    interpreter.memory.set_data(
        memory_offset,
        code_offset,
        len,
        interpreter.contract.bytecode.original_bytecode_slice(),
    );
}

pub fn calldataload(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    energy!(interpreter, energy::VERYLOW);
    pop!(interpreter, index);
    let index = as_usize_saturated!(index);

    let load = if index < interpreter.contract.input.len() {
        let n = 32.min(interpreter.contract.input.len() - index);
        let mut bytes = [0u8; 32];
        // SAFETY: n <= len - index -> index + n <= len
        let src = unsafe { interpreter.contract.input.get_unchecked(index..index + n) };
        bytes[..n].copy_from_slice(src);
        U256::from_be_bytes(bytes)
    } else {
        U256::ZERO
    };

    push!(interpreter, load);
}

pub fn calldatasize(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    energy!(interpreter, energy::BASE);
    push!(interpreter, U256::from(interpreter.contract.input.len()));
}

pub fn callvalue(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    energy!(interpreter, energy::BASE);
    push!(interpreter, interpreter.contract.value);
}

pub fn calldatacopy(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    pop!(interpreter, memory_offset, data_offset, len);
    let len = as_usize_or_fail!(interpreter, len, InstructionResult::InvalidOperandOOG);
    energy_or_fail!(interpreter, energy::verylowcopy_cost(len as u64));
    if len == 0 {
        return;
    }
    let memory_offset = as_usize_or_fail!(
        interpreter,
        memory_offset,
        InstructionResult::InvalidOperandOOG
    );
    let data_offset = as_usize_saturated!(data_offset);
    memory_resize!(interpreter, memory_offset, len);

    // Safety: set_data is unsafe function and memory_resize ensures us that it is safe to call it
    interpreter
        .memory
        .set_data(memory_offset, data_offset, len, &interpreter.contract.input);
}

pub fn returndatasize<SPEC: Spec>(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    energy!(interpreter, energy::BASE);
    // EIP-211: New opcodes: RETURNDATASIZE and RETURNDATACOPY
    check!(interpreter, SPEC::enabled(BYZANTIUM));
    push!(
        interpreter,
        U256::from(interpreter.return_data_buffer.len())
    );
}

pub fn returndatacopy<SPEC: Spec>(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    // EIP-211: New opcodes: RETURNDATASIZE and RETURNDATACOPY
    check!(interpreter, SPEC::enabled(BYZANTIUM));
    pop!(interpreter, memory_offset, offset, len);
    let len = as_usize_or_fail!(interpreter, len, InstructionResult::InvalidOperandOOG);
    energy_or_fail!(interpreter, energy::verylowcopy_cost(len as u64));
    let data_offset = as_usize_saturated!(offset);
    let (data_end, overflow) = data_offset.overflowing_add(len);
    if overflow || data_end > interpreter.return_data_buffer.len() {
        interpreter.instruction_result = InstructionResult::OutOfOffset;
        return;
    }
    if len != 0 {
        let memory_offset = as_usize_or_fail!(
            interpreter,
            memory_offset,
            InstructionResult::InvalidOperandOOG
        );
        memory_resize!(interpreter, memory_offset, len);
        interpreter.memory.set(
            memory_offset,
            &interpreter.return_data_buffer[data_offset..data_end],
        );
    }
}

pub fn energy(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    energy!(interpreter, energy::BASE);
    push!(interpreter, U256::from(interpreter.energy.remaining()));
}
