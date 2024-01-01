use crate::interpreter::{
    analysis::to_analysed, energy, instruction_result::SuccessOrHalt, return_ok, return_revert,
    CallContext, CallInputs, CallScheme, Contract, CreateInputs, CreateScheme, Host,
    InstructionResult, Interpreter, SelfDestructResult, Transfer, CALL_STACK_LIMIT,
};
use crate::primitives::{
    create2_address, create_address, sha3, Account, AnalysisKind, Bytecode, Bytes, EVMError,
    EVMResult, Env, ExecutionResult, HashMap, InvalidTransaction, Log, Output, ResultAndState,
    Spec,
    SpecId::{self, *},
    TransactTo, B176, B256, SHA3_EMPTY, U256,
};
use crate::{db::Database, journaled_state::JournaledState, precompile, Inspector};
use alloc::vec::Vec;
use core::{cmp::min, marker::PhantomData};
use revm_interpreter::energy::Energy;
use revm_interpreter::primitives::Network;
use revm_interpreter::MAX_CODE_SIZE;
use revm_precompile::{Precompile, Precompiles};
use std::cmp::Ordering;

pub struct EVMData<'a, DB: Database> {
    pub env: &'a mut Env,
    pub journaled_state: JournaledState,
    pub db: &'a mut DB,
    pub error: Option<DB::Error>,
}

pub struct EVMImpl<'a, GSPEC: Spec, DB: Database, const INSPECT: bool> {
    data: EVMData<'a, DB>,
    precompiles: Precompiles,
    inspector: &'a mut dyn Inspector<DB>,
    network_id: u64,
    _phantomdata: PhantomData<GSPEC>,
}

pub trait Transact<DBError> {
    /// Do transaction.
    /// InstructionResult InstructionResult, Output for call or Address if we are creating contract, energy spend, energy refunded, State that needs to be applied.
    fn transact(&mut self) -> EVMResult<DBError>;
}

