# Security policy

Assay is pre-audit and runs on Stellar testnet only. Do not rely on it to
protect funds.

## Reporting a vulnerability

Report privately through GitHub's
[private vulnerability reporting](https://github.com/garkuwa-labs/assay/security/advisories/new).
Do not open a public issue, PR or discussion.

Please include affected component (registry, guard, CLI, SDK), version or
commit, steps to reproduce and impact.

We aim to acknowledge within 3 business days and to agree a disclosure
timeline with you. We credit reporters unless you ask us not to.

## Scope

In scope: `contracts/*`, `crates/assay-stellar`, `packages/sdk`.
Out of scope: `contracts/demo-pool` economics (it is a testnet demo), and
third-party data providers.
