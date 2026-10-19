use revm::primitives::{AccountInfo, Bytecode, Bytes, B176, SHA3_EMPTY, U256};
use revm::{InMemoryDB, JournaledState};

fn address() -> B176 {
    B176([0x11; 22])
}
fn code(value: u8) -> Bytecode {
    Bytecode::new_raw(Bytes::from(vec![0x60, value, 0x00]))
}

#[test]
fn rollback_restores_preloaded_empty_code() {
    let mut db = InMemoryDB::default();
    let mut state = JournaledState::new(0);
    state.load_account(address(), &mut db).unwrap();
    let before = state.account(address()).info.clone();
    let checkpoint = state.checkpoint();
    state.set_code(address(), code(1));
    state.checkpoint_revert(checkpoint);
    assert_eq!(state.account(address()).info.code_hash, before.code_hash);
    assert_eq!(state.account(address()).info.code, before.code);
}

#[test]
fn rollback_restores_existing_code() {
    let mut db = InMemoryDB::default();
    db.insert_account_info(address(), AccountInfo::new(U256::ZERO, 1, code(1)));
    let mut state = JournaledState::new(0);
    state.load_code(address(), &mut db).unwrap();
    let before = state.account(address()).info.clone();
    let checkpoint = state.checkpoint();
    state.set_code(address(), code(2));
    state.checkpoint_revert(checkpoint);
    assert_eq!(state.account(address()).info.code_hash, before.code_hash);
    assert_eq!(state.account(address()).info.code, before.code);
}

#[test]
fn rollback_restores_unloaded_code_and_original_hash() {
    let mut db = InMemoryDB::default();
    let old = code(1);
    db.contracts.insert(old.hash(), old.clone());
    db.insert_account_info(
        address(),
        AccountInfo {
            code: None,
            code_hash: old.hash(),
            nonce: 1,
            balance: U256::ZERO,
        },
    );
    let mut state = JournaledState::new(0);
    state.load_account(address(), &mut db).unwrap();
    assert!(state.account(address()).info.code.is_none());
    let checkpoint = state.checkpoint();
    state.set_code(address(), code(2));
    state.checkpoint_revert(checkpoint);
    assert_eq!(state.account(address()).info.code_hash, old.hash());
    assert!(state.account(address()).info.code.is_none());
    assert_eq!(
        state.load_code(address(), &mut db).unwrap().0.info.code,
        Some(old)
    );
}

#[test]
fn rollback_removes_account_loaded_inside_checkpoint() {
    let mut db = InMemoryDB::default();
    let mut state = JournaledState::new(0);
    let checkpoint = state.checkpoint();
    state.load_account(address(), &mut db).unwrap();
    state.set_code(address(), code(1));
    state.checkpoint_revert(checkpoint);
    assert!(!state.state.contains_key(&address()));
}

#[test]
fn parent_revert_undoes_successful_child_creation_code() {
    let mut db = InMemoryDB::default();
    db.insert_account_info(address(), AccountInfo::from_balance(U256::from(1)));
    let mut state = JournaledState::new(0);
    state.load_account(address(), &mut db).unwrap();
    let parent = state.checkpoint();
    let _child = state.checkpoint();
    assert!(state.create_account(address(), false, &mut db).unwrap());
    state.inc_nonce(address());
    state
        .sstore(address(), U256::ZERO, U256::from(123), &mut db)
        .unwrap();
    state.checkpoint_commit();
    state.set_code(address(), code(1));
    state.checkpoint_revert(parent);
    let account = state.account(address());
    assert_eq!(account.info.nonce, 0);
    assert_eq!(account.info.balance, U256::from(1));
    assert!(account.storage.is_empty());
    assert_eq!(account.info.code_hash, SHA3_EMPTY);
    assert!(account
        .info
        .code
        .as_ref()
        .map_or(true, |code| code.is_empty()));
}