impl<'a, GSPEC: Spec, DB: Database, const INSPECT: bool> Transact<DB::Error>
    for EVMImpl<'a, GSPEC, DB, INSPECT>
{
    fn transact(&mut self) -> EVMResult<DB::Error> {
        let caller = self.data.env.tx.caller;
        let value = self.data.env.tx.value;
        let data = self.data.env.tx.data.clone();
        let energy_limit = self.data.env.tx.energy_limit;
        let effective_energy_price = self.data.env.effective_energy_price();

        #[cfg(feature = "optional_block_energy_limit")]
        let disable_block_energy_limit = self.env().cfg.disable_block_energy_limit;
        #[cfg(not(feature = "optional_block_energy_limit"))]
        let disable_block_energy_limit = false;

        // unusual to be found here, but check if energy_limit is more than block_energy_limit
        if !disable_block_energy_limit
            && U256::from(energy_limit) > self.data.env.block.energy_limit
        {
            return Err(InvalidTransaction::CallerEnergyLimitMoreThanBlock.into());
        }

        // load acc
        self.data
            .journaled_state
            .load_account(caller, self.data.db)
            .map_err(EVMError::Database)?;

        // Check if the transaction's chain id is correct
        if let Some(tx_chain_id) = self.data.env.tx.network_id {
            if U256::from(tx_chain_id) != U256::from(self.data.env.cfg.network_id) {
                return Err(InvalidTransaction::InvalidNetworkId.into());
            }
        }

        // Check that the transaction's nonce is correct
        if self.data.env.tx.nonce.is_some() {
            let state_nonce = self
                .data
                .journaled_state
                .state
                .get(&caller)
                .unwrap()
                .info
                .nonce;
            let tx_nonce = self.data.env.tx.nonce.unwrap();
            match tx_nonce.cmp(&state_nonce) {
                Ordering::Greater => {
                    return Err(InvalidTransaction::NonceTooHigh {
                        tx: tx_nonce,
                        state: state_nonce,
                    }
                    .into());
                }
                Ordering::Less => {
                    return Err(InvalidTransaction::NonceTooLow {
                        tx: tx_nonce,
                        state: state_nonce,
                    }
                    .into());
                }
                _ => {}
            }
        }

        #[cfg(feature = "optional_balance_check")]
        let disable_balance_check = self.env().cfg.disable_balance_check;
        #[cfg(not(feature = "optional_balance_check"))]
        let disable_balance_check = false;

        let caller_balance = &mut self
            .data
            .journaled_state
            .state
            .get_mut(&caller)
            .unwrap()
            .info
            .balance;

        let balance_check = U256::from(energy_limit)
            .checked_mul(self.data.env.tx.energy_price)
            .and_then(|energy_cost| energy_cost.checked_add(value))
            .ok_or(EVMError::Transaction(
                InvalidTransaction::OverflowPaymentInTransaction,
            ))?;

        // Check if account has enough balance for energy_limit*energy_price and value transfer.
        // Transfer will be done inside `*_inner` functions.
        if balance_check > *caller_balance && !disable_balance_check {
            return Err(InvalidTransaction::LackOfFundForEnergyLimit {
                energy_limit: balance_check,
                balance: *caller_balance,
            }
            .into());
        }

        // Reduce energy_limit*energy_price amount of caller account.
        // unwrap_or can only occur if disable_balance_check is enabled
        *caller_balance = caller_balance
            .checked_sub(U256::from(energy_limit) * effective_energy_price)
            .unwrap_or(U256::ZERO);

        let mut energy = Energy::new(energy_limit);
        // record initial energy cost. if not using energy metering init will return.
        if !energy.record_cost(self.initialization::<GSPEC>()?) {
            return Err(InvalidTransaction::CallEnergyCostMoreThanEnergyLimit.into());
        }

        // record all as cost. energy limit here is reduced by init cost of bytes and access lists.
        let energy_limit = energy.remaining();
        if crate::USE_ENERGY {
            energy.record_cost(energy_limit);
        }

        // call inner handling of call/create
        // TODO can probably be refactored to look nicer.
        let (exit_reason, ret_energy, output) = match self.data.env.tx.transact_to {
            TransactTo::Call(address) => {
                if self.data.journaled_state.inc_nonce(caller).is_some() {
                    let context = CallContext {
                        caller,
                        address,
                        code_address: address,
                        apparent_value: value,
                        scheme: CallScheme::Call,
                    };
                    let mut call_input = CallInputs {
                        contract: address,
                        transfer: Transfer {
                            source: caller,
                            target: address,
                            value,
                        },
                        input: data,
                        energy_limit,
                        context,
                        is_static: false,
                    };
                    let (exit, energy, bytes) = self.call_inner(&mut call_input);
                    (exit, energy, Output::Call(bytes))
                } else {
                    (
                        InstructionResult::NonceOverflow,
                        energy,
                        Output::Call(Bytes::new()),
                    )
                }
            }
            TransactTo::Create(scheme) => {
                let mut create_input = CreateInputs {
                    caller,
                    scheme,
                    value,
                    init_code: data,
                    energy_limit,
                };
                let (exit, address, ret_energy, bytes) = self.create_inner(&mut create_input);
                (exit, ret_energy, Output::Create(bytes, address))
            }
        };

        if crate::USE_ENERGY {
            match exit_reason {
                return_ok!() => {
                    energy.erase_cost(ret_energy.remaining());
                    energy.record_refund(ret_energy.refunded());
                }
                return_revert!() => {
                    energy.erase_cost(ret_energy.remaining());
                }
                _ => {}
            }
        }

        let (state, logs, energy_used, energy_refunded) = self.finalize::<GSPEC>(caller, &energy);

        let result = match exit_reason.into() {
            SuccessOrHalt::Success(reason) => ExecutionResult::Success {
                reason,
                energy_used,
                energy_refunded,
                logs,
                output,
            },
            SuccessOrHalt::Revert => ExecutionResult::Revert {
                energy_used,
                output: match output {
                    Output::Call(return_value) => return_value,
                    Output::Create(return_value, _) => return_value,
                },
            },
            SuccessOrHalt::Halt(reason) => ExecutionResult::Halt {
                reason,
                energy_used,
            },
            SuccessOrHalt::FatalExternalError => {
                return Err(EVMError::Database(self.data.error.take().unwrap()))
            }
            SuccessOrHalt::InternalContinue => {
                panic!("Internal return flags should remain internal {exit_reason:?}")
            }
        };

        Ok(ResultAndState { result, state })
    }
}

