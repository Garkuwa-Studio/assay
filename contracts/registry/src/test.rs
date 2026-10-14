extern crate std;

use super::*;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Events as _, Ledger as _},
    Event as _,
};

const DAY: u64 = 24 * 60 * 60;
const NOW: u64 = 1_800_000_000;
const SEQ: u32 = 1_000_000;
/// 1 token with 7 decimals.
const ONE: i128 = 10_000_000;

struct T {
    env: Env,
    id: Address,
    c: RegistryClient<'static>,
    asset: Address,
    asset_admin: Address,
}

fn setup() -> T {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| {
        l.timestamp = NOW;
        l.sequence_number = SEQ;
    });
    let admin = Address::generate(&env);
    let id = env.register(Registry, (&admin,));
    let c = RegistryClient::new(&env, &id);
    let asset = Address::generate(&env);
    let asset_admin = Address::generate(&env);
    c.register_asset(
        &asset,
        &asset_admin,
        &symbol_short!("USD"),
        &DEFAULT_MAX_AGE_SECS,
        &DEFAULT_MISMATCH_TOLERANCE_BPS,
    );
    T {
        env,
        id,
        c,
        asset,
        asset_admin,
    }
}

impl T {
    fn attestor(&self) -> Address {
        let a = Address::generate(&self.env);
        self.c.add_attestor(&self.asset, &a);
        a
    }

    fn report(&self, reserves: i128, liabilities: i128, as_of: u64) -> Report {
        Report {
            reserves,
            reserves_unit: symbol_short!("USD"),
            liabilities,
            as_of,
            ledger: SEQ - 10,
            uri: String::from_str(&self.env, "https://issuer.example/reserves.json"),
            sha256: BytesN::from_array(&self.env, &[7; 32]),
        }
    }

    fn post(&self, attestor: &Address, reserves: i128, liabilities: i128, as_of: u64) {
        self.c.post_report(
            &self.asset,
            attestor,
            &self.report(reserves, liabilities, as_of),
        );
    }

    fn witness_posts(&self, amount: i128) -> Address {
        let w = Address::generate(&self.env);
        self.c.set_supply_witness(&self.asset, &w);
        self.c.post_supply(&self.asset, &amount, &(SEQ - 1));
        w
    }

    fn advance(&self, secs: u64) {
        self.env.ledger().with_mut(|l| {
            l.timestamp += secs;
            l.sequence_number += (secs / 5) as u32;
        });
    }

    fn status(&self) -> Status {
        self.c.status(&self.asset)
    }
}

// --- basics ---------------------------------------------------------------

#[test]
fn version_is_zero() {
    let t = setup();
    assert_eq!(t.c.version(), 0);
}

#[test]
fn register_twice_rejected() {
    let t = setup();
    let r =
        t.c.try_register_asset(&t.asset, &t.asset_admin, &symbol_short!("USD"), &DAY, &50);
    assert_eq!(r, Err(Ok(Error::AlreadyRegistered)));
}

#[test]
fn register_rejects_bad_policy() {
    let t = setup();
    let other = Address::generate(&t.env);
    let usd = symbol_short!("USD");
    assert_eq!(
        t.c.try_register_asset(&other, &t.asset_admin, &usd, &0, &50),
        Err(Ok(Error::InvalidPolicy))
    );
    assert_eq!(
        t.c.try_register_asset(&other, &t.asset_admin, &usd, &DAY, &10_001),
        Err(Ok(Error::InvalidPolicy))
    );
}

#[test]
fn register_needs_contract_admin_auth() {
    let t = setup();
    let other = Address::generate(&t.env);
    t.c.register_asset(&other, &t.asset_admin, &symbol_short!("USD"), &DAY, &50);
    assert_eq!(t.env.auths()[0].0, t.c.admin());
}

// --- attestors --------------------------------------------------------------

#[test]
fn unregistered_attestor_rejected() {
    let t = setup();
    let stranger = Address::generate(&t.env);
    let r =
        t.c.try_post_report(&t.asset, &stranger, &t.report(ONE, ONE, NOW));
    assert_eq!(r, Err(Ok(Error::NotAttestor)));
    assert_eq!(t.status(), missing());
}

