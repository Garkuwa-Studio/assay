# Changelog

All notable changes to this project are documented here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow
`v0.x.y` tags.

## [Unreleased]

### Added
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
