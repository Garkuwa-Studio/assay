//! Assay Guard v0 (docs/ARCHITECTURE.md §4.3): `is_healthy(asset, policy)`
//! reads the reserve registry and says whether a protocol should accept more
//! exposure to an asset, with the first rule that failed.
//!
//! Read-only: the guard never moves funds and holds no tokens. What
//! "unhealthy" means for a protocol (reject a borrow, pause a market) is the
//! protocol's decision.
//!
//! Missing data is never healthy, and that is not configurable.
#![no_std]

use soroban_sdk::{
    contract, contractclient, contracterror, contractimpl, contracttype, Address, Env,
};

/// Basis points in 100%.
pub const BPS: u32 = 10_000;

/// The registry's `Status` (contracts/registry). Mirrored here so the guard
/// wasm does not link the registry; the guard's tests run against the real
/// registry, so a drift fails them.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistryStatus {
    pub coverage_bps: u32,
    pub newest_age_secs: u64,
    pub attestor_count: u32,
    pub disagreement_bps: u32,
    pub supply_mismatch: bool,
    pub supply_age_secs: u64,
}

#[contractclient(name = "RegistryClient")]
pub trait RegistryApi {
    fn status_with(
        env: Env,
        asset: Address,
        max_age_secs: u64,
        mismatch_tolerance_bps: u32,
    ) -> RegistryStatus;
}

/// A consumer's thresholds.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Policy {
    pub min_coverage_bps: u32,
    pub max_age_secs: u64,
    pub min_attestors: u32,
    pub max_disagreement_bps: u32,
    pub max_supply_mismatch_bps: u32,
}

impl Policy {
    /// The architecture's example policy: 100% coverage, reports under 35
    /// days, one attestor, 5% disagreement, 0.5% supply tolerance.
    pub fn standard() -> Self {
        Policy {
            min_coverage_bps: BPS,
            max_age_secs: 35 * 24 * 60 * 60,
            min_attestors: 1,
            max_disagreement_bps: 500,
            max_supply_mismatch_bps: 50,
        }
    }
}

/// Why an asset is or is not healthy. Rules are checked in this order and
/// the first failure is returned.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Reason {
    /// Every rule passed.
    Ok,
    /// No report at all, or the asset is not registered.
    NoReport,
    /// Reports exist but none is inside the max age.
    Stale,
    /// Fewer fresh attestors than the policy requires.
    TooFewAttestors,
    /// Lowest fresh coverage below the policy minimum.
    Undercovered,
    /// Spread between attestors above the policy maximum.
    Disagreement,
    /// Witnessed supply above stated liabilities beyond the tolerance.
    SupplyMismatch,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    InvalidPolicy = 1,
}

/// The guard's rules, usable inline by a protocol that reads the registry
/// itself. `s` must come from `status_with` under the same policy's max age
/// and supply tolerance.
pub fn evaluate(s: &RegistryStatus, p: &Policy) -> Reason {
    if s.newest_age_secs == u64::MAX {
        Reason::NoReport
    } else if s.attestor_count == 0 {
        Reason::Stale
    } else if s.attestor_count < p.min_attestors {
        Reason::TooFewAttestors
    } else if s.coverage_bps < p.min_coverage_bps {
        Reason::Undercovered
    } else if s.disagreement_bps > p.max_disagreement_bps {
        Reason::Disagreement
    } else if s.supply_mismatch {
        Reason::SupplyMismatch
    } else {
        Reason::Ok
    }
}

/// A policy that could let missing or meaningless data through is refused.
pub fn check_policy(p: &Policy) -> Result<(), Error> {
    if p.min_attestors == 0
        || p.max_age_secs == 0
        || p.min_coverage_bps == 0
        || p.max_supply_mismatch_bps > BPS
    {
        return Err(Error::InvalidPolicy);
    }
    Ok(())
}

#[derive(Clone)]
#[contracttype]
enum DataKey {
    Registry,
}

#[contract]
pub struct Guard;

#[contractimpl]
impl Guard {
    pub fn __constructor(env: Env, registry: Address) {
        env.storage().instance().set(&DataKey::Registry, &registry);
        let max = env.storage().max_ttl();
        env.storage().instance().extend_ttl(max, max);
    }

    /// Contract interface version.
    pub fn version() -> u32 {
        0
    }

    pub fn registry(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Registry).unwrap()
    }

    /// `(true, Ok)` when every rule passes; otherwise `(false, reason)` with
    /// the first rule that failed. Needs no authorization and writes nothing.
    pub fn is_healthy(env: Env, asset: Address, policy: Policy) -> Result<(bool, Reason), Error> {
        check_policy(&policy)?;
        let status = RegistryClient::new(&env, &Self::registry(env.clone())).status_with(
            &asset,
            &policy.max_age_secs,
            &policy.max_supply_mismatch_bps,
        );
        let reason = evaluate(&status, &policy);
        Ok((reason == Reason::Ok, reason))
    }
}

#[cfg(test)]
mod test;