#[test]
fn report_needs_attestor_auth() {
    let t = setup();
    let a = t.attestor();
    t.post(&a, ONE, ONE, NOW);
    assert_eq!(t.env.auths()[0].0, a);
}

#[test]
fn attestor_changes_need_asset_admin_auth() {
    let t = setup();
    let a = t.attestor();
    assert_eq!(t.env.auths()[0].0, t.asset_admin);
    t.c.remove_attestor(&t.asset, &a);
    assert_eq!(t.env.auths()[0].0, t.asset_admin);
}

#[test]
fn attestor_added_and_removed_are_recorded() {
    let t = setup();
    let a = t.attestor();
    let list = t.c.attestors(&t.asset);
    assert_eq!(list.len(), 1);
    assert_eq!(list.get(0).unwrap().added_at, NOW);
    assert_eq!(
        t.c.try_add_attestor(&t.asset, &a),
        Err(Ok(Error::AlreadyAttestor))
    );
    t.c.remove_attestor(&t.asset, &a);
    assert_eq!(t.c.attestors(&t.asset).len(), 0);
    assert_eq!(
        t.c.try_remove_attestor(&t.asset, &a),
        Err(Ok(Error::NotAttestor))
    );
}

#[test]
fn attestor_limit() {
    let t = setup();
    for _ in 0..MAX_ATTESTORS {
        t.attestor();
    }
    let extra = Address::generate(&t.env);
    assert_eq!(
        t.c.try_add_attestor(&t.asset, &extra),
        Err(Ok(Error::TooManyAttestors))
    );
}

#[test]
fn removed_attestor_no_longer_counts() {
    let t = setup();
    let a = t.attestor();
    let b = t.attestor();
    t.post(&a, 2 * ONE, ONE, NOW);
    t.post(&b, ONE, ONE, NOW);
    t.c.remove_attestor(&t.asset, &b);
    let s = t.status();
    assert_eq!(s.attestor_count, 1);
    assert_eq!(s.coverage_bps, 20_000);
    // History stays readable.
    assert_eq!(t.c.reports(&t.asset, &b).len(), 1);
    assert_eq!(
        t.c.try_post_report(&t.asset, &b, &t.report(ONE, ONE, NOW)),
        Err(Ok(Error::NotAttestor))
    );
}

#[test]
#[ignore = "T1: attestor changes are timelocked; v0 applies them at once"]
fn attestor_change_is_timelocked() {
    // T1: add_attestor schedules the change; it takes effect after the
    // timelock, and the pending change is readable meanwhile.
}

// --- report validation --------------------------------------------------------

#[test]
fn report_checks() {
    let t = setup();
    let a = t.attestor();
    let mut r = t.report(ONE, ONE, NOW);
    r.reserves_unit = symbol_short!("EUR");
    assert_eq!(
        t.c.try_post_report(&t.asset, &a, &r),
        Err(Ok(Error::UnitMismatch))
    );
    assert_eq!(
        t.c.try_post_report(&t.asset, &a, &t.report(-1, ONE, NOW)),
        Err(Ok(Error::NegativeAmount))
    );
    assert_eq!(
        t.c.try_post_report(&t.asset, &a, &t.report(ONE, -1, NOW)),
        Err(Ok(Error::NegativeAmount))
    );
    assert_eq!(
        t.c.try_post_report(&t.asset, &a, &t.report(ONE, ONE, NOW + 1)),
        Err(Ok(Error::FutureTimestamp))
    );
    let mut r = t.report(ONE, ONE, NOW);
    r.ledger = SEQ + 1;
    assert_eq!(
        t.c.try_post_report(&t.asset, &a, &r),
        Err(Ok(Error::FutureLedger))
    );
    let other = Address::generate(&t.env);
    assert_eq!(
        t.c.try_post_report(&other, &a, &t.report(ONE, ONE, NOW)),
        Err(Ok(Error::NotRegistered))
    );
}

