use crate::{SpecId, B176, U256};
use bytes::Bytes;

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Env {
    pub cfg: CfgEnv,
    pub block: BlockEnv,
    pub tx: TxEnv,
}
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BlockEnv {
    pub number: U256,
    /// Coinbase or miner or address that created and signed the block.
    /// Address where we are going to send energy spend
    pub coinbase: B176,
    pub timestamp: U256,
    /// Difficulty is removed and not used after Paris (aka TheMerge). Value is replaced with prevrandao.
    pub difficulty: U256,
    /// basefee is added in EIP1559 London upgrade
    pub energy_limit: U256,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TxEnv {
    /// Caller or Author or tx signer
    pub caller: B176,
    pub energy_limit: u64,
    pub energy_price: U256,
    pub transact_to: TransactTo,
    pub value: U256,
    #[cfg_attr(feature = "serde", serde(with = "crate::utilities::serde_hex_bytes"))]
    pub data: Bytes,
    pub network_id: Option<u64>,
    pub nonce: Option<u64>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TransactTo {
    Call(B176),
    Create(CreateScheme),
}

impl TransactTo {
    pub fn create() -> Self {
        Self::Create(CreateScheme::Create)
    }
    pub fn is_create(&self) -> bool {
        matches!(self, Self::Create(_))
    }
}

/// Create scheme.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CreateScheme {
    /// Legacy create scheme of `CREATE`.
    Create,
    /// Create scheme of `CREATE2`.
    Create2 {
        /// Salt.
        salt: U256,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CfgEnv {
    pub network_id: u64,
    pub spec_id: SpecId,
    /// If all precompiles have some balance we can skip initially fetching them from the database.
    /// This is is not really needed on mainnet, and defaults to false, but in most cases it is
    /// safe to be set to `true`, depending on the chain.
    pub perf_all_precompiles_have_balance: bool,
    /// Bytecode that is created with CREATE/CREATE2 is by default analysed and jumptable is created.
    /// This is very benefitial for testing and speeds up execution of that bytecode if called multiple times.
    ///
    /// Default: Analyse
    pub perf_analyse_created_bytecodes: AnalysisKind,
    /// If some it will effects EIP-170: Contract code size limit. Usefull to increase this because of tests.
    /// By default it is 0x6000 (~25kb).
    pub limit_contract_code_size: Option<usize>,
    /// A hard memory limit in bytes beyond which [Memory] cannot be resized.
    ///
    /// In cases where the energy limit may be extraordinarily high, it is recommended to set this to
    /// a sane value to prevent memory allocation panics. Defaults to `2^32 - 1` bytes per
    /// EIP-1985.
    #[cfg(feature = "memory_limit")]
    pub memory_limit: u64,
    /// Skip balance checks if true. Adds transaction cost to balance to ensure execution doesn't fail.
    #[cfg(feature = "optional_balance_check")]
    pub disable_balance_check: bool,
    /// There are use cases where it's allowed to provide a energy limit that's higher than a block's energy limit. To that
    /// end, you can disable the block energy limit validation.
    /// By default, it is set to `false`.
    #[cfg(feature = "optional_block_energy_limit")]
    pub disable_block_energy_limit: bool,
    /// Disables all energy refunds. This is useful when using chains that have energy refunds disabled e.g. Avalanche.
    /// Reasoning behind removing energy refunds can be found in EIP-3298.
    /// By default, it is set to `false`.
    #[cfg(feature = "optional_energy_refund")]
    pub disable_energy_refund: bool,
}

#[derive(Clone, Default, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AnalysisKind {
    Raw,
    Check,
    #[default]
    Analyse,
}

impl Default for CfgEnv {
    fn default() -> CfgEnv {
        CfgEnv {
            network_id: 1,
            // For the CVM the target is Istanbul
            spec_id: SpecId::ISTANBUL,
            perf_all_precompiles_have_balance: false,
            perf_analyse_created_bytecodes: Default::default(),
            limit_contract_code_size: None,
            #[cfg(feature = "memory_limit")]
            memory_limit: 2u64.pow(32) - 1,
            #[cfg(feature = "optional_balance_check")]
            disable_balance_check: false,
            #[cfg(feature = "optional_block_energy_limit")]
            disable_block_energy_limit: false,
            #[cfg(feature = "optional_energy_refund")]
            disable_energy_refund: false,
        }
    }
}

impl CfgEnv {
    pub fn new_with_netowork_id(network_id: u64) -> CfgEnv {
        CfgEnv {
            network_id,
            // For the CVM the target is Istanbul
            spec_id: SpecId::ISTANBUL,
            perf_all_precompiles_have_balance: false,
            perf_analyse_created_bytecodes: Default::default(),
            limit_contract_code_size: None,
            #[cfg(feature = "memory_limit")]
            memory_limit: 2u64.pow(32) - 1,
            #[cfg(feature = "optional_balance_check")]
            disable_balance_check: false,
            #[cfg(feature = "optional_block_energy_limit")]
            disable_block_energy_limit: false,
            #[cfg(feature = "optional_energy_refund")]
            disable_energy_refund: false,
        }
    }
}

impl Default for BlockEnv {
    fn default() -> BlockEnv {
        BlockEnv {
            energy_limit: U256::MAX,
            number: U256::ZERO,
            coinbase: B176::zero(),
            timestamp: U256::from(1),
            difficulty: U256::ZERO,
        }
    }
}

impl Default for TxEnv {
    fn default() -> TxEnv {
        TxEnv {
            caller: B176::zero(),
            energy_limit: u64::MAX,
            energy_price: U256::ZERO,
            transact_to: TransactTo::Call(B176::zero()), //will do nothing
            value: U256::ZERO,
            data: Bytes::new(),
            network_id: None,
            nonce: None,
        }
    }
}

impl Env {
    pub fn effective_energy_price(&self) -> U256 {
        self.tx.energy_price
    }
}
