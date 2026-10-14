extern crate std;

use super::*;
use assay_registry::{Registry, RegistryClient as RealRegistryClient, Report};
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Ledger as _},
    BytesN, String,
};

const DAY: u64 = 24 * 60 * 60;
const NOW: u64 = 1_800_000_000;
const SEQ: u32 = 1_000_000;
const ONE: i128 = 10_000_000;

fn healthy() -> RegistryStatus {
    RegistryStatus {
        coverage_bps: 10_100,
        newest_age_secs: DAY,
        attestor_count: 2,
        disagreement_bps: 100,
        supply_mismatch: false,
        supply_age_secs: 60,
    }
}

// --- evaluate: one test per reason -------------------------------------------

#[test]
fn ok_when_every_rule_passes() {
    assert_eq!(evaluate(&healthy(), &Policy::standard()), Reason::Ok);
}

#[test]
fn no_report() {
    let s = RegistryStatus {
        coverage_bps: 0,
        newest_age_secs: u64::MAX,
        attestor_count: 0,
        disagreement_bps: 0,
        supply_mismatch: false,
        supply_age_secs: u64::MAX,
    };
    assert_eq!(evaluate(&s, &Policy::standard()), Reason::NoReport);
}

#[test]
fn stale() {
    let s = RegistryStatus {
        coverage_bps: 0,
        newest_age_secs: 40 * DAY,
        attestor_count: 0,
        ..healthy()
    };
    assert_eq!(evaluate(&s, &Policy::standard()), Reason::Stale);
}

#[test]
fn too_few_attestors() {
    let p = Policy {
        min_attestors: 3,
        ..Policy::standard()
    };
    assert_eq!(evaluate(&healthy(), &p), Reason::TooFewAttestors);
}

#[test]
fn undercovered() {
    let s = RegistryStatus {
        coverage_bps: 9_200,
        ..healthy()
    };
    assert_eq!(evaluate(&s, &Policy::standard()), Reason::Undercovered);
}

#[test]
fn disagreement() {
    let s = RegistryStatus {
        disagreement_bps: 501,
        ..healthy()
    };
    assert_eq!(evaluate(&s, &Policy::standard()), Reason::Disagreement);
}

#[test]
fn supply_mismatch() {
    let s = RegistryStatus {
        supply_mismatch: true,
        ..healthy()
    };
    assert_eq!(evaluate(&s, &Policy::standard()), Reason::SupplyMismatch);
}

#[test]
fn first_failing_rule_wins() {
    // Stale and everything else wrong: Stale is reported.
    let s = RegistryStatus {
        coverage_bps: 0,
        newest_age_secs: 40 * DAY,
        attestor_count: 0,
        disagreement_bps: 9_000,
        supply_mismatch: true,
        supply_age_secs: u64::MAX,
    };
    assert_eq!(evaluate(&s, &Policy::standard()), Reason::Stale);
    // Undercovered beats disagreement and supply mismatch.
    let s = RegistryStatus {
        coverage_bps: 9_000,
        disagreement_bps: 9_000,
        supply_mismatch: true,
        ..healthy()
    };
    assert_eq!(evaluate(&s, &Policy::standard()), Reason::Undercovered);
}

// --- evaluate: boundaries ---------------------------------------------------------

#[test]
fn coverage_exactly_at_minimum_passes() {
    let at = RegistryStatus {
        coverage_bps: 10_000,
        ..healthy()
    };
    assert_eq!(evaluate(&at, &Policy::standard()), Reason::Ok);
    let below = RegistryStatus {
        coverage_bps: 9_999,
        ..healthy()
    };
    assert_eq!(evaluate(&below, &Policy::standard()), Reason::Undercovered);
}

#[test]
fn attestors_exactly_at_minimum_pass() {
    let p = Policy {
        min_attestors: 2,
        ..Policy::standard()
    };
    assert_eq!(evaluate(&healthy(), &p), Reason::Ok);
    let one = RegistryStatus {
        attestor_count: 1,
        disagreement_bps: 0,
        ..healthy()
    };
    assert_eq!(evaluate(&one, &p), Reason::TooFewAttestors);
    // One attestor under the standard policy is enough.
    assert_eq!(evaluate(&one, &Policy::standard()), Reason::Ok);
}

#[test]
fn disagreement_exactly_at_limit_passes() {
    let at = RegistryStatus {
        disagreement_bps: 500,
        ..healthy()
    };
    assert_eq!(evaluate(&at, &Policy::standard()), Reason::Ok);
}

#[test]
fn invalid_policies_refused() {
    let base = Policy::standard();
    assert_eq!(check_policy(&base), Ok(()));
    for p in [
        Policy {
            min_attestors: 0,
            ..base.clone()
        },
        Policy {
            max_age_secs: 0,
            ..base.clone()
        },
        Policy {
            min_coverage_bps: 0,
            ..base.clone()
        },
        Policy {
            max_supply_mismatch_bps: 10_001,
            ..base.clone()
        },
    ] {
        assert_eq!(check_policy(&p), Err(Error::InvalidPolicy));
    }
}

// --- against the real registry --------------------------------------------------------

struct T {
    env: Env,
    reg: RealRegistryClient<'static>,
    guard: GuardClient<'static>,
    asset: Address,
}

fn setup() -> T {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| {
        l.timestamp = NOW;
        l.sequence_number = SEQ;
    });
    let admin = Address::generate(&env);
    let reg_id = env.register(Registry, (&admin,));
    let reg = RealRegistryClient::new(&env, &reg_id);
    let guard_id = env.register(Guard, (&reg_id,));
    let guard = GuardClient::new(&env, &guard_id);
    let asset = Address::generate(&env);
    reg.register_asset(
        &asset,
        &Address::generate(&env),
        &symbol_short!("USD"),
        &(35 * DAY),
        &50,
    );
    T {
        env,
        reg,
        guard,
        asset,
    }
}