impl<'a, GSPEC: Spec, DB: Database, const INSPECT: bool> EVMImpl<'a, GSPEC, DB, INSPECT> {
    pub fn new(
        db: &'a mut DB,
        env: &'a mut Env,
        inspector: &'a mut dyn Inspector<DB>,
        precompiles: Precompiles,
        network_id: u64,
    ) -> Self {
        let journaled_state = if GSPEC::enabled(SpecId::SPURIOUS_DRAGON) {
            JournaledState::new(precompiles.len())
        } else {
            JournaledState::new_legacy(precompiles.len())
        };
        Self {
            data: EVMData {
                env,
                journaled_state,
                db,
                error: None,
            },
            precompiles,
            inspector,
            network_id,
            _phantomdata: PhantomData {},
        }
    }

    #[allow(clippy::extra_unused_type_parameters)]
    fn finalize<SPEC: Spec>(
        &mut self,
        caller: B176,
        energy: &Energy,
    ) -> (HashMap<B176, Account>, Vec<Log>, u64, u64) {
        let coinbase = self.data.env.block.coinbase;
        let (energy_used, energy_refunded) = if crate::USE_ENERGY {
            let effective_energy_price = self.data.env.effective_energy_price();

            #[cfg(feature = "optional_energy_refund")]
            let disable_energy_refund = self.env().cfg.disable_energy_refund;
            #[cfg(not(feature = "optional_energy_refund"))]
            let disable_energy_refund = false;

            let energy_refunded = if disable_energy_refund {
                0
            } else {
                // EIP-3529: Reduction in refunds
                let max_refund_quotient = 2;
                min(
                    energy.refunded() as u64,
                    energy.spend() / max_refund_quotient,
                )
            };
            let acc_caller = self.data.journaled_state.state().get_mut(&caller).unwrap();
            acc_caller.info.balance = acc_caller.info.balance.saturating_add(
                effective_energy_price * U256::from(energy.remaining() + energy_refunded),
            );

            // EIP-1559
            let coinbase_energy_price = effective_energy_price;

            // TODO
            let _ = self
                .data
                .journaled_state
                .load_account(coinbase, self.data.db);
            self.data.journaled_state.touch(&coinbase);
            let acc_coinbase = self
                .data
                .journaled_state
                .state()
                .get_mut(&coinbase)
                .unwrap();
            acc_coinbase.info.balance = acc_coinbase.info.balance.saturating_add(
                coinbase_energy_price * U256::from(energy.spend() - energy_refunded),
            );
            (energy.spend() - energy_refunded, energy_refunded)
        } else {
            // touch coinbase
            // TODO return
            let _ = self
                .data
                .journaled_state
                .load_account(coinbase, self.data.db);
            self.data.journaled_state.touch(&coinbase);
            (0, 0)
        };
        let (mut new_state, logs) = self.data.journaled_state.finalize();
        // precompiles are special case. If there is precompiles in finalized Map that means some balance is
        // added to it, we need now to load precompile address from db and add this amount to it so that we
        // will have sum.
        if self.data.env.cfg.perf_all_precompiles_have_balance {
            for address in self.precompiles.addresses() {
                let address = B176(*address);
                if let Some(precompile) = new_state.get_mut(&address) {
                    // we found it.
                    precompile.info.balance += self
                        .data
                        .db
                        .basic(address)
                        .ok()
                        .flatten()
                        .map(|acc| acc.balance)
                        .unwrap_or_default();
                }
            }
        }

        (new_state, logs, energy_used, energy_refunded)
    }

    fn initialization<SPEC: Spec>(&mut self) -> Result<u64, EVMError<DB::Error>> {
        let is_create = matches!(self.data.env.tx.transact_to, TransactTo::Create(_));
        let input = &self.data.env.tx.data;

        // EIP-3860: Limit and meter initcode
        let initcode_cost = 0;

        if crate::USE_ENERGY {
            let zero_data_len = input.iter().filter(|v| **v == 0).count() as u64;
            let non_zero_data_len = input.len() as u64 - zero_data_len;
            let (accessed_accounts, accessed_slots) = (0, 0);

            let transact = if is_create {
                if SPEC::enabled(HOMESTEAD) {
                    // EIP-2: Homestead Hard-fork Changes
                    53000
                } else {
                    21000
                }
            } else {
                21000
            };

            // EIP-2028: Transaction data energy cost reduction
            let energy_transaction_non_zero_data = if SPEC::enabled(ISTANBUL) { 16 } else { 68 };

            Ok(transact
                + initcode_cost
                + zero_data_len * energy::TRANSACTION_ZERO_DATA
                + non_zero_data_len * energy_transaction_non_zero_data
                + accessed_accounts * energy::ACCESS_LIST_ADDRESS
                + accessed_slots * energy::ACCESS_LIST_STORAGE_KEY)
        } else {
            Ok(0)
        }
    }

