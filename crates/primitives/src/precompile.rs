use crate::Network;
use libgoldilocks::errors::LibgoldilockErrors;

/// A precompile operation result.
pub type PrecompileResult = Result<(u64, Vec<u8>), PrecompileError>;

pub type StandardPrecompileFn = fn(&[u8], u64, Network) -> PrecompileResult;
pub type CustomPrecompileFn = fn(&[u8], u64, Network) -> PrecompileResult;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrecompileError {
    /// out of energy is the main error. Other are just here for completness
    OutOfEnergy,
    // Blake2 erorr
    Blake2WrongLength,
    Blake2WrongFinalIndicatorFlag,
    // Modexp errors
    ModexpExpOverflow,
    ModexpBaseOverflow,
    ModexpModOverflow,
    // Bn128 errors
    Bn128FieldPointNotAMember,
    Bn128AffineGFailedToCreate,
    Bn128PairLength,

    EcrecoverDecodeError,
    EcrecoverDecodePubkeyError,
    EcrecoverDecodeSignatureError,
    EcrecoverInvalidLengthError,
    EcrecoverInvalidPrivKeyLengthError(usize),
    EcrecoverInvalidPubkeyLengthError,
    EcrecoverInvalidSignatureLengthError,
    EcrecoverInvalidSignatureError,
}

impl From<LibgoldilockErrors> for PrecompileError {
    fn from(value: LibgoldilockErrors) -> Self {
        match value {
            LibgoldilockErrors::DecodeError => PrecompileError::EcrecoverDecodeError,
            LibgoldilockErrors::DecodePubkeyError => PrecompileError::EcrecoverDecodePubkeyError,
            LibgoldilockErrors::DecodeSignatureError => {
                PrecompileError::EcrecoverDecodeSignatureError
            }
            LibgoldilockErrors::InvalidLengthError => PrecompileError::EcrecoverInvalidLengthError,
            LibgoldilockErrors::InvalidPrivKeyLengthErrro(size) => {
                PrecompileError::EcrecoverInvalidPrivKeyLengthError(size)
            }
            LibgoldilockErrors::InvalidPubkeyLengthError => {
                PrecompileError::EcrecoverInvalidPubkeyLengthError
            }
            LibgoldilockErrors::InvalidSignatureLengthError => {
                PrecompileError::EcrecoverInvalidSignatureLengthError
            }
            LibgoldilockErrors::InvalidSignatureError => {
                PrecompileError::EcrecoverInvalidSignatureError
            }
        }
    }
}
