use revm::primitives::AccountInfo;
use revm::{db::InMemoryDB, primitives::TransactTo};
use revm::{
    primitives::{B176, U256},
    EVM,
};
use std::str::FromStr;

#[test]
fn test_send_ican() {
    // Create AccountInfo with balance for gas
    let account = AccountInfo {
        balance: U256::from_str("10000000000000000000000").unwrap(),
        ..Default::default()
    };

    // initialise an empty (default) EVM
    let mut evm = EVM::new();

    evm.env.cfg.chain_id = 1;

    // initialise the database
    evm.database(InMemoryDB::default());

    // Assign the balance to the zero address
    evm.db.as_mut().unwrap().insert_account_info(
        B176::from_str("0x00000000000000000000000000000000000000000000").unwrap(),
        account,
    );

    // Caller is the 0 address
    evm.env.tx.caller = B176::from_str("0x00000000000000000000000000000000000000000000").unwrap();
    // Account we want to transact with
    evm.env.tx.transact_to =
        TransactTo::Call(B176::from_str("0x00000000000000000000000000000000000000000002").unwrap());
    // transaction value in wei
    evm.env.tx.value = U256::from_str("100000000").unwrap();

    // execute transaction and write it to the db
    let _ = evm.transact_commit().unwrap();

    // Get the balance of the 0x02 account
    let balance = evm
        .db
        .unwrap()
        .accounts
        .get(&B176::from_str("0x00000000000000000000000000000000000000000002").unwrap())
        .unwrap()
        .info
        .balance;

    // Make sure it is the same amount as we sent
    assert_eq!(balance, U256::from(100000000));
}
