use crate::{Log, State, B176};
use bytes::Bytes;
use ruint::aliases::U256;

pub type EVMResult<DB> = core::result::Result<ResultAndState, EVMError<DB>>;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ResultAndState {
    /// Status of execution
    pub result: ExecutionResult,
    /// State that got updated
    pub state: State,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExecutionResult {
    /// Returned successfully
    Success {
        reason: Eval,
        energy_used: u64,
        energy_refunded: u64,
        logs: Vec<Log>,
        output: Output,
    },
    /// Reverted by `REVERT` opcode that doesn't spend all energy.
    Revert { energy_used: u64, output: Bytes },
    /// Reverted for various reasons and spend all energy.
    Halt {
        reason: Halt,
        /// Halting will spend all the energy, and will be equal to energy_limit.
        energy_used: u64,
    },
}

impl ExecutionResult {
    /// Returns if transaction execution is successful.
    /// 1 indicates success, 0 indicates revert.
    /// https://eips.ethereum.org/EIPS/eip-658
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// Return logs, if execution is not successful, function will return empty vec.
    pub fn logs(&self) -> Vec<Log> {
        match self {
            Self::Success { logs, .. } => logs.clone(),
            _ => Vec::new(),
        }
    }

    pub fn energy_used(&self) -> u64 {
        let (Self::Success { energy_used, .. }
        | Self::Revert { energy_used, .. }
        | Self::Halt { energy_used, .. }) = self;

        *energy_used
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Output {
    #[cfg_attr(feature = "serde", serde(with = "crate::utilities::serde_hex_bytes"))]
    Call(Bytes),
    Create(
        #[cfg_attr(feature = "serde", serde(with = "crate::utilities::serde_hex_bytes"))] Bytes,
        Option<B176>,
    ),
}

impl Output {
    /// Returns the output data of the execution output.
    pub fn into_data(self) -> Bytes {
        match self {
            Output::Call(data) => data,
            Output::Create(data, _) => data,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EVMError<DB> {
    Transaction(InvalidTransaction),
    /// REVM specific and related to environment.
    PrevrandaoNotSet,
    Database(DB),
}

impl<DB> From<InvalidTransaction> for EVMError<DB> {
    fn from(invalid: InvalidTransaction) -> Self {
        EVMError::Transaction(invalid)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InvalidTransaction {
    EnergyMaxFeeGreaterThanPriorityFee,
    EnergyPriceLessThanBasefee,
    CallerEnergyLimitMoreThanBlock,
    CallEnergyCostMoreThanEnergyLimit,
    /// EIP-3607 Reject transactions from senders with deployed code
    RejectCallerWithCode,
    /// Transaction account does not have enough amount of ether to cover transferred value and energy_limit*energy_price.
    LackOfFundForEnergyLimit {
        energy_limit: U256,
        balance: U256,
    },
    /// Overflow payment in transaction.
    OverflowPaymentInTransaction,
    /// Nonce overflows in transaction.
    NonceOverflowInTransaction,
    NonceTooHigh {
        tx: u64,
        state: u64,
    },
    NonceTooLow {
        tx: u64,
        state: u64,
    },
    /// EIP-3860: Limit and meter initcode
    CreateInitcodeSizeLimit,
    InvalidNetworkId,
}

/// When transaction return successfully without halts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Eval {
    Stop,
    Return,
    SelfDestruct,
}

/// Indicates that the EVM has experienced an exceptional halt. This causes execution to
/// immediately end with all energy being consumed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Halt {
    OutOfEnergy(OutOfEnergyError),
    OpcodeNotFound,
    InvalidFEOpcode,
    InvalidJump,
    NotActivated,
    StackUnderflow,
    StackOverflow,
    OutOfOffset,
    CreateCollision,
    PrecompileError,
    NonceOverflow,
    /// Create init code size exceeds limit (runtime).
    CreateContractSizeLimit,
    /// Error on created contract that begins with EF
    CreateContractStartingWithEF,
    /// EIP-3860: Limit and meter initcode. Initcode size limit exceeded.
    CreateInitcodeSizeLimit,

    /* Internal Halts that can be only found inside Inspector */
    OverflowPayment,
    StateChangeDuringStaticCall,
    CallNotAllowedInsideStatic,
    OutOfFund,
    CallTooDeep,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OutOfEnergyError {
    // Basic OOG error
    BasicOutOfEnergy,
    // Tried to expand past REVM limit
    MemoryLimit,
    // Basic OOG error from memory expansion
    Memory,
    // Precompile threw OOG error
    Precompile,
    // When performing something that takes a U256 and casts down to a u64, if its too large this would fire
    // i.e. in `as_usize_or_fail`
    InvalidOperand,
}