    fn create_inner(
        &mut self,
        inputs: &mut CreateInputs,
    ) -> (InstructionResult, Option<B176>, Energy, Bytes) {
        // Call inspector
        if INSPECT {
            let (ret, address, energy, out) = self.inspector.create(&mut self.data, inputs);
            if ret != InstructionResult::Continue {
                return self.inspector.create_end(
                    &mut self.data,
                    inputs,
                    ret,
                    address,
                    energy,
                    out,
                );
            }
        }

        let energy = Energy::new(inputs.energy_limit);
        self.load_account(inputs.caller);

        // Check depth of calls
        if self.data.journaled_state.depth() > CALL_STACK_LIMIT {
            return self.create_end(
                inputs,
                InstructionResult::CallTooDeep,
                None,
                energy,
                Bytes::new(),
            );
        }
        // Check balance of caller and value. Do this before increasing nonce
        match self.balance(inputs.caller) {
            Some(i) if i.0 < inputs.value => {
                return self.create_end(
                    inputs,
                    InstructionResult::OutOfFund,
                    None,
                    energy,
                    Bytes::new(),
                )
            }
            Some(_) => (),
            _ => {
                return self.create_end(
                    inputs,
                    InstructionResult::FatalExternalError,
                    None,
                    energy,
                    Bytes::new(),
                )
            }
        }

        // Increase nonce of caller and check if it overflows
        let old_nonce;
        if let Some(nonce) = self.data.journaled_state.inc_nonce(inputs.caller) {
            old_nonce = nonce - 1;
        } else {
            return self.create_end(
                inputs,
                InstructionResult::Return,
                None,
                energy,
                Bytes::new(),
            );
        }

        // Create address
        let code_hash = sha3(&inputs.init_code);
        let created_address = match inputs.scheme {
            CreateScheme::Create => create_address(inputs.caller, old_nonce),
            CreateScheme::Create2 { salt } => {
                create2_address(inputs.caller, code_hash, salt, self.network_id)
            }
        };
        let ret = Some(created_address);

        // Load account so that it will be hot
        self.load_account(created_address);

        // Enter subroutine
        let checkpoint = self.data.journaled_state.checkpoint();

        // Create contract account and check for collision
        match self.data.journaled_state.create_account(
            created_address,
            self.precompiles.contains(&created_address),
            self.data.db,
        ) {
            Ok(false) => {
                self.data.journaled_state.checkpoint_revert(checkpoint);
                return self.create_end(
                    inputs,
                    InstructionResult::CreateCollision,
                    ret,
                    energy,
                    Bytes::new(),
                );
            }
            Err(err) => {
                self.data.error = Some(err);
                return self.create_end(
                    inputs,
                    InstructionResult::FatalExternalError,
                    ret,
                    energy,
                    Bytes::new(),
                );
            }
            Ok(true) => (),
        }

        // Transfer value to contract address
        if let Err(e) = self.data.journaled_state.transfer(
            &inputs.caller,
            &created_address,
            inputs.value,
            self.data.db,
        ) {
            self.data.journaled_state.checkpoint_revert(checkpoint);
            return self.create_end(inputs, e, ret, energy, Bytes::new());
        }

        // EIP-161: State trie clearing (invariant-preserving alternative)
        if GSPEC::enabled(SPURIOUS_DRAGON)
            && self
                .data
                .journaled_state
                .inc_nonce(created_address)
                .is_none()
        {
            // overflow
            self.data.journaled_state.checkpoint_revert(checkpoint);
            return self.create_end(
                inputs,
                InstructionResult::Return,
                None,
                energy,
                Bytes::new(),
            );
        }

        // Create new interpreter and execute initcode
        let contract = Contract::new(
            Bytes::new(),
            Bytecode::new_raw(inputs.init_code.clone()),
            created_address,
            inputs.caller,
            inputs.value,
        );

        #[cfg(feature = "memory_limit")]
        let mut interpreter = Interpreter::new_with_memory_limit(
            contract,
            energy.limit(),
            false,
            self.data.env.cfg.memory_limit,
        );

        #[cfg(not(feature = "memory_limit"))]
        let mut interpreter = Interpreter::new(contract, energy.limit(), false);

        if INSPECT {
            self.inspector
                .initialize_interp(&mut interpreter, &mut self.data, false);
        }
        let exit_reason = if INSPECT {
            interpreter.run_inspect::<Self, GSPEC>(self)
        } else {
            interpreter.run::<Self, GSPEC>(self)
        };
        // Host error if present on execution\
        let (ret, address, energy, out) = match exit_reason {
            return_ok!() => {
                // if ok, check contract creation limit and calculate energy deduction on output len.
                let mut bytes = interpreter.return_value();

                // EIP-170: Contract code size limit
                // By default limit is 0x6000 (~25kb)
                if GSPEC::enabled(SPURIOUS_DRAGON)
                    && bytes.len()
                        > self
                            .data
                            .env
                            .cfg
                            .limit_contract_code_size
                            .unwrap_or(MAX_CODE_SIZE)
                {
                    self.data.journaled_state.checkpoint_revert(checkpoint);
                    return self.create_end(
                        inputs,
                        InstructionResult::CreateContractSizeLimit,
                        ret,
                        interpreter.energy,
                        bytes,
                    );
                }
                if crate::USE_ENERGY {
                    let energy_for_code = bytes.len() as u64 * energy::CODEDEPOSIT;
                    if !interpreter.energy.record_cost(energy_for_code) {
                        // record code deposit energy cost and check if we are out of energy.
                        // EIP-2 point 3: If contract creation does not have enough energy to pay for the
                        // final energy fee for adding the contract code to the state, the contract
                        //  creation fails (i.e. goes out-of-energy) rather than leaving an empty contract.
                        if GSPEC::enabled(HOMESTEAD) {
                            self.data.journaled_state.checkpoint_revert(checkpoint);
                            return self.create_end(
                                inputs,
                                InstructionResult::OutOfEnergy,
                                ret,
                                interpreter.energy,
                                bytes,
                            );
                        } else {
                            bytes = Bytes::new();
                        }
                    }
                }
                // if we have enough energy
                self.data.journaled_state.checkpoint_commit();
                // Do analysis of bytecode straight away.
                let bytecode = match self.data.env.cfg.perf_analyse_created_bytecodes {
                    AnalysisKind::Raw => Bytecode::new_raw(bytes.clone()),
                    AnalysisKind::Check => Bytecode::new_raw(bytes.clone()).to_checked(),
                    AnalysisKind::Analyse => to_analysed(Bytecode::new_raw(bytes.clone())),
                };

                self.data
                    .journaled_state
                    .set_code(created_address, bytecode);
                (InstructionResult::Return, ret, interpreter.energy, bytes)
            }
            _ => {
                self.data.journaled_state.checkpoint_revert(checkpoint);
                (
                    exit_reason,
                    ret,
                    interpreter.energy,
                    interpreter.return_value(),
                )
            }
        };

        self.create_end(inputs, ret, address, energy, out)
    }

