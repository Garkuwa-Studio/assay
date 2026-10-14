# Remediation log

One row per finding from `make audit-scan` or review. Newest first.

| Date | Source | Finding | Affects | Decision | Status |
|---|---|---|---|---|---|
| 2026-10-14 | `cargo audit` | `soroban-env-common`, `-guest`, `-host` and `-macros` 29.0.0 are **yanked**. Per the soroban-sdk 29.0.1 release notes, env 29.0.0 "erroneously contained the wasmi runtime from v28". | The runtime our local tests ran on. Not the testnet contracts: the network runs its own host and only executes our wasm. | Upgraded `soroban-sdk` 29.0.0 → **29.0.1** (env 29.0.1). All 71 contract tests, clippy and `stellar contract build` pass on it. Contracts deployed on 2026-10-13/14 were built with 29.0.0; the next deploy uses 29.0.1. | **Fixed** |
| 2026-10-14 | `cargo audit` | `paste` 1.0.15 unmaintained ([RUSTSEC-2024-0436](https://rustsec.org/advisories/RUSTSEC-2024-0436)). | A compile-time proc-macro reached only through `soroban-env-host` → `ark-*` crates (host side). Not in the contract wasm. No vulnerability is reported. | Accept; it comes from the Stellar SDK. Recheck at each SDK upgrade. | Accepted |
| 2026-10-14 | `cargo clippy -D warnings` | No findings. | — | — | Clean |
| 2026-10-14 | `pnpm audit` | No known vulnerabilities. | — | — | Clean |

Scan outputs: [`dep-scan/2026-10-14-*`](dep-scan/).