#[test]
fn reports_must_move_forward() {
    let t = setup();
    let a = t.attestor();
    t.post(&a, ONE, ONE, NOW - DAY);
    assert_eq!(
        t.c.try_post_report(&t.asset, &a, &t.report(ONE, ONE, NOW - DAY)),
        Err(Ok(Error::OutOfOrder))
    );
    assert_eq!(
        t.c.try_post_report(&t.asset, &a, &t.report(ONE, ONE, NOW - 2 * DAY)),
        Err(Ok(Error::OutOfOrder))
    );
    t.post(&a, ONE, ONE, NOW);
}

#[test]
fn history_is_bounded() {
    let t = setup();
    let a = t.attestor();
    for i in 0..(HISTORY_LEN as u64 + 3) {
        t.post(&a, ONE, ONE, NOW - 100 + i);
    }
    let h = t.c.reports(&t.asset, &a);
    assert_eq!(h.len(), HISTORY_LEN);
    // Oldest dropped, newest last.
    assert_eq!(h.get(0).unwrap().report.as_of, NOW - 100 + 3);
    assert_eq!(h.last().unwrap().report.as_of, NOW - 100 + 12);
    assert_eq!(h.last().unwrap().posted_ledger, SEQ);
}

// --- coverage -----------------------------------------------------------------

#[test]
fn coverage_exactly_100() {
    let t = setup();
    let a = t.attestor();
    t.post(&a, 1_000_000 * ONE, 1_000_000 * ONE, NOW);
    assert_eq!(t.status().coverage_bps, 10_000);
}

#[test]
fn coverage_above_100() {
    let t = setup();
    let a = t.attestor();
    t.post(&a, 1_010_000 * ONE, 1_000_000 * ONE, NOW);
    assert_eq!(t.status().coverage_bps, 10_100);
}

#[test]
fn coverage_below_100() {
    let t = setup();
    let a = t.attestor();
    t.post(&a, 995_000 * ONE, 1_000_000 * ONE, NOW);
    assert_eq!(t.status().coverage_bps, 9_950);
}

#[test]
fn coverage_rounds_down() {
    // 99.999% must not round up to 100%.
    assert_eq!(coverage_bps(99_999, 100_000, None), 9_999);
    assert_eq!(coverage_bps(1, 3, None), 3_333);
}

#[test]
fn coverage_edge_values() {
    assert_eq!(coverage_bps(ONE, 0, None), 0);
    assert_eq!(coverage_bps(0, ONE, None), 0);
    assert_eq!(coverage_bps(ONE, 0, Some(0)), 0);
    assert_eq!(coverage_bps(i128::MAX, 1, None), u32::MAX);
    assert_eq!(
        coverage_bps(i128::MAX / 10_000, i128::MAX / 10_000, None),
        10_000
    );
}

#[test]
fn coverage_uses_larger_of_stated_and_witnessed_supply() {
    let t = setup();
    let a = t.attestor();
    // Stated liabilities 1.0M, witnessed supply 1.1M → 1.1M is used.
    t.witness_posts(1_100_000 * ONE);
    t.post(&a, 1_100_000 * ONE, 1_000_000 * ONE, NOW);
    assert_eq!(t.status().coverage_bps, 10_000);
    // Witnessed below stated → stated is used.
    assert_eq!(coverage_bps(ONE, 2 * ONE, Some(ONE)), 5_000);
}

#[test]
fn coverage_is_min_across_fresh_attestors() {
    let t = setup();
    let a = t.attestor();
    let b = t.attestor();
    t.post(&a, 1_010_000 * ONE, 1_000_000 * ONE, NOW);
    t.post(&b, 995_000 * ONE, 1_000_000 * ONE, NOW);
    let s = t.status();
    assert_eq!(s.attestor_count, 2);
    assert_eq!(s.coverage_bps, 9_950);
}