    fn create_end(
        &mut self,
        inputs: &CreateInputs,
        ret: InstructionResult,
        address: Option<B176>,
        energy: Energy,
        out: Bytes,
    ) -> (InstructionResult, Option<B176>, Energy, Bytes) {
        if INSPECT {
            self.inspector
                .create_end(&mut self.data, inputs, ret, address, energy, out)
        } else {
            (ret, address, energy, out)
        }
    }

    fn call_inner(&mut self, inputs: &mut CallInputs) -> (InstructionResult, Energy, Bytes) {
        // Call the inspector
        if INSPECT {
            let (ret, energy, out) = self
                .inspector
                .call(&mut self.data, inputs, inputs.is_static);
            if ret != InstructionResult::Continue {
                return self.inspector.call_end(
                    &mut self.data,
                    inputs,
                    energy,
                    ret,
                    out,
                    inputs.is_static,
                );
            }
        }

        let mut energy = Energy::new(inputs.energy_limit);
        // Load account and get code. Account is now hot.
        let bytecode = if let Some((bytecode, _)) = self.code(inputs.contract) {
            bytecode
        } else {
            return (InstructionResult::FatalExternalError, energy, Bytes::new());
        };

        // Check depth
        if self.data.journaled_state.depth() > CALL_STACK_LIMIT {
            let (ret, energy, out) = (InstructionResult::CallTooDeep, energy, Bytes::new());
            if INSPECT {
                return self.inspector.call_end(
                    &mut self.data,
                    inputs,
                    energy,
                    ret,
                    out,
                    inputs.is_static,
                );
            } else {
                return (ret, energy, out);
            }
        }

        // Create subroutine checkpoint
        let checkpoint = self.data.journaled_state.checkpoint();

        // Touch address. For "EIP-158 State Clear", this will erase empty accounts.
        if inputs.transfer.value == U256::ZERO {
            self.load_account(inputs.context.address);
            self.data.journaled_state.touch(&inputs.context.address);
        }

        // Transfer value from caller to called account
        if let Err(e) = self.data.journaled_state.transfer(
            &inputs.transfer.source,
            &inputs.transfer.target,
            inputs.transfer.value,
            self.data.db,
        ) {
            self.data.journaled_state.checkpoint_revert(checkpoint);
            let (ret, energy, out) = (e, energy, Bytes::new());
            if INSPECT {
                return self.inspector.call_end(
                    &mut self.data,
                    inputs,
                    energy,
                    ret,
                    out,
                    inputs.is_static,
                );
            } else {
                return (ret, energy, out);
            }
        }

        // Call precompiles
        let (ret, energy, out) = if let Some(precompile) = self.precompiles.get(&inputs.contract) {
            let out = match precompile {
                Precompile::Standard(fun) => fun(inputs.input.as_ref(), inputs.energy_limit),
                Precompile::Custom(fun) => {
                    let network = inputs.contract.as_bytes();
                    // This is kidn of a hack, we check the prefix of the calling contract, and
                    // according to that we know what prefix to use for the ecrecover recovered
                    // address.
                    let network = Network::from_prefix_numerical(network[0]);
                    fun(inputs.input.as_ref(), inputs.energy_limit, network)
                }
            };
            match out {
                Ok((energy_used, data)) => {
                    if !crate::USE_ENERGY || energy.record_cost(energy_used) {
                        self.data.journaled_state.checkpoint_commit();
                        (InstructionResult::Return, energy, Bytes::from(data))
                    } else {
                        self.data.journaled_state.checkpoint_revert(checkpoint);
                        (InstructionResult::PrecompileOOG, energy, Bytes::new())
                    }
                }
                Err(e) => {
                    let ret = if let precompile::Error::OutOfEnergy = e {
                        InstructionResult::PrecompileOOG
                    } else {
                        InstructionResult::PrecompileError
                    };
                    self.data.journaled_state.checkpoint_revert(checkpoint);
                    (ret, energy, Bytes::new())
                }
            }
        } else {
            // Create interpreter and execute subcall
            let contract =
                Contract::new_with_context(inputs.input.clone(), bytecode, &inputs.context);

            #[cfg(feature = "memory_limit")]
            let mut interpreter = Interpreter::new_with_memory_limit(
                contract,
                energy.limit(),
                inputs.is_static,
                self.data.env.cfg.memory_limit,
            );

            #[cfg(not(feature = "memory_limit"))]
            let mut interpreter = Interpreter::new(contract, energy.limit(), inputs.is_static);

            if INSPECT {
                // create is always no static call.
                self.inspector
                    .initialize_interp(&mut interpreter, &mut self.data, false);
            }
            let exit_reason = if INSPECT {
                interpreter.run_inspect::<Self, GSPEC>(self)
            } else {
                interpreter.run::<Self, GSPEC>(self)
            };

            if matches!(exit_reason, return_ok!()) {
                self.data.journaled_state.checkpoint_commit();
            } else {
                self.data.journaled_state.checkpoint_revert(checkpoint);
            }

            (exit_reason, interpreter.energy, interpreter.return_value())
        };

        if INSPECT {
            self.inspector
                .call_end(&mut self.data, inputs, energy, ret, out, inputs.is_static)
        } else {
            (ret, energy, out)
        }
    }
}

