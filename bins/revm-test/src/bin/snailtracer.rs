use std::time::Duration;

use bytes::Bytes;
use revm::{
    db::BenchmarkDB,
    interpreter::{analysis::to_analysed, BytecodeLocked, Contract, DummyHost, Interpreter},
    primitives::{Bytecode, IstanbulSpec, TransactTo},
};
extern crate alloc;

pub fn simple_example() {
    let contract_data : Bytes = hex::decode("604c8060093d393df360003560e01c80633e58c58c14610011575b60043575ffffffffffffffffffffffffffffffffffffffffffff16600060006000600034855af16001146100455760006000fd5b0060006000fd").unwrap().into();

    // BenchmarkDB is dummy state that implements Database trait.
    let mut evm = revm::new();
    let bytecode = to_analysed(Bytecode::new_raw(contract_data));
    evm.database(BenchmarkDB::new_bytecode(bytecode.clone()));

    // execution globals block hash/energy_limit/coinbase/timestamp..
    evm.env.tx.caller = "0x10000000000000000000000000000000000000000000"
        .parse()
        .unwrap();
    evm.env.tx.transact_to = TransactTo::Call(
        "0x00000000000000000000000000000000000000000000"
            .parse()
            .unwrap(),
    );
    evm.env.tx.data = Bytes::from(
        hex::decode("3e58c58c00000000000000000000ffffffffffffffffffffffffffffffffffffffffffff")
            .unwrap(),
    );

    // Microbenchmark
    let bench_options = microbench::Options::default().time(Duration::from_secs(2));

    let env = evm.env.clone();
    microbench::bench(
        &bench_options,
        "Snailtracer Host+Interpreter benchmark",
        || {
            let _ = evm.transact().unwrap();
        },
    );

    // revm interpreter
    let contract = Contract {
        input: evm.env.tx.data,
        bytecode: BytecodeLocked::try_from(bytecode).unwrap(),
        ..Default::default()
    };

    let mut host = DummyHost::new(env);
    microbench::bench(&bench_options, "Snailtracer Interpreter benchmark", || {
        let mut interpreter = Interpreter::new(contract.clone(), u64::MAX, false);
        interpreter.run::<_, IstanbulSpec>(&mut host);
        host.clear()
    });
}

fn main() {
    println!("Running snailtracer bench!");
    simple_example();
    println!("end!");
}
