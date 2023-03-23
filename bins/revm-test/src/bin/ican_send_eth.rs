use bytes::Bytes;
use revm::primitives::AccountInfo;
use revm::{db::InMemoryDB, primitives::TransactTo};
use revm::{
    primitives::{B176, U256},
    EVM,
};
use std::str::FromStr;

fn main() {
    let account = AccountInfo {
        balance: U256::from_str("10000000000000000000000").unwrap(),
        ..Default::default()
    };

    // initialise an empty (default) EVM
    let mut evm = EVM::new();
    // let bytecode: Bytes = hex::decode("123").unwrap().into();
    // let bytecode = to_analysed(Bytecode::new_raw(bytecode));
    // let bytecode = Bytecode::from(bytecode);
    evm.database(InMemoryDB::default());
    evm.db.as_mut().unwrap().insert_account_info(
        B176::from_str("0x00000000000000000000000000000000000000000000").unwrap(),
        account,
    );

    evm.env.tx.caller = B176::from_str("0x00000000000000000000000000000000000000000000").unwrap();
    // account you want to transact with
    evm.env.tx.transact_to =
        TransactTo::Call(B176::from_str("0x00000000000000000000000000000000000000000002").unwrap());
    // calldata formed via abigen
    evm.env.tx.data = Bytes::default();
    // transaction value in wei
    evm.env.tx.value = U256::from_str("1000000000000000000").unwrap();

    // execute transaction without writing to the DB
    let _ = evm.transact_commit().unwrap();
    // select ExecutionResult struct

    let state = evm.clone().db.unwrap();
    let state = state
        .accounts
        .get(&B176::from_str("0x00000000000000000000000000000000000000000002").unwrap());

    println!("STATE: {:#?}", state);
    println!("{:#?}", evm.env);
}
