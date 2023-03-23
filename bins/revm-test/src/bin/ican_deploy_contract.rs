use bytes::Bytes;
use revm::{
    db::InMemoryDB,
    primitives::{AccountInfo, ExecutionResult, Output, TransactTo, TxEnv, B176, U256},
    EVM,
};
use std::str::FromStr;

fn main() {
    // Contract bytecode -> bins/revm-test/src/bin/Huff/Send.huff
    let contract_data: Bytes = hex::decode("604c8060093d393df360003560e01c80633e58c58c14610011575b60043575ffffffffffffffffffffffffffffffffffffffffffff16600060006000600034855af16001146100455760006000fd5b0060006000fd").unwrap().into();

    // Create new EVM
    let mut evm = EVM::new();

    // Create new in memory database
    evm.database(InMemoryDB::default());

    // create AccountInfo with balance for gas
    let account_user = AccountInfo {
        balance: U256::from_str("10000000000000000000000").unwrap(),
        ..Default::default()
    };

    // Assign the AccountInfo to 0x02 address
    evm.db.as_mut().unwrap().insert_account_info(
        B176::from_str("0x00000000000000000000000000000000000000000002").unwrap(),
        account_user,
    );

    // Transaction caller is 0x02
    evm.env.tx.caller = B176::from_str("0x00000000000000000000000000000000000000000002").unwrap();

    // account you want to transact with
    // Noramlly this would just be the zero address but REVM needs it explicitly
    evm.env.tx.transact_to = TransactTo::Create(revm::primitives::CreateScheme::Create);

    // Bytecode to deploy
    evm.env.tx.data = contract_data;

    // Transact and write to database
    let s = evm.transact_commit().unwrap();

    let addr;
    if let ExecutionResult::Success {
        output: Output::Create(_, Some(address)),
        ..
    } = s
    {
        // Get the deployed address
        addr = address;
    } else {
        // If the pattern doesn't match it means the deployment failed
        panic!("Deployment failed");
    }

    // Create new transaction
    let tx = TxEnv {
        caller: B176::from_str("0x00000000000000000000000000000000000000000002").unwrap(),
        // We call the deployed contract
        transact_to: TransactTo::Call(addr),
        // Custom encoded calldata
        // selector: 3e58c58c = keccak256(send(address))
        // address: 00000000000000000000ffffffffffffffffffffffffffffffffffffffffffff with left padding
        data: Bytes::from(
            hex::decode("3e58c58c00000000000000000000ffffffffffffffffffffffffffffffffffffffffffff")
                .unwrap(),
        ),

        // Value we want to send
        value: U256::from_str("10000000").unwrap(),

        // Rest is default
        ..Default::default()
    };

    // Assign the tx
    evm.env.tx = tx;

    // Transact and write to database
    let _ = evm.transact_commit().unwrap();

    // Get balance of the 0xff account
    let balance = evm
        .db
        .unwrap()
        .accounts
        .get(&B176::from_str("0xffffffffffffffffffffffffffffffffffffffffffff").unwrap())
        .unwrap()
        .info
        .balance;

    // Make sure it is the same amount we sent earlier at line 70
    assert_eq!(balance, U256::from(10000000));
}