#[test]
fn one_attestor() {
    let t = setup();
    let a = t.attestor();
    t.post(&a, 1_010_000 * ONE, 1_000_000 * ONE, NOW - DAY);
    let s = t.status();
    assert_eq!(s.attestor_count, 1);
    assert_eq!(s.coverage_bps, 10_100);
    assert_eq!(s.disagreement_bps, 0);
    assert_eq!(s.newest_age_secs, DAY);
}

#[test]
fn latest_report_is_the_one_counted() {
    let t = setup();
    let a = t.attestor();
    t.post(&a, 2 * ONE, ONE, NOW - DAY);
    t.post(&a, ONE / 2, ONE, NOW);
    assert_eq!(t.status().coverage_bps, 5_000);
}

// --- age and freshness ----------------------------------------------------------

#[test]
fn age_exactly_at_max_is_fresh() {
    let t = setup();
    let a = t.attestor();
    t.post(&a, ONE, ONE, NOW - DEFAULT_MAX_AGE_SECS);
    let s = t.status();
    assert_eq!(s.attestor_count, 1);
    assert_eq!(s.newest_age_secs, DEFAULT_MAX_AGE_SECS);
    assert_eq!(s.coverage_bps, 10_000);
}

#[test]
fn stale_report_excluded() {
    let t = setup();
    let a = t.attestor();
    t.post(&a, ONE, ONE, NOW - DEFAULT_MAX_AGE_SECS);
    t.advance(1);
    let s = t.status();
    assert_eq!(s.attestor_count, 0);
    assert_eq!(s.coverage_bps, 0);
    // The age is still shown, so a reader can see why.
    assert_eq!(s.newest_age_secs, DEFAULT_MAX_AGE_SECS + 1);
}

#[test]
fn stale_attestor_excluded_from_count_and_coverage() {
    let t = setup();
    let fresh = t.attestor();
    let stale = t.attestor();
    t.post(&stale, ONE / 2, ONE, NOW - DEFAULT_MAX_AGE_SECS - 1);
    t.post(&fresh, ONE, ONE, NOW - DAY);
    let s = t.status();
    assert_eq!(s.attestor_count, 1);
    assert_eq!(s.coverage_bps, 10_000);
    assert_eq!(s.newest_age_secs, DAY);
    assert_eq!(s.disagreement_bps, 0);
}

#[test]
fn status_with_applies_callers_max_age() {
    let t = setup();
    let a = t.attestor();
    t.post(&a, ONE, ONE, NOW - 10 * DAY);
    assert_eq!(
        t.c.status_with(&t.asset, &(10 * DAY), &50).attestor_count,
        1
    );
    assert_eq!(
        t.c.status_with(&t.asset, &(10 * DAY - 1), &50)
            .attestor_count,
        0
    );
}

// --- disagreement ----------------------------------------------------------------

#[test]
fn disagreement_computed() {
    let t = setup();
    let a = t.attestor();
    let b = t.attestor();
    let c = t.attestor();
    t.post(&a, 1_010_000 * ONE, 1_000_000 * ONE, NOW);
    t.post(&b, 995_000 * ONE, 1_000_000 * ONE, NOW);
    t.post(&c, 1_000_000 * ONE, 1_000_000 * ONE, NOW);
    let s = t.status();
    assert_eq!(s.disagreement_bps, 150);
    assert_eq!(s.coverage_bps, 9_950);
}

#[test]
fn identical_reports_have_no_disagreement() {
    let t = setup();
    let a = t.attestor();
    let b = t.attestor();
    t.post(&a, ONE, ONE, NOW);
    t.post(&b, ONE, ONE, NOW);
    assert_eq!(t.status().disagreement_bps, 0);
}

// --- supply witness ----------------------------------------------------------------

#[test]
fn supply_mismatch_flagged() {
    let t = setup();
    let a = t.attestor();
    // Stated 1.0M, witnessed 1.1M: 9.1% gap, tolerance 0.5%.
    t.witness_posts(1_100_000 * ONE);
    t.post(&a, 1_100_000 * ONE, 1_000_000 * ONE, NOW);
    assert!(t.status().supply_mismatch);
}

