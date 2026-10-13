# CLAUDE.md — assay (core repo)

Assay, by Garkuwa Labs: open proof of reserves and circuit breakers for RWAs
and stablecoins on Stellar. Core repo in a five-repo org: assay (this) ·
assay-indexer · assay-board · proposals · .github.

Contains: contracts/registry, contracts/guard, contracts/demo-pool (TESTNET
DEMO ONLY), crates/assay-stellar (CLI `assay`), packages/sdk
(@garkuwa/assay-sdk), scripts/, docs/.

## Specs (private, in ../proposals/specs/)
- 10-build-prompts.md — scaffold and post-award prompts.
- 11-demo-sprint-prompts.md — D0–D5 (pre-submission).
- 13-technical-architecture.md → public copy in docs/ARCHITECTURE.md.
- docs/SCF-SUBMISSION.md (after award) — acceptance criteria.

## Workflow (every task)
1. Plan first, then STOP and wait for explicit approval.
2. Verify against live sources, never memory: Stellar protocol, soroban-sdk,
   stellar-sdk, SEP-1 fields, OpenZeppelin smart-account policies, oracle
   offerings. If something doesn't exist, say so — never invent APIs.
3. Build to green: cargo fmt --check, clippy, cargo test, pnpm -r build/test.
4. Report: built, verified (commands + results), stubbed, what the human
   should check.

## Hard rules
- Status must be conservative: missing or stale data is never "healthy".
  Every change to status math needs tests at the boundaries (exactly 100%,
  max age, one attestor, disagreement threshold).
- Only registered attestors can post reports; attestor changes are
  recorded (timelocked from T1).
- The guard never moves funds. demo-pool is testnet-only and says so in
  code comments, README and UI.
- Never print, log or commit secrets. Testnet by default; mainnet only when
  the human says "mainnet" in this session.
- Don't push to git; the human commits.
- Names fixed: crates assay-guard, assay-sdk, assay-indexer, assay-stellar
  (binary `assay`); npm @garkuwa/assay-*. Do not use the `assay` or
  `assay-cli` crate names (taken).
- Release discipline: downstream repos use tagged releases only.

## Facts (verified in D0, 2026-10-12 — see docs/STACK-STATUS.md)
- SEP-1 v2.7.0: `attestation_of_reserve` is an OPTIONAL url field. Never
  describe it as required, and never treat its absence as a fault.
- Contracts cannot read classic-asset total supply via the SAC (no
  total_supply in SAC or SEP-41) → supply witness design
  (docs/D0-RECOMMENDATION.md).
- Oracles on Stellar: RedStone (SEP-40 prices, live), Reflector, Band, DIA
  (testnet), Pyth; Chainlink announced, Data Feeds live status unverified.
  No proof of reserve on Stellar from any provider.
- Blend v2: only pool-wide `set_status` is immediate; per-reserve disable
  has a 1-week timelock.
- Positioning: "Complement oracles, don't compete. Facts, not ratings."

D0 results: protocol 29 (mainnet + testnet) · SAC supply readable? no ·
census top-50: anchored 8 / with link 3 / link 2xx to automated request 0;
RWA supplement: anchored 6 / with link 6 / 2xx 5; structured reports 0 ·
"reserve-verified" means: not found (Stellar's Dune dashboard; question
drafted for Dev Discord)

## Status checklist (human updates)
Pre-submission
- [ ] Prompt 0 scaffold · [ ] D0 census + stack
- [ ] D3 registry on testnet (contract: ____)
- [ ] D4 guard + demo run recorded
- [ ] D5 evidence pack + SEP draft posted (link: ____)
- [ ] v0.1.0 tagged
Post-award
- [ ] T1-D2 · [ ] T1-D3 · [ ] T2-D1 · [ ] T2-D3 · [ ] T3-D2 · [ ] T3-D3
