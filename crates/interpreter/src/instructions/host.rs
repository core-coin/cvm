use crate::primitives::{Bytes, Spec, SpecId::*, B176, B256, U256};
use crate::{
    alloc::vec::Vec,
    energy::{self},
    interpreter::Interpreter,
    return_ok, return_revert, CallContext, CallInputs, CallScheme, CreateInputs, CreateScheme,
    Host, InstructionResult, Transfer,
};
use core::cmp::min;

pub fn balance<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    pop_address!(interpreter, address);
    let ret = host.balance(address);
    if ret.is_none() {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    }
    let (balance, _is_cold) = ret.unwrap();
    energy!(
        interpreter,
        if SPEC::enabled(ISTANBUL) {
            // EIP-1884: Repricing for trie-size-dependent opcodes
            energy::account_access_energy::<SPEC>()
        } else if SPEC::enabled(TANGERINE) {
            400
        } else {
            20
        }
    );
    push!(interpreter, balance);
}

pub fn selfbalance<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    // EIP-1884: Repricing for trie-size-dependent opcodes
    check!(interpreter, SPEC::enabled(ISTANBUL));
    energy!(interpreter, energy::LOW);
    let ret = host.balance(interpreter.contract.address);
    if ret.is_none() {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    }
    let (balance, _) = ret.unwrap();
    push!(interpreter, balance);
}

pub fn extcodesize<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    pop_address!(interpreter, address);
    let ret = host.code(address);
    if ret.is_none() {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    }
    let (code, _is_cold) = ret.unwrap();

    if SPEC::enabled(TANGERINE) {
        energy!(interpreter, 700);
    } else {
        energy!(interpreter, 20);
    }

    push!(interpreter, U256::from(code.len()));
}

pub fn extcodehash<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    check!(interpreter, SPEC::enabled(CONSTANTINOPLE)); // EIP-1052: EXTCODEHASH opcode
    pop_address!(interpreter, address);
    let ret = host.code_hash(address);
    if ret.is_none() {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    }
    let (code_hash, _is_cold) = ret.unwrap();
    if SPEC::enabled(ISTANBUL) {
        energy!(interpreter, 700);
    } else {
        energy!(interpreter, 400);
    }
    push_b256!(interpreter, code_hash);
}

pub fn extcodecopy<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    pop_address!(interpreter, address);
    pop!(interpreter, memory_offset, code_offset, len_u256);

    let ret = host.code(address);
    if ret.is_none() {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    }
    let (code, _is_cold) = ret.unwrap();

    let len = as_usize_or_fail!(interpreter, len_u256, InstructionResult::InvalidOperandOOG);
    energy_or_fail!(interpreter, energy::extcodecopy_cost::<SPEC>(len as u64));
    if len == 0 {
        return;
    }
    let memory_offset = as_usize_or_fail!(
        interpreter,
        memory_offset,
        InstructionResult::InvalidOperandOOG
    );
    let code_offset = min(as_usize_saturated!(code_offset), code.len());
    memory_resize!(interpreter, memory_offset, len);

    // Safety: set_data is unsafe function and memory_resize ensures us that it is safe to call it
    interpreter
        .memory
        .set_data(memory_offset, code_offset, len, code.bytes());
}

pub fn blockhash(interpreter: &mut Interpreter, host: &mut dyn Host) {
    energy!(interpreter, energy::BLOCKHASH);
    pop_top!(interpreter, number);

    if let Some(diff) = host.env().block.number.checked_sub(*number) {
        let diff = as_usize_saturated!(diff);
        // blockhash should push zero if number is same as current block number.
        if diff <= 256 && diff != 0 {
            let ret = host.block_hash(*number);
            if ret.is_none() {
                interpreter.instruction_result = InstructionResult::FatalExternalError;
                return;
            }
            *number = U256::from_be_bytes(*ret.unwrap());
            return;
        }
    }
    *number = U256::ZERO;
}

pub fn sload<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    pop!(interpreter, index);

    let ret = host.sload(interpreter.contract.address, index);
    if ret.is_none() {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    }
    let (value, is_cold) = ret.unwrap();
    energy!(interpreter, energy::sload_cost::<SPEC>(is_cold));
    push!(interpreter, value);
}

pub fn sstore<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    check_staticcall!(interpreter);

    pop!(interpreter, index, value);
    let ret = host.sstore(interpreter.contract.address, index, value);
    if ret.is_none() {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    }
    let (original, old, new, is_cold) = ret.unwrap();
    energy_or_fail!(interpreter, {
        let remaining_energy = interpreter.energy.remaining();
        energy::sstore_cost::<SPEC>(original, old, new, remaining_energy, is_cold)
    });
    refund!(
        interpreter,
        energy::sstore_refund::<SPEC>(original, old, new)
    );
}

