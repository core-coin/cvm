use std::str::FromStr;

use bytes::Bytes;
use revm::primitives::{Env, TransactTo, B176, U256};
use structopt::StructOpt;

#[derive(StructOpt, Clone, Debug)]
pub struct CliEnv {
    #[structopt(flatten)]
    block: CliEnvBlock,
    #[structopt(flatten)]
    tx: CliEnvTx,
}

macro_rules! local_fill {
    ($left:expr, $right:expr, $fun:expr) => {
        if let Some(right) = $right {
            $left = $fun(right)
        }
    };
    ($left:expr, $right:expr) => {
        if let Some(right) = $right {
            $left = right
        }
    };
}

impl From<CliEnv> for Env {
    fn from(from: CliEnv) -> Self {
        let mut env = Env::default();
        local_fill!(
            env.block.energy_limit,
            from.block.block_energy_limit,
            U256::from
        );
        local_fill!(env.block.number, from.block.number, U256::from);
        local_fill!(env.block.coinbase, from.block.coinbase);
        local_fill!(env.block.timestamp, from.block.timestamp, U256::from);
        local_fill!(env.block.difficulty, from.block.difficulty, U256::from);

        local_fill!(env.tx.caller, from.tx.caller);
        local_fill!(env.tx.energy_limit, from.tx.tx_energy_limit);
        local_fill!(env.tx.value, from.tx.value, U256::from);
        local_fill!(env.tx.data, from.tx.data);
        env.tx.network_id = from.tx.network_id;
        env.tx.nonce = from.tx.nonce;

        env.tx.transact_to = if let Some(to) = from.tx.transact_to {
            TransactTo::Call(to)
        } else {
            TransactTo::create()
        };
        //TODO tx access_list

        env
    }
}

#[derive(StructOpt, Clone, Debug)]
pub struct CliEnvBlock {
    #[structopt(long = "env.block.energy_limit")]
    pub block_energy_limit: Option<u64>,
    /// somebody call it nonce
    #[structopt(long = "env.block.number")]
    pub number: Option<u64>,
    /// Coinbase or miner or address that created and signed the block.
    /// Address where we are going to send energy spend
    #[structopt(long = "env.block.coinbase", parse(try_from_str = parse_b176))]
    pub coinbase: Option<B176>,
    #[structopt(long = "env.block.timestamp")]
    pub timestamp: Option<u64>,
    #[structopt(long = "env.block.difficulty")]
    pub difficulty: Option<u64>,
}

#[derive(StructOpt, Clone, Debug)]
pub struct CliEnvTx {
    /// Caller or Author or tx signer
    #[structopt(long = "env.tx.caller", parse(try_from_str = parse_b176))]
    pub caller: Option<B176>,
    #[structopt(long = "env.tx.energy_limit")]
    pub tx_energy_limit: Option<u64>,
    #[structopt(long = "env.tx.energy_price")]
    pub energy_price: Option<u64>,
    #[structopt(long = "env.tx.energy_priority_fee")]
    pub energy_priority_fee: Option<u64>,
    #[structopt(long = "env.tx.to", parse(try_from_str = parse_b176))]
    pub transact_to: Option<B176>,
    #[structopt(long = "env.tx.value")]
    pub value: Option<u64>,
    #[structopt(long = "env.tx.data", parse(try_from_str = parse_hex))]
    pub data: Option<Bytes>,
    #[structopt(long = "env.tx.chain_id")]
    pub network_id: Option<u64>,
    #[structopt(long = "env.tx.nonce")]
    pub nonce: Option<u64>,
    //#[structopt(long = "env.")]
    //TODO pub access_list: Vec<(B176, Vec<U256>)>,
}

fn parse_hex(src: &str) -> Result<Bytes, hex::FromHexError> {
    Ok(Bytes::from(hex::decode(src)?))
}

pub fn parse_b176(input: &str) -> Result<B176, <B176 as FromStr>::Err> {
    B176::from_str(input)
}
