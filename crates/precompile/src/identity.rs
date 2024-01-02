use revm_primitives::Network;

use super::calc_linear_cost_u32;
use crate::{Error, Precompile, PrecompileAddress, PrecompileResult, StandardPrecompileFn};

pub const FUN: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b176(4),
    Precompile::Standard(identity_run as StandardPrecompileFn),
);

/// The base cost of the operation.
const IDENTITY_BASE: u64 = 15;
/// The cost per word.
const IDENTITY_PER_WORD: u64 = 3;

/// Takes the input bytes, copies them, and returns it as the output.
///
/// See: https://ethereum.github.io/yellowpaper/paper.pdf
/// See: https://etherscan.io/address/0000000000000000000000000000000000000004
fn identity_run(input: &[u8], energy_limit: u64, _: Network) -> PrecompileResult {
    let energy_used = calc_linear_cost_u32(input.len(), IDENTITY_BASE, IDENTITY_PER_WORD);
    if energy_used > energy_limit {
        return Err(Error::OutOfEnergy);
    }
    Ok((energy_used, input.to_vec()))
}