pub fn log<const N: u8>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    check_staticcall!(interpreter);

    pop!(interpreter, offset, len);
    let len = as_usize_or_fail!(interpreter, len, InstructionResult::InvalidOperandOOG);
    energy_or_fail!(interpreter, energy::log_cost(N, len as u64));
    let data = if len == 0 {
        Bytes::new()
    } else {
        let offset = as_usize_or_fail!(interpreter, offset, InstructionResult::InvalidOperandOOG);
        memory_resize!(interpreter, offset, len);
        Bytes::copy_from_slice(interpreter.memory.get_slice(offset, len))
    };
    let n = N as usize;
    if interpreter.stack.len() < n {
        interpreter.instruction_result = InstructionResult::StackUnderflow;
        return;
    }

    let mut topics = Vec::with_capacity(n);
    for _ in 0..(n) {
        // Safety: stack bounds already checked few lines above
        topics.push(B256(unsafe {
            interpreter.stack.pop_unsafe().to_be_bytes()
        }));
    }

    host.log(interpreter.contract.address, topics, data);
}

pub fn selfdestruct<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    check_staticcall!(interpreter);
    pop_address!(interpreter, target);

    let res = host.selfdestruct(interpreter.contract.address, target);
    if res.is_none() {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    }
    let res = res.unwrap();

    energy!(interpreter, energy::selfdestruct_cost::<SPEC>(res));

    interpreter.instruction_result = InstructionResult::SelfDestruct;
}

pub fn create<const IS_CREATE2: bool, SPEC: Spec>(
    interpreter: &mut Interpreter,
    host: &mut dyn Host,
) {
    check_staticcall!(interpreter);
    if IS_CREATE2 {
        // EIP-1014: Skinny CREATE2
        check!(interpreter, SPEC::enabled(PETERSBURG));
    }

    interpreter.return_data_buffer = Bytes::new();

    pop!(interpreter, value, code_offset, len);
    let len = as_usize_or_fail!(interpreter, len, InstructionResult::InvalidOperandOOG);

    let code = if len == 0 {
        Bytes::new()
    } else {
        let code_offset = as_usize_or_fail!(
            interpreter,
            code_offset,
            InstructionResult::InvalidOperandOOG
        );
        memory_resize!(interpreter, code_offset, len);
        Bytes::copy_from_slice(interpreter.memory.get_slice(code_offset, len))
    };

    let scheme = if IS_CREATE2 {
        pop!(interpreter, salt);
        energy_or_fail!(interpreter, energy::create2_cost(len));
        CreateScheme::Create2 { salt }
    } else {
        energy!(interpreter, energy::CREATE);
        CreateScheme::Create
    };

    let mut energy_limit = interpreter.energy().remaining();

    // EIP-150: Energy cost changes for IO-heavy operations
    if SPEC::enabled(TANGERINE) {
        // take remaining energy and deduce l64 part of it.
        energy_limit -= energy_limit / 64
    }
    energy!(interpreter, energy_limit);

    let mut create_input = CreateInputs {
        caller: interpreter.contract.address,
        scheme,
        value,
        init_code: code,
        energy_limit,
    };

    let (return_reason, address, energy, return_data) = host.create(&mut create_input);
    interpreter.return_data_buffer = match return_reason {
        // Save data to return data buffer if the create reverted
        return_revert!() => return_data,
        // Otherwise clear it
        _ => Bytes::new(),
    };

    match return_reason {
        return_ok!() => {
            push_b256!(interpreter, address.unwrap_or_default().into());
            if crate::USE_ENERGY {
                interpreter.energy.erase_cost(energy.remaining());
                interpreter.energy.record_refund(energy.refunded());
            }
        }
        return_revert!() => {
            push_b256!(interpreter, B256::zero());
            if crate::USE_ENERGY {
                interpreter.energy.erase_cost(energy.remaining());
            }
        }
        InstructionResult::FatalExternalError => {
            interpreter.instruction_result = InstructionResult::FatalExternalError;
        }
        _ => {
            push_b256!(interpreter, B256::zero());
        }
    }
}

pub fn call<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    call_inner::<SPEC>(interpreter, CallScheme::Call, host);
}

pub fn call_code<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    call_inner::<SPEC>(interpreter, CallScheme::CallCode, host);
}

pub fn delegate_call<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    call_inner::<SPEC>(interpreter, CallScheme::DelegateCall, host);
}

pub fn static_call<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    call_inner::<SPEC>(interpreter, CallScheme::StaticCall, host);
}

