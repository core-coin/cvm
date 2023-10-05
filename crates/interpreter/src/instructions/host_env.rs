use crate::{
    energy, interpreter::Interpreter, primitives::Spec, primitives::SpecId::*, Host,
    InstructionResult,
};

pub fn network_id<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    // EIP-1344: ChainID opcode
    check!(interpreter, SPEC::enabled(ISTANBUL));
    energy!(interpreter, energy::BASE);
    push!(interpreter, host.env().cfg.network.as_u256());
}

pub fn coinbase(interpreter: &mut Interpreter, host: &mut dyn Host) {
    energy!(interpreter, energy::BASE);
    push_b256!(interpreter, host.env().block.coinbase.into());
}

pub fn timestamp(interpreter: &mut Interpreter, host: &mut dyn Host) {
    energy!(interpreter, energy::BASE);
    push!(interpreter, host.env().block.timestamp);
}

pub fn number(interpreter: &mut Interpreter, host: &mut dyn Host) {
    energy!(interpreter, energy::BASE);
    push!(interpreter, host.env().block.number);
}

#[allow(clippy::extra_unused_type_parameters)]
pub fn difficulty<H: Host, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    energy!(interpreter, energy::BASE);
    push!(interpreter, host.env().block.difficulty);
}

pub fn gaslimit(interpreter: &mut Interpreter, host: &mut dyn Host) {
    energy!(interpreter, energy::BASE);
    push!(interpreter, host.env().block.energy_limit);
}

pub fn gasprice(interpreter: &mut Interpreter, host: &mut dyn Host) {
    energy!(interpreter, energy::BASE);
    push!(interpreter, host.env().effective_energy_price());
}

pub fn origin(interpreter: &mut Interpreter, host: &mut dyn Host) {
    energy!(interpreter, energy::BASE);
    push_b256!(interpreter, host.env().tx.caller.into());
}