impl<'a, GSPEC: Spec, DB: Database + 'a, const INSPECT: bool> Host
    for EVMImpl<'a, GSPEC, DB, INSPECT>
{
    fn step(&mut self, interp: &mut Interpreter, is_static: bool) -> InstructionResult {
        self.inspector.step(interp, &mut self.data, is_static)
    }

    fn step_end(
        &mut self,
        interp: &mut Interpreter,
        is_static: bool,
        ret: InstructionResult,
    ) -> InstructionResult {
        self.inspector
            .step_end(interp, &mut self.data, is_static, ret)
    }

    fn env(&mut self) -> &mut Env {
        self.data.env
    }

    fn block_hash(&mut self, number: U256) -> Option<B256> {
        self.data
            .db
            .block_hash(number)
            .map_err(|e| self.data.error = Some(e))
            .ok()
    }

    fn load_account(&mut self, address: B176) -> Option<(bool, bool)> {
        self.data
            .journaled_state
            .load_account_exist(address, self.data.db)
            .map_err(|e| self.data.error = Some(e))
            .ok()
    }

    fn balance(&mut self, address: B176) -> Option<(U256, bool)> {
        let db = &mut self.data.db;
        let journal = &mut self.data.journaled_state;
        let error = &mut self.data.error;
        journal
            .load_account(address, db)
            .map_err(|e| *error = Some(e))
            .ok()
            .map(|(acc, is_cold)| (acc.info.balance, is_cold))
    }

    fn code(&mut self, address: B176) -> Option<(Bytecode, bool)> {
        let journal = &mut self.data.journaled_state;
        let db = &mut self.data.db;
        let error = &mut self.data.error;

        let (acc, is_cold) = journal
            .load_code(address, db)
            .map_err(|e| *error = Some(e))
            .ok()?;
        Some((acc.info.code.clone().unwrap(), is_cold))
    }

    /// Get code hash of address.
    fn code_hash(&mut self, address: B176) -> Option<(B256, bool)> {
        let journal = &mut self.data.journaled_state;
        let db = &mut self.data.db;
        let error = &mut self.data.error;

        let (acc, is_cold) = journal
            .load_code(address, db)
            .map_err(|e| *error = Some(e))
            .ok()?;
        //asume that all precompiles have some balance
        let is_precompile = self.precompiles.contains(&address);
        if is_precompile && self.data.env.cfg.perf_all_precompiles_have_balance {
            return Some((SHA3_EMPTY, is_cold));
        }
        if acc.is_empty() {
            // TODO check this for pre tangerine fork
            return Some((B256::zero(), is_cold));
        }

        Some((acc.info.code_hash, is_cold))
    }

    fn sload(&mut self, address: B176, index: U256) -> Option<(U256, bool)> {
        // account is always hot. reference on that statement https://eips.ethereum.org/EIPS/eip-2929 see `Note 2:`
        self.data
            .journaled_state
            .sload(address, index, self.data.db)
            .map_err(|e| self.data.error = Some(e))
            .ok()
    }

    fn sstore(
        &mut self,
        address: B176,
        index: U256,
        value: U256,
    ) -> Option<(U256, U256, U256, bool)> {
        self.data
            .journaled_state
            .sstore(address, index, value, self.data.db)
            .map_err(|e| self.data.error = Some(e))
            .ok()
    }

    fn log(&mut self, address: B176, topics: Vec<B256>, data: Bytes) {
        if INSPECT {
            self.inspector.log(&mut self.data, &address, &topics, &data);
        }
        let log = Log {
            address,
            topics,
            data,
        };
        self.data.journaled_state.log(log);
    }

    fn selfdestruct(&mut self, address: B176, target: B176) -> Option<SelfDestructResult> {
        if INSPECT {
            self.inspector.selfdestruct(address, target);
        }
        self.data
            .journaled_state
            .selfdestruct(address, target, self.data.db)
            .map_err(|e| self.data.error = Some(e))
            .ok()
    }

    fn create(
        &mut self,
        inputs: &mut CreateInputs,
    ) -> (InstructionResult, Option<B176>, Energy, Bytes) {
        self.create_inner(inputs)
    }

    fn call(&mut self, inputs: &mut CallInputs) -> (InstructionResult, Energy, Bytes) {
        self.call_inner(inputs)
    }
}
