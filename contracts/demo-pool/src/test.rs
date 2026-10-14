extern crate std;

use super::*;
use assay_guard::Guard;
use assay_registry::{Registry, RegistryClient, Report};
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Events as _, Ledger as _},
    token::{StellarAssetClient, TokenClient},
    BytesN, Event as _,
};

const NOW: u64 = 1_800_000_000;
const SEQ: u32 = 1_000_000;
const ONE: i128 = 10_000_000;

struct T {
    env: Env,
    reg: RegistryClient<'static>,
    pool: DemoPoolClient<'static>,
    asset: Address,
    attestor: Address,
    user: Address,
    collateral: TokenClient<'static>,
    debt: TokenClient<'static>,
}

fn demo_policy() -> Policy {
    Policy {
        min_coverage_bps: 10_000,
        max_age_secs: 90,
        min_attestors: 1,
        max_disagreement_bps: 500,
        max_supply_mismatch_bps: 50,
    }
}

fn setup() -> T {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| {
        l.timestamp = NOW;
        l.sequence_number = SEQ;
    });
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);

    let collateral = env.register_stellar_asset_contract_v2(issuer.clone());
    let debt = env.register_stellar_asset_contract_v2(admin.clone());
    let asset = collateral.address();

    let reg_id = env.register(Registry, (&admin,));
    let reg = RegistryClient::new(&env, &reg_id);
    reg.register_asset(&asset, &issuer, &symbol_short!("USD"), &(35 * 86_400), &50);
    let attestor = Address::generate(&env);
    reg.add_attestor(&asset, &attestor);

    let guard_id = env.register(Guard, (&reg_id,));
    let pool_id = env.register(
        DemoPool,
        (
            &admin,
            &guard_id,
            &asset,
            &debt.address(),
            demo_policy(),
            5_000u32,
        ),
    );
    let pool = DemoPoolClient::new(&env, &pool_id);

    let user = Address::generate(&env);
    StellarAssetClient::new(&env, &asset).mint(&user, &(1_000 * ONE));
    StellarAssetClient::new(&env, &debt.address()).mint(&pool_id, &(100_000 * ONE));

    T {
        reg,
        pool,
        asset: asset.clone(),
        attestor,
        user,
        collateral: TokenClient::new(&env, &asset),
        debt: TokenClient::new(&env, &debt.address()),
        env,
    }
}

impl T {
    fn report(&self, reserves: i128, liabilities: i128) {
        let now = self.env.ledger().timestamp();
        self.reg.post_report(
            &self.asset,
            &self.attestor,
            &Report {
                reserves,
                reserves_unit: symbol_short!("USD"),
                liabilities,
                as_of: now,
                ledger: SEQ - 1,
                uri: String::from_str(&self.env, "https://issuer.example/r.json"),
                sha256: BytesN::from_array(&self.env, &[3; 32]),
            },
        );
    }

    fn advance(&self, secs: u64) {
        self.env.ledger().with_mut(|l| l.timestamp += secs);
    }

    fn borrow(&self, amount: i128) -> BorrowResult {
        self.pool.borrow(&self.user, &amount)
    }
}

#[test]
fn says_it_is_a_testnet_demo() {
    let t = setup();
    let n = t.pool.notice();
    assert_eq!(n, String::from_str(&t.env, NOTICE));
    assert!(NOTICE.starts_with("TESTNET DEMO ONLY"));
    assert_eq!(t.pool.version(), 0);
}

#[test]
fn deposit_moves_collateral() {
    let t = setup();
    t.pool.deposit(&t.user, &(1_000 * ONE));
    // Auths and events cover the last call only, so check them first.
    assert_eq!(t.env.auths()[0].0, t.user);
    assert_eq!(t.collateral.balance(&t.user), 0);
    assert_eq!(t.collateral.balance(&t.pool.address), 1_000 * ONE);
    assert_eq!(t.pool.position(&t.user).collateral, 1_000 * ONE);
}

#[test]
fn healthy_borrow_pays_out() {
    let t = setup();
    t.pool.deposit(&t.user, &(1_000 * ONE));
    t.report(101 * ONE, 100 * ONE);
    assert_eq!(t.borrow(100 * ONE), BorrowResult::Borrowed);
    assert_eq!(t.env.auths()[0].0, t.user);
    assert_eq!(
        t.env.events().all().filter_by_contract(&t.pool.address),
        std::vec![Borrowed {
            user: t.user.clone(),
            amount: 100 * ONE,
        }
        .to_xdr(&t.env, &t.pool.address)]
    );
    assert_eq!(t.debt.balance(&t.user), 100 * ONE);
    assert_eq!(t.pool.position(&t.user).debt, 100 * ONE);
}