#[test]
fn supply_mismatch_tolerance_boundary() {
    // Supply 1,000,000; tolerance 50 bps = 5,000.
    assert!(!mismatch(995_000, 1_000_000, 50));
    assert!(mismatch(994_999, 1_000_000, 50));
    // Supply below stated is never a mismatch.
    assert!(!mismatch(1_100_000, 1_000_000, 0));
    assert!(!mismatch(1_000_000, 1_000_000, 0));
    // Zero tolerance: any excess is a mismatch.
    assert!(mismatch(999_999, 1_000_000, 0));
    // Overflow reads as a mismatch.
    assert!(mismatch(0, i128::MAX, 50));
}

#[test]
fn no_mismatch_within_tolerance() {
    let t = setup();
    let a = t.attestor();
    t.witness_posts(1_000_000 * ONE);
    t.post(&a, 1_000_000 * ONE, 995_000 * ONE, NOW);
    let s = t.status();
    assert!(!s.supply_mismatch);
    // Coverage still uses the larger supply.
    assert_eq!(s.coverage_bps, 10_000);
}

#[test]
fn stale_supply_counts_as_missing_not_zero() {
    let t = setup();
    let a = t.attestor();
    t.witness_posts(2_000_000 * ONE);
    t.advance(DEFAULT_MAX_AGE_SECS);
    t.post(
        &a,
        1_000_000 * ONE,
        1_000_000 * ONE,
        NOW + DEFAULT_MAX_AGE_SECS,
    );
    // Exactly at max age: supply is fresh, coverage uses it.
    let s = t.status();
    assert_eq!(s.supply_age_secs, DEFAULT_MAX_AGE_SECS);
    assert_eq!(s.coverage_bps, 5_000);
    assert!(s.supply_mismatch);
    // One second later: supply is missing; stated liabilities alone.
    t.advance(1);
    let s = t.status();
    assert_eq!(s.supply_age_secs, DEFAULT_MAX_AGE_SECS + 1);
    assert_eq!(s.coverage_bps, 10_000);
    assert!(!s.supply_mismatch);
}

#[test]
fn supply_post_checks() {
    let t = setup();
    assert_eq!(
        t.c.try_post_supply(&t.asset, &ONE, &(SEQ - 1)),
        Err(Ok(Error::NoWitness))
    );
    let w = t.witness_posts(ONE);
    assert_eq!(t.env.auths()[0].0, w);
    assert_eq!(
        t.c.try_post_supply(&t.asset, &ONE, &(SEQ - 1)),
        Err(Ok(Error::StaleLedger))
    );
    assert_eq!(
        t.c.try_post_supply(&t.asset, &ONE, &(SEQ - 2)),
        Err(Ok(Error::StaleLedger))
    );
    assert_eq!(
        t.c.try_post_supply(&t.asset, &ONE, &(SEQ + 1)),
        Err(Ok(Error::FutureLedger))
    );
    assert_eq!(
        t.c.try_post_supply(&t.asset, &-1, &SEQ),
        Err(Ok(Error::NegativeAmount))
    );
    t.c.post_supply(&t.asset, &(2 * ONE), &SEQ);
    let p = t.c.supply(&t.asset).unwrap();
    assert_eq!((p.amount, p.ledger, p.posted_at), (2 * ONE, SEQ, NOW));
}

#[test]
fn new_witness_clears_old_supply() {
    let t = setup();
    t.witness_posts(ONE);
    let w2 = Address::generate(&t.env);
    t.c.set_supply_witness(&t.asset, &w2);
    assert_eq!(t.c.supply(&t.asset), None);
    assert_eq!(t.status().supply_age_secs, u64::MAX);
    assert_eq!(t.c.asset(&t.asset).unwrap().witness, Some(w2));
}

// --- missing data -------------------------------------------------------------------

