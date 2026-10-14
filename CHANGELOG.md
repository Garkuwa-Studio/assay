# Changelog

All notable changes to this project are documented here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow
`v0.x.y` tags.

## [Unreleased]

### Added
- Evidence pack (demo sprint D5): `docs/SEP-DRAFT-reserve-report.md`
  (signed reserve reports: RFC 8785 canonical form, SEP-53 Ed25519
  signature, issuer and attestor keys in stellar.toml), `docs/PRIOR-ART.md`,
  `docs/PILOT.md`, `docs/DEMO-SCRIPT.md`, `docs/ARCHITECTURE.md` (public
  copy) and the `docs/audit/` skeleton with the first dependency scans.
- `@garkuwa/assay-sdk`: `canonicalize`, `signReport`, `verifyReport`, with
  test vectors checked against `stellar message verify`.
- Scripts: `sign-report` (signs via the stellar CLI keystore) and
  `check-links` (every link, transaction and contract ID in the docs).
- Makefile: `e2e` runs both testnet demos; `audit-scan` saves dated scans.

### Changed
- `soroban-sdk` 29.0.0 → 29.0.1: the 29.0.0 env crates were yanked (see
  `docs/audit/remediation-log.md`).
- CI builds the JS packages before typechecking.
- Assay Guard v0 (`contracts/guard`): `is_healthy(asset, policy)` returns
  `(bool, reason)` with the first failing rule (NoReport, Stale,
  TooFewAttestors, Undercovered, Disagreement, SupplyMismatch); `evaluate`
  exported for inline use. Read-only; policies that would let missing data
  through are refused.
- Demo pool (`contracts/demo-pool`), TESTNET DEMO ONLY: deposit and borrow;
  `borrow` asks the guard and, when unhealthy, moves nothing and records
  `Rejected(reason)`.
- `scripts/src/demo-circuit-breaker.ts`: the D4 story on testnet, recorded in
  `docs/DEMO-RUN.md`.
- Registry v0 (`contracts/registry`): asset registration, attestors (max 20
  per asset), one supply witness, reserve reports with bounded history (10
  per attestor), and a conservative `status` / `status_with`: lowest
  coverage across fresh attestors, liabilities = max(stated, fresh witnessed
  supply), missing or stale data reads as coverage 0. Events on every write;
  TTL extended on writes. Attestor timelock is stubbed for T1.
- `scripts/src/demo-registry.ts`: testnet demo; run recorded in
  `docs/TESTNET.md` and `docs/audit/e2e-receipts/`.
- Repository scaffold: registry, guard and demo-pool contract placeholders,
  `assay` CLI placeholder, `@garkuwa/assay-sdk` placeholder, CI and release
  workflow stubs.