fn assert_rejected(t: &T, reason: Reason) {
    let before = (t.debt.balance(&t.user), t.pool.position(&t.user));
    assert_eq!(t.borrow(10 * ONE), BorrowResult::Rejected(reason));
    // The rejection is the pool's only event; no token transfer happened.
    assert_eq!(
        t.env.events().all().filter_by_contract(&t.pool.address),
        std::vec![BorrowRejected {
            user: t.user.clone(),
            amount: 10 * ONE,
            reason,
        }
        .to_xdr(&t.env, &t.pool.address)]
    );
    assert_eq!(
        t.env
            .events()
            .all()
            .filter_by_contract(&t.debt.address)
            .events()
            .len(),
        0
    );
    assert_eq!((t.debt.balance(&t.user), t.pool.position(&t.user)), before);
}

#[test]
fn rejected_with_no_report() {
    let t = setup();
    t.pool.deposit(&t.user, &(1_000 * ONE));
    assert_rejected(&t, Reason::NoReport);
}

#[test]
fn rejected_when_undercovered() {
    let t = setup();
    t.pool.deposit(&t.user, &(1_000 * ONE));
    t.report(92 * ONE, 100 * ONE);
    assert_rejected(&t, Reason::Undercovered);
}

#[test]
fn rejected_when_stale() {
    let t = setup();
    t.pool.deposit(&t.user, &(1_000 * ONE));
    t.report(101 * ONE, 100 * ONE);
    t.advance(91);
    assert_rejected(&t, Reason::Stale);
}

#[test]
fn rejected_on_supply_mismatch() {
    let t = setup();
    t.pool.deposit(&t.user, &(1_000 * ONE));
    let w = Address::generate(&t.env);
    t.reg.set_supply_witness(&t.asset, &w);
    t.reg.post_supply(&t.asset, &(110 * ONE), &(SEQ - 1));
    t.report(120 * ONE, 100 * ONE);
    assert_rejected(&t, Reason::SupplyMismatch);
}

#[test]
fn borrow_limit_enforced() {
    let t = setup();
    t.pool.deposit(&t.user, &(1_000 * ONE));
    t.report(101 * ONE, 100 * ONE);
    // 50% of 1,000.
    assert_eq!(t.borrow(500 * ONE), BorrowResult::Borrowed);
    assert_eq!(t.pool.try_borrow(&t.user, &1), Err(Ok(Error::OverLimit)));
    assert_eq!(
        t.pool.try_borrow(&t.user, &0),
        Err(Ok(Error::InvalidAmount))
    );
}

#[test]
#[should_panic]
fn bad_config_refused() {
    let env = Env::default();
    let a = Address::generate(&env);
    env.register(DemoPool, (&a, &a, &a, &a, demo_policy(), 0u32));
}

/// The D4 story end to end, as the testnet script runs it.
#[test]
fn circuit_breaker_story() {
    let t = setup();
    t.pool.deposit(&t.user, &(1_000 * ONE));

    // 1. Fresh report at 101% → borrow succeeds.
    t.report(1_010_000 * ONE, 1_000_000 * ONE);
    assert_eq!(t.borrow(100 * ONE), BorrowResult::Borrowed);

    // 2. Attestor posts 92% → rejected, UNDERCOVERED.
    t.advance(10);
    t.report(920_000 * ONE, 1_000_000 * ONE);
    assert_eq!(
        t.borrow(100 * ONE),
        BorrowResult::Rejected(Reason::Undercovered)
    );

    // 3. Fresh report at 100.4% → borrow succeeds.
    t.advance(10);
    t.report(1_004_000 * ONE, 1_000_000 * ONE);
    assert_eq!(t.borrow(100 * ONE), BorrowResult::Borrowed);

    // 4. No new report past the 90 s max age → rejected, STALE.
    t.advance(91);
    assert_eq!(t.borrow(100 * ONE), BorrowResult::Rejected(Reason::Stale));

    assert_eq!(t.debt.balance(&t.user), 200 * ONE);
    assert_eq!(t.pool.position(&t.user).debt, 200 * ONE);
}
