use crate::primitives::{hash_map::Entry, Bytecode, Bytes, HashMap, U256};
use crate::{
    primitives::{Env, Log, B176, B256, SHA3_EMPTY},
    CallInputs, CreateInputs, Energy, Host, InstructionResult, Interpreter, SelfDestructResult,
};

pub struct DummyHost {
    pub env: Env,
    pub storage: HashMap<U256, U256>,
    pub log: Vec<Log>,
}

impl DummyHost {
    pub fn new(env: Env) -> Self {
        Self {
            env,
            storage: HashMap::new(),
            log: Vec::new(),
        }
    }
    pub fn clear(&mut self) {
        self.storage.clear();
        self.log.clear();
    }
}

impl Host for DummyHost {
    fn step(&mut self, _interp: &mut Interpreter, _is_static: bool) -> InstructionResult {
        InstructionResult::Continue
    }

    fn step_end(
        &mut self,
        _interp: &mut Interpreter,
        _is_static: bool,
        _ret: InstructionResult,
    ) -> InstructionResult {
        InstructionResult::Continue
    }

    fn env(&mut self) -> &mut Env {
        &mut self.env
    }

    fn load_account(&mut self, _address: B176) -> Option<(bool, bool)> {
        Some((true, true))
    }

    fn block_hash(&mut self, _number: U256) -> Option<B256> {
        Some(B256::zero())
    }

    fn balance(&mut self, _address: B176) -> Option<(U256, bool)> {
        Some((U256::ZERO, false))
    }

    fn code(&mut self, _address: B176) -> Option<(Bytecode, bool)> {
        Some((Bytecode::default(), false))
    }

    fn code_hash(&mut self, __address: B176) -> Option<(B256, bool)> {
        Some((SHA3_EMPTY, false))
    }

    fn sload(&mut self, __address: B176, index: U256) -> Option<(U256, bool)> {
        match self.storage.entry(index) {
            Entry::Occupied(entry) => Some((*entry.get(), false)),
            Entry::Vacant(entry) => {
                entry.insert(U256::ZERO);
                Some((U256::ZERO, true))
            }
        }
    }

    fn sstore(
        &mut self,
        _address: B176,
        index: U256,
        value: U256,
    ) -> Option<(U256, U256, U256, bool)> {
        let (present, is_cold) = match self.storage.entry(index) {
            Entry::Occupied(mut entry) => (entry.insert(value), false),
            Entry::Vacant(entry) => {
                entry.insert(value);
                (U256::ZERO, true)
            }
        };

        Some((U256::ZERO, present, value, is_cold))
    }

    fn log(&mut self, address: B176, topics: Vec<B256>, data: Bytes) {
        self.log.push(Log {
            address,
            topics,
            data,
        })
    }

    fn selfdestruct(&mut self, _address: B176, _target: B176) -> Option<SelfDestructResult> {
        panic!("Create is not supported for this host")
    }

    fn create(
        &mut self,
        _inputs: &mut CreateInputs,
    ) -> (InstructionResult, Option<B176>, Energy, Bytes) {
        panic!("Create is not supported for this host")
    }

    fn call(&mut self, _input: &mut CallInputs) -> (InstructionResult, Energy, Bytes) {
        panic!("Call is not supported for this host")
    }
}