pub fn call_inner<SPEC: Spec>(
    interpreter: &mut Interpreter,
    scheme: CallScheme,
    host: &mut dyn Host,
) {
    match scheme {
        CallScheme::DelegateCall => check!(interpreter, SPEC::enabled(HOMESTEAD)), // EIP-7: DELEGATECALL
        CallScheme::StaticCall => check!(interpreter, SPEC::enabled(BYZANTIUM)), // EIP-214: New opcode STATICCALL
        _ => (),
    }
    interpreter.return_data_buffer = Bytes::new();

    pop!(interpreter, local_energy_limit);
    pop_address!(interpreter, to);
    let local_energy_limit = u64::try_from(local_energy_limit).unwrap_or(u64::MAX);

    let value = match scheme {
        CallScheme::CallCode => {
            pop!(interpreter, value);
            value
        }
        CallScheme::Call => {
            pop!(interpreter, value);
            if interpreter.is_static && value != U256::ZERO {
                interpreter.instruction_result = InstructionResult::CallNotAllowedInsideStatic;
                return;
            }
            value
        }
        CallScheme::DelegateCall | CallScheme::StaticCall => U256::ZERO,
    };

    pop!(interpreter, in_offset, in_len, out_offset, out_len);

    let in_len = as_usize_or_fail!(interpreter, in_len, InstructionResult::InvalidOperandOOG);
    let input = if in_len != 0 {
        let in_offset =
            as_usize_or_fail!(interpreter, in_offset, InstructionResult::InvalidOperandOOG);
        memory_resize!(interpreter, in_offset, in_len);
        Bytes::copy_from_slice(interpreter.memory.get_slice(in_offset, in_len))
    } else {
        Bytes::new()
    };

    let out_len = as_usize_or_fail!(interpreter, out_len, InstructionResult::InvalidOperandOOG);
    let out_offset = if out_len != 0 {
        let out_offset = as_usize_or_fail!(
            interpreter,
            out_offset,
            InstructionResult::InvalidOperandOOG
        );
        memory_resize!(interpreter, out_offset, out_len);
        out_offset
    } else {
        usize::MAX //unrealistic value so we are sure it is not used
    };

    let context = match scheme {
        CallScheme::Call | CallScheme::StaticCall => CallContext {
            address: to,
            caller: interpreter.contract.address,
            code_address: to,
            apparent_value: value,
            scheme,
        },
        CallScheme::CallCode => CallContext {
            address: interpreter.contract.address,
            caller: interpreter.contract.address,
            code_address: to,
            apparent_value: value,
            scheme,
        },
        CallScheme::DelegateCall => CallContext {
            address: interpreter.contract.address,
            caller: interpreter.contract.caller,
            code_address: to,
            apparent_value: interpreter.contract.value,
            scheme,
        },
    };

    let transfer = if scheme == CallScheme::Call {
        Transfer {
            source: interpreter.contract.address,
            target: to,
            value,
        }
    } else if scheme == CallScheme::CallCode {
        Transfer {
            source: interpreter.contract.address,
            target: interpreter.contract.address,
            value,
        }
    } else {
        //this is dummy send for StaticCall and DelegateCall, it should do nothing and dont touch anything.
        Transfer {
            source: interpreter.contract.address,
            target: interpreter.contract.address,
            value: U256::ZERO,
        }
    };

    // load account and calculate energy cost.
    let res = host.load_account(to);
    if res.is_none() {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    }
    let (is_cold, exist) = res.unwrap();
    let is_new = !exist;

    energy!(
        interpreter,
        energy::call_cost::<SPEC>(
            value,
            is_new,
            is_cold,
            matches!(scheme, CallScheme::Call | CallScheme::CallCode),
            matches!(scheme, CallScheme::Call | CallScheme::StaticCall),
        )
    );

    // take l64 part of energy_limit
    let mut energy_limit = if SPEC::enabled(TANGERINE) {
        //EIP-150: Energy cost changes for IO-heavy operations
        let energy = interpreter.energy().remaining();
        min(energy - energy / 64, local_energy_limit)
    } else {
        local_energy_limit
    };

    energy!(interpreter, energy_limit);

    // add call stipend if there is value to be transferred.
    if matches!(scheme, CallScheme::Call | CallScheme::CallCode) && transfer.value != U256::ZERO {
        energy_limit = energy_limit.saturating_add(energy::CALL_STIPEND);
    }
    let is_static = matches!(scheme, CallScheme::StaticCall) || interpreter.is_static;

    let mut call_input = CallInputs {
        contract: to,
        transfer,
        input,
        energy_limit,
        context,
        is_static,
    };

    // Call host to interuct with target contract
    println!("cvm call input: {:#?}", call_input.input);
    let (reason, energy, return_data) = host.call(&mut call_input);
    println!("cvm call res: {:#?}", reason);

    interpreter.return_data_buffer = return_data;

    let target_len = min(out_len, interpreter.return_data_buffer.len());

    match reason {
        return_ok!() => {
            // return unspend energy.
            if crate::USE_ENERGY {
                interpreter.energy.erase_cost(energy.remaining());
                interpreter.energy.record_refund(energy.refunded());
            }
            interpreter
                .memory
                .set(out_offset, &interpreter.return_data_buffer[..target_len]);
            push!(interpreter, U256::from(1));
        }
        return_revert!() => {
            if crate::USE_ENERGY {
                interpreter.energy.erase_cost(energy.remaining());
            }
            interpreter
                .memory
                .set(out_offset, &interpreter.return_data_buffer[..target_len]);
            push!(interpreter, U256::ZERO);
        }
        InstructionResult::FatalExternalError => {
            interpreter.instruction_result = InstructionResult::FatalExternalError;
        }
        _ => {
            push!(interpreter, U256::ZERO);
        }
    }
}
