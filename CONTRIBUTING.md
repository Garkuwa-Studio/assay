# Contributing to Assay

Thanks for helping. Assay is Apache-2.0 and accepts pull requests.

## Developer Certificate of Origin

Every commit must be signed off under the
[DCO](https://developercertificate.org/):

```bash
git commit -s -m "registry: add attestor removal"
```

This adds `Signed-off-by: Your Name <you@example.com>`, certifying you wrote
the change or have the right to submit it under the project licence.

## Setup

- Rust (version pinned in `rust-toolchain.toml`) with `wasm32v1-none`
- [stellar-cli](https://developers.stellar.org/docs/tools/cli) 28.x
- Node 24 LTS and pnpm 11 (`corepack enable`)

```bash
pnpm install
make build
make test
```

## Rules

- `make test` must pass: fmt, clippy (`-D warnings`), cargo test, lint,
  typecheck, JS tests.
- Changes to status logic need tests at the boundaries: exactly 100%
  coverage, maximum report age, one attestor, disagreement threshold.
  Missing or stale data must never produce a healthy status.
- `contracts/demo-pool` is a testnet demo only. Never add mainnet config.
- Never commit keys, seeds or `.env` files.
- Add a line to `CHANGELOG.md` under `[Unreleased]`.

## Security issues

Do not open a public issue. See [SECURITY.md](SECURITY.md).
