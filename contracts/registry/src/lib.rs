//! Assay reserve registry v0 (docs/ARCHITECTURE.md §4.2).
//!
//! Registered attestors post reserve reports per asset, a supply witness
//! posts the asset's supply, and anyone reads a status computed on read.
//! The status is conservative by design: the lowest coverage across fresh
//! attestors, liabilities taken as the larger of stated and witnessed
//! supply, and missing or stale data never reads as healthy.
//!
//! Amounts are integers with 7 decimals (the Stellar convention). The
//! contract never converts units: a report must state reserves in the unit
//! the asset was registered with.
#![no_std]

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, Address, BytesN, Env,
    String, Symbol, Vec,
};

/// Basis points in 100%.
pub const BPS: i128 = 10_000;
/// Attestors per asset; bounds the loop in `status`.
pub const MAX_ATTESTORS: u32 = 20;
/// Reports kept per attestor per asset; older ones are dropped.
pub const HISTORY_LEN: u32 = 10;
/// Suggested `max_age_secs` at registration (35 days).
pub const DEFAULT_MAX_AGE_SECS: u64 = 35 * 24 * 60 * 60;
/// Suggested `mismatch_tolerance_bps` at registration (0.5%).
pub const DEFAULT_MISMATCH_TOLERANCE_BPS: u32 = 50;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyRegistered = 1,
    NotRegistered = 2,
    AlreadyAttestor = 3,
    NotAttestor = 4,
    TooManyAttestors = 5,
    NoWitness = 6,
    UnitMismatch = 7,
    FutureTimestamp = 8,
    FutureLedger = 9,
    StaleLedger = 10,
    OutOfOrder = 11,
    NegativeAmount = 12,
    InvalidPolicy = 13,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetConfig {
    pub asset_admin: Address,
    /// What the asset is backed by, e.g. `USD`. Reports must use it.
    pub unit: Symbol,
    pub max_age_secs: u64,
    pub mismatch_tolerance_bps: u32,
    pub witness: Option<Address>,
    pub registered_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Attestor {
    pub address: Address,
    pub added_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Report {
    pub reserves: i128,
    pub reserves_unit: Symbol,
    /// Liabilities as stated by the report, in token units.
    pub liabilities: i128,
    /// Unix seconds the figures are as of. Report age is measured from it.
    pub as_of: u64,
    pub ledger: u32,
    pub uri: String,
    /// SHA-256 of the full report document.
    pub sha256: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReportEntry {
    pub report: Report,
    pub posted_at: u64,
    pub posted_ledger: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupplyPost {
    pub amount: i128,
    /// Ledger the supply was measured at.
    pub ledger: u32,
    pub posted_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Status {
    /// Lowest coverage across fresh reports; 0 when there is none.
    pub coverage_bps: u32,
    /// Age of the newest report; `u64::MAX` when there is none.
    pub newest_age_secs: u64,
    /// Attestors with a report inside the max age.
    pub attestor_count: u32,
    /// Highest minus lowest coverage across fresh reports.
    pub disagreement_bps: u32,
    /// Fresh witnessed supply above a fresh report's stated liabilities
    /// by more than the tolerance.
    pub supply_mismatch: bool,
    /// Age of the witnessed supply; `u64::MAX` when there is none.
    /// Stale supply counts as missing, never as zero.
    pub supply_age_secs: u64,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    Asset(Address),
    Attestors(Address),
    Reports(Address, Address),
    Supply(Address),
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetRegistered {
    #[topic]
    pub asset: Address,
    pub asset_admin: Address,
    pub unit: Symbol,
    pub max_age_secs: u64,
    pub mismatch_tolerance_bps: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttestorAdded {
    #[topic]
    pub asset: Address,
    #[topic]
    pub attestor: Address,
    pub at: u64,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttestorRemoved {
    #[topic]
    pub asset: Address,
    #[topic]
    pub attestor: Address,
    pub at: u64,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WitnessSet {
    #[topic]
    pub asset: Address,
    pub witness: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupplyPosted {
    #[topic]
    pub asset: Address,
    pub amount: i128,
    pub ledger: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReportPosted {
    #[topic]
    pub asset: Address,
    #[topic]
    pub attestor: Address,
    pub reserves: i128,
    pub liabilities: i128,
    pub as_of: u64,
    pub ledger: u32,
    pub sha256: BytesN<32>,
}

#[contract]
pub struct Registry;

#[contractimpl]
impl Registry {
    pub fn __constructor(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
        bump_instance(&env);
    }

    /// Contract interface version.
    pub fn version() -> u32 {
        0
    }

    pub fn admin(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).unwrap()
    }

    /// Registers an asset (its token contract address). Contract admin
    /// only in v0; issuers claim their asset via stellar.toml in T1.
    pub fn register_asset(
        env: Env,
        asset: Address,
        asset_admin: Address,
        unit: Symbol,
        max_age_secs: u64,
        mismatch_tolerance_bps: u32,
    ) -> Result<(), Error> {
        Self::admin(env.clone()).require_auth();
        let key = DataKey::Asset(asset.clone());
        if env.storage().persistent().has(&key) {
            return Err(Error::AlreadyRegistered);
        }
        if max_age_secs == 0 || mismatch_tolerance_bps as i128 > BPS {
            return Err(Error::InvalidPolicy);
        }
        let cfg = AssetConfig {
            asset_admin: asset_admin.clone(),
            unit: unit.clone(),
            max_age_secs,
            mismatch_tolerance_bps,
            witness: None,
            registered_at: env.ledger().timestamp(),
        };
        set_persistent(&env, &key, &cfg);
        set_persistent(
            &env,
            &DataKey::Attestors(asset.clone()),
            &Vec::<Attestor>::new(&env),
        );
        bump_instance(&env);
        AssetRegistered {
            asset,
            asset_admin,
            unit,
            max_age_secs,
            mismatch_tolerance_bps,
        }
        .publish(&env);
        Ok(())
    }

    /// Adds an attestor. Asset admin only.
    // TODO(T1): timelock attestor changes (docs/ARCHITECTURE.md §6).
    pub fn add_attestor(env: Env, asset: Address, attestor: Address) -> Result<(), Error> {
        let cfg = load_asset(&env, &asset)?;
        cfg.asset_admin.require_auth();
        let key = DataKey::Attestors(asset.clone());
        let mut list: Vec<Attestor> = env.storage().persistent().get(&key).unwrap();
        if list.iter().any(|a| a.address == attestor) {
            return Err(Error::AlreadyAttestor);
        }
        if list.len() >= MAX_ATTESTORS {
            return Err(Error::TooManyAttestors);
        }
        let at = env.ledger().timestamp();
        list.push_back(Attestor {
            address: attestor.clone(),
            added_at: at,
        });
        set_persistent(&env, &key, &list);
        bump_asset(&env, &asset);
        AttestorAdded {
            asset,
            attestor,
            at,
        }
        .publish(&env);
        Ok(())
    }

    /// Removes an attestor; its reports stay readable but no longer count
    /// toward status. Asset admin only.
    // TODO(T1): timelock attestor changes (docs/ARCHITECTURE.md §6).
    pub fn remove_attestor(env: Env, asset: Address, attestor: Address) -> Result<(), Error> {
        let cfg = load_asset(&env, &asset)?;
        cfg.asset_admin.require_auth();
        let key = DataKey::Attestors(asset.clone());
        let mut list: Vec<Attestor> = env.storage().persistent().get(&key).unwrap();
        let i = list
            .iter()
            .position(|a| a.address == attestor)
            .ok_or(Error::NotAttestor)?;
        list.remove(i as u32);
        set_persistent(&env, &key, &list);
        bump_asset(&env, &asset);
        AttestorRemoved {
            asset,
            attestor,
            at: env.ledger().timestamp(),
        }
        .publish(&env);
        Ok(())
    }

    /// Sets the asset's supply witness (one in v0). Clears the previous
    /// witness's post, so supply reads as missing until the new one posts.
    pub fn set_supply_witness(env: Env, asset: Address, witness: Address) -> Result<(), Error> {
        let mut cfg = load_asset(&env, &asset)?;
        cfg.asset_admin.require_auth();
        cfg.witness = Some(witness.clone());
        set_persistent(&env, &DataKey::Asset(asset.clone()), &cfg);
        env.storage()
            .persistent()
            .remove(&DataKey::Supply(asset.clone()));
        bump_asset(&env, &asset);
        WitnessSet { asset, witness }.publish(&env);
        Ok(())
    }

    /// Posts the asset's supply measured at `ledger`. Witness only.
    pub fn post_supply(env: Env, asset: Address, amount: i128, ledger: u32) -> Result<(), Error> {
        let cfg = load_asset(&env, &asset)?;
        let witness = cfg.witness.ok_or(Error::NoWitness)?;
        witness.require_auth();
        if amount < 0 {
            return Err(Error::NegativeAmount);
        }
        if ledger > env.ledger().sequence() {
            return Err(Error::FutureLedger);
        }
        let key = DataKey::Supply(asset.clone());
        if let Some(prev) = env.storage().persistent().get::<_, SupplyPost>(&key) {
            if ledger <= prev.ledger {
                return Err(Error::StaleLedger);
            }
        }
        let post = SupplyPost {
            amount,
            ledger,
            posted_at: env.ledger().timestamp(),
        };
        set_persistent(&env, &key, &post);
        bump_asset(&env, &asset);
        SupplyPosted {
            asset,
            amount,
            ledger,
        }
        .publish(&env);
        Ok(())
    }

    /// Posts a reserve report. The attestor must be registered on the asset
    /// and sign the call.
    pub fn post_report(
        env: Env,
        asset: Address,
        attestor: Address,
        report: Report,
    ) -> Result<(), Error> {
        attestor.require_auth();
        let cfg = load_asset(&env, &asset)?;
        let list: Vec<Attestor> = env
            .storage()
            .persistent()
            .get(&DataKey::Attestors(asset.clone()))
            .unwrap();
        if !list.iter().any(|a| a.address == attestor) {
            return Err(Error::NotAttestor);
        }
        if report.reserves_unit != cfg.unit {
            return Err(Error::UnitMismatch);
        }
        if report.reserves < 0 || report.liabilities < 0 {
            return Err(Error::NegativeAmount);
        }
        if report.as_of > env.ledger().timestamp() {
            return Err(Error::FutureTimestamp);
        }
        if report.ledger > env.ledger().sequence() {
            return Err(Error::FutureLedger);
        }
        let key = DataKey::Reports(asset.clone(), attestor.clone());
        let mut history: Vec<ReportEntry> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(&env));
        if let Some(last) = history.last() {
            if report.as_of <= last.report.as_of {
                return Err(Error::OutOfOrder);
            }
        }
        history.push_back(ReportEntry {
            report: report.clone(),
            posted_at: env.ledger().timestamp(),
            posted_ledger: env.ledger().sequence(),
        });
        while history.len() > HISTORY_LEN {
            history.pop_front();
        }
        set_persistent(&env, &key, &history);
        bump_asset(&env, &asset);
        ReportPosted {
            asset,
            attestor,
            reserves: report.reserves,
            liabilities: report.liabilities,
            as_of: report.as_of,
            ledger: report.ledger,
            sha256: report.sha256,
        }
        .publish(&env);
        Ok(())
    }

    /// Status under the asset's own max age and mismatch tolerance. An
    /// unregistered asset reads as missing data.
    pub fn status(env: Env, asset: Address) -> Status {
        match env
            .storage()
            .persistent()
            .get::<_, AssetConfig>(&DataKey::Asset(asset.clone()))
        {
            Some(cfg) => compute_status(&env, &asset, cfg.max_age_secs, cfg.mismatch_tolerance_bps),
            None => missing(),
        }
    }

    /// Status under a caller's policy (used by the guard).
    pub fn status_with(
        env: Env,
        asset: Address,
        max_age_secs: u64,
        mismatch_tolerance_bps: u32,
    ) -> Status {
        if !env
            .storage()
            .persistent()
            .has(&DataKey::Asset(asset.clone()))
        {
            return missing();
        }
        compute_status(&env, &asset, max_age_secs, mismatch_tolerance_bps)
    }

    pub fn asset(env: Env, asset: Address) -> Option<AssetConfig> {
        env.storage().persistent().get(&DataKey::Asset(asset))
    }

    pub fn attestors(env: Env, asset: Address) -> Vec<Attestor> {
        env.storage()
            .persistent()
            .get(&DataKey::Attestors(asset))
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Kept history, oldest first.
    pub fn reports(env: Env, asset: Address, attestor: Address) -> Vec<ReportEntry> {
        env.storage()
            .persistent()
            .get(&DataKey::Reports(asset, attestor))
            .unwrap_or_else(|| Vec::new(&env))
    }

    pub fn supply(env: Env, asset: Address) -> Option<SupplyPost> {
        env.storage().persistent().get(&DataKey::Supply(asset))
    }
}

fn missing() -> Status {
    Status {
        coverage_bps: 0,
        newest_age_secs: u64::MAX,
        attestor_count: 0,
        disagreement_bps: 0,
        supply_mismatch: false,
        supply_age_secs: u64::MAX,
    }
}

fn compute_status(env: &Env, asset: &Address, max_age_secs: u64, tolerance_bps: u32) -> Status {
    let now = env.ledger().timestamp();
    let mut s = missing();

    // Witnessed supply counts only while fresh; stale is missing, never zero.
    let mut supply: Option<i128> = None;
    if let Some(p) = env
        .storage()
        .persistent()
        .get::<_, SupplyPost>(&DataKey::Supply(asset.clone()))
    {
        s.supply_age_secs = now.saturating_sub(p.posted_at);
        if s.supply_age_secs <= max_age_secs {
            supply = Some(p.amount);
        }
    }

    let attestors: Vec<Attestor> = env
        .storage()
        .persistent()
        .get(&DataKey::Attestors(asset.clone()))
        .unwrap_or_else(|| Vec::new(env));
    let mut min_cov = u32::MAX;
    let mut max_cov = 0u32;
    for a in attestors.iter() {
        let history: Option<Vec<ReportEntry>> = env
            .storage()
            .persistent()
            .get(&DataKey::Reports(asset.clone(), a.address));
        let Some(last) = history.and_then(|h| h.last()) else {
            continue;
        };
        let r = last.report;
        let age = now.saturating_sub(r.as_of);
        s.newest_age_secs = s.newest_age_secs.min(age);
        if age > max_age_secs {
            continue;
        }
        s.attestor_count += 1;
        let cov = coverage_bps(r.reserves, r.liabilities, supply);
        min_cov = min_cov.min(cov);
        max_cov = max_cov.max(cov);
        if let Some(w) = supply {
            s.supply_mismatch |= mismatch(r.liabilities, w, tolerance_bps);
        }
    }
    if s.attestor_count > 0 {
        s.coverage_bps = min_cov;
        s.disagreement_bps = max_cov - min_cov;
    }
    s
}

/// reserves ÷ max(stated liabilities, witnessed supply), rounded down.
/// 0 when the denominator is 0: nothing to measure is not "covered".
pub fn coverage_bps(reserves: i128, liabilities: i128, supply: Option<i128>) -> u32 {
    let denom = liabilities.max(supply.unwrap_or(0));
    if denom <= 0 || reserves <= 0 {
        return 0;
    }
    match reserves.checked_mul(BPS) {
        Some(n) => u32::try_from(n / denom).unwrap_or(u32::MAX),
        // Only reachable for reserves above i128::MAX / 10_000.
        None => u32::MAX,
    }
}

/// Witnessed supply above stated liabilities by more than `tolerance_bps`
/// of the supply. Exactly at the tolerance is not a mismatch.
pub fn mismatch(liabilities: i128, supply: i128, tolerance_bps: u32) -> bool {
    if supply <= liabilities {
        return false;
    }
    let gap = supply - liabilities;
    match (
        gap.checked_mul(BPS),
        supply.checked_mul(tolerance_bps as i128),
    ) {
        (Some(g), Some(t)) => g > t,
        _ => true,
    }
}

fn load_asset(env: &Env, asset: &Address) -> Result<AssetConfig, Error> {
    env.storage()
        .persistent()
        .get(&DataKey::Asset(asset.clone()))
        .ok_or(Error::NotRegistered)
}

fn set_persistent<V: soroban_sdk::IntoVal<Env, soroban_sdk::Val>>(
    env: &Env,
    key: &DataKey,
    value: &V,
) {
    let max = env.storage().max_ttl();
    env.storage().persistent().set(key, value);
    env.storage().persistent().extend_ttl(key, max, max);
}

/// Keeps every entry `status` reads alive on each write to the asset.
fn bump_asset(env: &Env, asset: &Address) {
    let max = env.storage().max_ttl();
    let p = env.storage().persistent();
    for key in [
        DataKey::Asset(asset.clone()),
        DataKey::Attestors(asset.clone()),
    ] {
        p.extend_ttl(&key, max, max);
    }
    let attestors: Vec<Attestor> = p
        .get(&DataKey::Attestors(asset.clone()))
        .unwrap_or_else(|| Vec::new(env));
    for a in attestors.iter() {
        let k = DataKey::Reports(asset.clone(), a.address);
        if p.has(&k) {
            p.extend_ttl(&k, max, max);
        }
    }
    let k = DataKey::Supply(asset.clone());
    if p.has(&k) {
        p.extend_ttl(&k, max, max);
    }
    bump_instance(env);
}

fn bump_instance(env: &Env) {
    let max = env.storage().max_ttl();
    env.storage().instance().extend_ttl(max, max);
}

#[cfg(test)]
mod test;
