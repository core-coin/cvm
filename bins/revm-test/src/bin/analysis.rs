use std::time::Instant;

use bytes::Bytes;
use cvm::{
    db::BenchmarkDB,
    interpreter::analysis::to_analysed,
    primitives::{Bytecode, TransactTo},
};

fn main() {
    let contract_data : Bytes = hex::decode("604c8060093d393df360003560e01c80633e58c58c14610011575b60043575ffffffffffffffffffffffffffffffffffffffffffff16600060006000600034855af16001146100455760006000fd5b0060006000fd").unwrap().into();

    // BenchmarkDB is dummy state that implements Database trait.
    let mut cvm = cvm::new();

    // execution globals block hash/energy_limit/coinbase/timestamp..
    cvm.env.tx.caller = "0x10000000000000000000000000000000000000000000"
        .parse()
        .unwrap();
    cvm.env.tx.transact_to = TransactTo::Call(
        "0x00000000000000000000000000000000000000000000"
            .parse()
            .unwrap(),
    );
    //cvm.env.tx.data = Bytes::from(hex::decode("30627b7c").unwrap());
    cvm.env.tx.data = Bytes::from(
        hex::decode("3e58c58c00000000000000000000ffffffffffffffffffffffffffffffffffffffffffff")
            .unwrap(),
    );
    cvm.env.cfg.perf_all_precompiles_have_balance = true;

    let bytecode_raw = Bytecode::new_raw(contract_data.clone());
    let bytecode_checked = Bytecode::new_raw(contract_data.clone()).to_checked();
    let bytecode_analysed = to_analysed(Bytecode::new_raw(contract_data));

    cvm.database(BenchmarkDB::new_bytecode(bytecode_raw));

    // just to spead up processor.
    for _ in 0..10000 {
        let _ = cvm.transact().unwrap();
    }

    let timer = Instant::now();
    for _ in 0..30000 {
        let _ = cvm.transact().unwrap();
    }
    println!("Raw elapsed time: {:?}", timer.elapsed());

    cvm.database(BenchmarkDB::new_bytecode(bytecode_checked));

    let timer = Instant::now();
    for _ in 0..30000 {
        let _ = cvm.transact().unwrap();
    }
    println!("Checked elapsed time: {:?}", timer.elapsed());

    cvm.database(BenchmarkDB::new_bytecode(bytecode_analysed));

    let timer = Instant::now();
    for _ in 0..30000 {
        let _ = cvm.transact().unwrap();
    }
    println!("Analysed elapsed time: {:?}", timer.elapsed());
}
