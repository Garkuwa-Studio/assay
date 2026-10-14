//! TESTNET DEMO ONLY. Not a lending product: no interest, no liquidation,
//! fixed 1:1 prices, no audit. Never deploy to mainnet.
//!
//! A deliberately tiny pool that shows Assay Guard as a circuit breaker:
//! users deposit a demo RWA token as collateral and borrow a demo
//! stablecoin, and `borrow` asks the guard first. When the guard says the
//! collateral asset is unhealthy, the borrow is rejected with the guard's
//! reason. The rejection is recorded (an event and the return value) in a
//! successful transaction so the demo has a transaction to link to; a
//! production pool would more likely revert.
#![no_std]

use soroban_sdk::{
    contract, contractclient, contracterror, contractevent, contractimpl, contracttype, token,
    Address, Env, String,
};

pub const NOTICE: &str =
    "TESTNET DEMO ONLY. Not a lending product: no interest, no liquidation, fixed 1:1 prices. Never deploy to mainnet.";

/// Basis points in 100%.
pub const BPS: i128 = 10_000;

/// Assay Guard's `Policy` (contracts/guard), mirrored so this wasm does not
/// link the guard. Tests run against the real guard.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Policy {
    pub min_coverage_bps: u32,
    pub max_age_secs: u64,
    pub min_attestors: u32,
    pub max_disagreement_bps: u32,
    pub max_supply_mismatch_bps: u32,
}

/// Assay Guard's `Reason`, mirrored.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Reason {
    Ok,
    NoReport,
    Stale,
    TooFewAttestors,
    Undercovered,
    Disagreement,
    SupplyMismatch,
}

#[contractclient(name = "GuardClient")]
pub trait GuardApi {
    fn is_healthy(env: Env, asset: Address, policy: Policy) -> (bool, Reason);
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub admin: Address,
    pub guard: Address,
    /// Demo RWA token taken as collateral.
    pub collateral: Address,
    /// Demo stablecoin lent out.
    pub debt: Address,
    pub policy: Policy,
    /// Borrow limit as a share of collateral (prices fixed at 1:1).
    pub ltv_bps: u32,
}

#[contracttype]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Position {
    pub collateral: i128,
    pub debt: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BorrowResult {
    Borrowed,
    Rejected(Reason),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    InvalidAmount = 1,
    OverLimit = 2,
    InvalidConfig = 3,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Config,
    Position(Address),
}

#[contractevent(topics = ["testnet_demo", "deposited"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Deposited {
    #[topic]
    pub user: Address,
    pub amount: i128,
}

#[contractevent(topics = ["testnet_demo", "borrowed"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Borrowed {
    #[topic]
    pub user: Address,
    pub amount: i128,
}

#[contractevent(topics = ["testnet_demo", "borrow_rejected"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BorrowRejected {
    #[topic]
    pub user: Address,
    pub amount: i128,
    pub reason: Reason,
}

#[contract]
pub struct DemoPool;

#[contractimpl]
impl DemoPool {
    pub fn __constructor(
        env: Env,
        admin: Address,
        guard: Address,
        collateral: Address,
        debt: Address,
        policy: Policy,
        ltv_bps: u32,
    ) -> Result<(), Error> {
        if ltv_bps == 0 || ltv_bps as i128 > BPS {
            return Err(Error::InvalidConfig);
        }
        let cfg = Config {
            admin,
            guard,
            collateral,
            debt,
            policy,
            ltv_bps,
        };
        env.storage().instance().set(&DataKey::Config, &cfg);
        bump_instance(&env);
        Ok(())
    }

    /// Contract interface version.
    pub fn version() -> u32 {
        0
    }

    /// Says what this contract is.
    pub fn notice(env: Env) -> String {
        String::from_str(&env, NOTICE)
    }

    pub fn config(env: Env) -> Config {
        env.storage().instance().get(&DataKey::Config).unwrap()
    }

    pub fn position(env: Env, user: Address) -> Position {
        env.storage()
            .persistent()
            .get(&DataKey::Position(user))
            .unwrap_or_default()
    }

    /// Moves `amount` of the collateral token from `user` into the pool.
    pub fn deposit(env: Env, user: Address, amount: i128) -> Result<(), Error> {
        user.require_auth();
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        let cfg = Self::config(env.clone());
        token::Client::new(&env, &cfg.collateral).transfer(
            &user,
            env.current_contract_address(),
            &amount,
        );
        let mut pos = Self::position(env.clone(), user.clone());
        pos.collateral += amount;
        save_position(&env, &user, &pos);
        Deposited { user, amount }.publish(&env);
        Ok(())
    }

    /// Asks the guard about the collateral asset, then lends `amount` of the
    /// debt token. When the guard says unhealthy, nothing moves and the
    /// guard's reason is returned and emitted.
    pub fn borrow(env: Env, user: Address, amount: i128) -> Result<BorrowResult, Error> {
        user.require_auth();
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        let cfg = Self::config(env.clone());
        let (healthy, reason) =
            GuardClient::new(&env, &cfg.guard).is_healthy(&cfg.collateral, &cfg.policy);
        if !healthy {
            BorrowRejected {
                user,
                amount,
                reason,
            }
            .publish(&env);
            bump_instance(&env);
            return Ok(BorrowResult::Rejected(reason));
        }
        let mut pos = Self::position(env.clone(), user.clone());
        let limit = pos.collateral * cfg.ltv_bps as i128 / BPS;
        let new_debt = pos.debt.checked_add(amount).ok_or(Error::OverLimit)?;
        if new_debt > limit {
            return Err(Error::OverLimit);
        }
        token::Client::new(&env, &cfg.debt).transfer(
            &env.current_contract_address(),
            &user,
            &amount,
        );
        pos.debt = new_debt;
        save_position(&env, &user, &pos);
        Borrowed { user, amount }.publish(&env);
        Ok(BorrowResult::Borrowed)
    }
}

fn save_position(env: &Env, user: &Address, pos: &Position) {
    let key = DataKey::Position(user.clone());
    let max = env.storage().max_ttl();
    env.storage().persistent().set(&key, pos);
    env.storage().persistent().extend_ttl(&key, max, max);
    bump_instance(env);
}

fn bump_instance(env: &Env) {
    let max = env.storage().max_ttl();
    env.storage().instance().extend_ttl(max, max);
}

#[cfg(test)]
mod test;
