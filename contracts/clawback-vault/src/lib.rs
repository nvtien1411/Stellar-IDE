#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Env, Address, Symbol, symbol_short};

// ===== STORAGE KEYS =====
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Owner,
    Beneficiary,
    LastPing,
    Timeout,
    Grace,
}

// ===== CONTRACT =====
#[contract]
pub struct LegacyVault;

#[contractimpl]
impl LegacyVault {

    // ===== INIT =====
    pub fn init(
        env: Env,
        owner: Address,
        beneficiary: Address,
        timeout: u64,
        grace: u64,
    ) {
        let storage = env.storage().instance();

        if storage.has(&DataKey::Owner) {
            panic!("Already initialized");
        }

        owner.require_auth();

        storage.set(&DataKey::Owner, &owner);
        storage.set(&DataKey::Beneficiary, &beneficiary);
        storage.set(&DataKey::Timeout, &timeout);
        storage.set(&DataKey::Grace, &grace);

        let now = env.ledger().timestamp();
        storage.set(&DataKey::LastPing, &now);
    }

    // ===== PING =====
    pub fn ping(env: Env) {
        let storage = env.storage().instance();

        let owner: Address = storage.get(&DataKey::Owner).unwrap();
        owner.require_auth();

        let now = env.ledger().timestamp();
        storage.set(&DataKey::LastPing, &now);
    }

    // ===== CANCEL =====
    pub fn cancel(env: Env, token: Address) {
        let storage = env.storage().instance();

        let owner: Address = storage.get(&DataKey::Owner).unwrap();
        owner.require_auth();

        let contract = env.current_contract_address();

        let client = soroban_sdk::token::Client::new(&env, &token);
        let balance = client.balance(&contract);

        if balance > 0 {
            client.transfer(&contract, &owner, &balance);
        }
    }

    // ===== CLAIM =====
    pub fn claim(env: Env, token: Address) {
        let storage = env.storage().instance();

        let beneficiary: Address = storage.get(&DataKey::Beneficiary).unwrap();
        beneficiary.require_auth();

        let last_ping: u64 = storage.get(&DataKey::LastPing).unwrap();
        let timeout: u64 = storage.get(&DataKey::Timeout).unwrap();
        let grace: u64 = storage.get(&DataKey::Grace).unwrap();

        let now = env.ledger().timestamp();

        if now <= last_ping + timeout + grace {
            panic!("Not claimable yet");
        }

        let contract = env.current_contract_address();
        let client = soroban_sdk::token::Client::new(&env, &token);
        let balance = client.balance(&contract);

        if balance > 0 {
            client.transfer(&contract, &beneficiary, &balance);
        }
    }

    // ===== VIEW =====
    pub fn get_state(env: Env) -> Symbol {
        let storage = env.storage().instance();

        let last_ping: u64 = storage.get(&DataKey::LastPing).unwrap();
        let timeout: u64 = storage.get(&DataKey::Timeout).unwrap();
        let grace: u64 = storage.get(&DataKey::Grace).unwrap();

        let now = env.ledger().timestamp();

        if now <= last_ping + timeout {
            symbol_short!("ACTIVE")
        } else if now <= last_ping + timeout + grace {
            symbol_short!("GRACE")
        } else {
            symbol_short!("CLAIM")
        }
    }
}