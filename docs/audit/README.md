# Audit

Evidence for reviewers: what was deployed and run, what the scans found,
and what was done about it. Nothing here is an external audit; that is
planned for T3 ([ARCHITECTURE.md §9](../ARCHITECTURE.md)).

| Folder or file | What it holds | How it is produced |
|---|---|---|
| [`e2e-receipts/`](e2e-receipts/) | One JSON receipt per testnet run: contract IDs, wasm SHA-256, every transaction hash with a stellar.expert link, the status read, and whether it matched what was expected. | `make e2e` (testnet only) |
| [`dep-scan/`](dep-scan/) | Dated outputs of `cargo audit`, `cargo clippy -D warnings` and `pnpm audit`. | `make audit-scan` |
| [`remediation-log.md`](remediation-log.md) | Every finding, the decision taken and the fix, with dates. | By hand, after each scan |

## Receipts so far

| Date | Run | Receipt |
|---|---|---|
| 2026-10-13 | Registry v0: register, two attestors, supply, two reports, status | [registry-2026-10-13.json](e2e-receipts/registry-2026-10-13.json) |
| 2026-10-14 | Guard and demo pool: the four-step circuit breaker | [circuit-breaker-2026-10-14.json](e2e-receipts/circuit-breaker-2026-10-14.json) |

Human-readable versions: [TESTNET.md](../TESTNET.md) and
[DEMO-RUN.md](../DEMO-RUN.md).

## Planned

- Scout (CoinFabrik's Soroban analyzer) and a STRIDE threat model: T2.
- External audit of the registry and guard before mainnet: T3.