#[test]
fn missing_data_reads_unhealthy() {
    let t = setup();
    let expect = Status {
        coverage_bps: 0,
        newest_age_secs: u64::MAX,
        attestor_count: 0,
        disagreement_bps: 0,
        supply_mismatch: false,
        supply_age_secs: u64::MAX,
    };
    // Registered, no attestors.
    assert_eq!(t.status(), expect);
    // Attestor, no report.
    t.attestor();
    assert_eq!(t.status(), expect);
    // Unregistered asset.
    let other = Address::generate(&t.env);
    assert_eq!(t.c.status(&other), expect);
    assert_eq!(t.c.status_with(&other, &u64::MAX, &0), expect);
}

#[test]
fn zero_liabilities_and_no_supply_is_not_covered() {
    let t = setup();
    let a = t.attestor();
    t.post(&a, ONE, 0, NOW);
    let s = t.status();
    assert_eq!(s.attestor_count, 1);
    assert_eq!(s.coverage_bps, 0);
}

// --- events ---------------------------------------------------------------------------

#[test]
fn every_write_emits_an_event() {
    let t = setup();
    let usd = symbol_short!("USD");

    let other = Address::generate(&t.env);
    t.c.register_asset(&other, &t.asset_admin, &usd, &DAY, &50);
    assert_eq!(
        t.env.events().all(),
        std::vec![AssetRegistered {
            asset: other.clone(),
            asset_admin: t.asset_admin.clone(),
            unit: usd.clone(),
            max_age_secs: DAY,
            mismatch_tolerance_bps: 50,
        }
        .to_xdr(&t.env, &t.id)]
    );

    let a = Address::generate(&t.env);
    t.c.add_attestor(&t.asset, &a);
    assert_eq!(
        t.env.events().all(),
        std::vec![AttestorAdded {
            asset: t.asset.clone(),
            attestor: a.clone(),
            at: NOW,
        }
        .to_xdr(&t.env, &t.id)]
    );

    let w = Address::generate(&t.env);
    t.c.set_supply_witness(&t.asset, &w);
    assert_eq!(
        t.env.events().all(),
        std::vec![WitnessSet {
            asset: t.asset.clone(),
            witness: w,
        }
        .to_xdr(&t.env, &t.id)]
    );

    t.c.post_supply(&t.asset, &ONE, &SEQ);
    assert_eq!(
        t.env.events().all(),
        std::vec![SupplyPosted {
            asset: t.asset.clone(),
            amount: ONE,
            ledger: SEQ,
        }
        .to_xdr(&t.env, &t.id)]
    );

    let r = t.report(ONE, ONE, NOW);
    t.c.post_report(&t.asset, &a, &r);
    assert_eq!(
        t.env.events().all(),
        std::vec![ReportPosted {
            asset: t.asset.clone(),
            attestor: a.clone(),
            reserves: ONE,
            liabilities: ONE,
            as_of: NOW,
            ledger: r.ledger,
            sha256: r.sha256.clone(),
        }
        .to_xdr(&t.env, &t.id)]
    );

    t.c.remove_attestor(&t.asset, &a);
    assert_eq!(
        t.env.events().all(),
        std::vec![AttestorRemoved {
            asset: t.asset.clone(),
            attestor: a,
            at: NOW,
        }
        .to_xdr(&t.env, &t.id)]
    );
}

#[test]
fn reads_emit_no_events() {
    let t = setup();
    let a = t.attestor();
    t.post(&a, ONE, ONE, NOW);
    t.status();
    assert_eq!(t.env.events().all().events().len(), 0);
}

// --- storage lifetime ------------------------------------------------------------------

#[test]
fn writes_extend_ttl_to_max() {
    use soroban_sdk::testutils::storage::Persistent as _;
    let t = setup();
    let a = t.attestor();
    t.post(&a, ONE, ONE, NOW);
    let max = t.env.storage().max_ttl();
    t.env.as_contract(&t.id, || {
        let p = t.env.storage().persistent();
        assert_eq!(p.get_ttl(&DataKey::Asset(t.asset.clone())), max);
        assert_eq!(p.get_ttl(&DataKey::Attestors(t.asset.clone())), max);
        assert_eq!(
            p.get_ttl(&DataKey::Reports(t.asset.clone(), a.clone())),
            max
        );
    });
}