impl T {
    fn attestor(&self) -> Address {
        let a = Address::generate(&self.env);
        self.reg.add_attestor(&self.asset, &a);
        a
    }

    fn post(&self, a: &Address, reserves: i128, liabilities: i128, as_of: u64) {
        self.reg.post_report(
            &self.asset,
            a,
            &Report {
                reserves,
                reserves_unit: symbol_short!("USD"),
                liabilities,
                as_of,
                ledger: SEQ - 1,
                uri: String::from_str(&self.env, "https://issuer.example/r.json"),
                sha256: BytesN::from_array(&self.env, &[1; 32]),
            },
        );
    }

    fn check(&self, p: &Policy) -> (bool, Reason) {
        self.guard.is_healthy(&self.asset, p)
    }
}

#[test]
fn registry_status_type_matches() {
    // Decoding the real registry's Status into RegistryStatus must work and
    // agree field by field.
    let t = setup();
    let a = t.attestor();
    t.post(&a, 101 * ONE, 100 * ONE, NOW - DAY);
    let real = t.reg.status_with(&t.asset, &(35 * DAY), &50);
    let mirrored =
        RegistryClient::new(&t.env, &t.reg.address).status_with(&t.asset, &(35 * DAY), &50);
    assert_eq!(
        (
            real.coverage_bps,
            real.newest_age_secs,
            real.attestor_count,
            real.disagreement_bps,
            real.supply_mismatch,
            real.supply_age_secs
        ),
        (
            mirrored.coverage_bps,
            mirrored.newest_age_secs,
            mirrored.attestor_count,
            mirrored.disagreement_bps,
            mirrored.supply_mismatch,
            mirrored.supply_age_secs
        )
    );
}

#[test]
fn unregistered_asset_is_no_report() {
    let t = setup();
    let other = Address::generate(&t.env);
    assert_eq!(
        t.guard.is_healthy(&other, &Policy::standard()),
        (false, Reason::NoReport)
    );
}

#[test]
fn registered_without_reports_is_no_report() {
    let t = setup();
    t.attestor();
    assert_eq!(t.check(&Policy::standard()), (false, Reason::NoReport));
}

#[test]
fn healthy_then_undercovered_then_stale() {
    let t = setup();
    let a = t.attestor();
    let p = Policy {
        max_age_secs: 90,
        ..Policy::standard()
    };
    t.post(&a, 101 * ONE, 100 * ONE, NOW);
    assert_eq!(t.check(&p), (true, Reason::Ok));
    t.env.ledger().with_mut(|l| l.timestamp += 10);
    t.post(&a, 92 * ONE, 100 * ONE, NOW + 10);
    assert_eq!(t.check(&p), (false, Reason::Undercovered));
    t.env.ledger().with_mut(|l| l.timestamp += 10);
    t.post(&a, 1004 * ONE, 1000 * ONE, NOW + 20);
    assert_eq!(t.check(&p), (true, Reason::Ok));
    // Exactly 90 s old: still fresh. One second later: stale.
    t.env.ledger().with_mut(|l| l.timestamp = NOW + 20 + 90);
    assert_eq!(t.check(&p), (true, Reason::Ok));
    t.env.ledger().with_mut(|l| l.timestamp += 1);
    assert_eq!(t.check(&p), (false, Reason::Stale));
}

#[test]
fn policy_max_age_reaches_the_registry() {
    // The registry's own max age is 35 days; the guard's policy decides.
    let t = setup();
    let a = t.attestor();
    t.post(&a, 101 * ONE, 100 * ONE, NOW - 2 * DAY);
    assert_eq!(t.check(&Policy::standard()), (true, Reason::Ok));
    let p = Policy {
        max_age_secs: DAY,
        ..Policy::standard()
    };
    assert_eq!(t.check(&p), (false, Reason::Stale));
}

#[test]
fn policy_tolerance_reaches_the_registry() {
    let t = setup();
    let a = t.attestor();
    let w = Address::generate(&t.env);
    t.reg.set_supply_witness(&t.asset, &w);
    // Witnessed 1,000,000 vs stated 990,000: a 1% gap.
    t.reg.post_supply(&t.asset, &(1_000_000 * ONE), &(SEQ - 1));
    t.post(&a, 1_100_000 * ONE, 990_000 * ONE, NOW);
    assert_eq!(
        t.check(&Policy::standard()),
        (false, Reason::SupplyMismatch)
    );
    let p = Policy {
        max_supply_mismatch_bps: 100,
        ..Policy::standard()
    };
    assert_eq!(t.check(&p), (true, Reason::Ok));
}

#[test]
fn invalid_policy_rejected_by_contract() {
    let t = setup();
    let p = Policy {
        min_attestors: 0,
        ..Policy::standard()
    };
    assert_eq!(
        t.guard.try_is_healthy(&t.asset, &p),
        Err(Ok(Error::InvalidPolicy))
    );
}

#[test]
fn needs_no_authorization() {
    let t = setup();
    let a = t.attestor();
    t.post(&a, 101 * ONE, 100 * ONE, NOW);
    t.check(&Policy::standard());
    // No address had to sign: the guard cannot act for anyone.
    assert!(t.env.auths().is_empty());
}

#[test]
fn version_is_zero() {
    let t = setup();
    assert_eq!(t.guard.version(), 0);
    assert_eq!(t.guard.registry(), t.reg.address);
}
