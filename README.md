# Assay

**Open proof of reserves and circuit breakers for real-world assets and
stablecoins on Stellar.** By [Garkuwa Labs](https://github.com/Garkuwa-Studio).

Board: **TODO** (address pending, domain moving to Cloudflare) ·
[Architecture](docs/ARCHITECTURE.md) · [Docs](docs/) ·
[Testnet demo](docs/DEMO-RUN.md) · Demo video: **TODO** (not recorded yet)

Stellar holds billions in tokenized real-world assets and stablecoins. Each
one promises it is backed by real money, bonds or loans. Stellar's SEP-1
standard gives issuers a field for a link to their reserve proof — but
nothing checks those links, tracks how old they are, or lets a smart
contract react when backing goes missing.

Assay makes every reserve claim on Stellar checkable, by people and by
contracts:

- **Board** — a public page for every anchored asset: is there a reserve
  report, how old is it, what does it say, does it cover the supply?
- **Registry** — a Soroban contract where independent attestors post signed
  reserve reports.
- **Guard** — circuit breakers so lending markets and vaults stop new
  exposure when backing is missing, stale or short.
- **Standard** — a draft SEP for machine-readable, signed reserve reports.

Assay is not an oracle and not a rating agency. It works with any data
provider and publishes facts with sources.

> ⚠️ **Status: testnet, not audited.** The board shows figures as stated by
> issuers and attestors; it is not an audit or investment advice.

---

## How it works

| Component | What it does | Status |
|---|---|---|
| `assay-indexer` | Census of anchored assets, SEP-1 fields, report checks, supply | [Demo sprint] → [SCF T1] |
| `assay-board` | Public board and docs | [Demo sprint] → [SCF T1] |
| `contracts/registry` | Multi-attestor reserve reports; conservative status | v0 on testnet ([TESTNET.md](docs/TESTNET.md)) → [SCF T1] |
| `contracts/guard` | `is_healthy(asset, policy)` circuit breaker | v0 on testnet ([DEMO-RUN.md](docs/DEMO-RUN.md)) → [SCF T2] |
| `contracts/demo-pool` | TESTNET DEMO ONLY lending pool that calls the guard | Testnet demo |
| `@garkuwa/assay-sdk` | Sign and verify reserve reports (SEP draft) | Built (signing); status lookups [SCF T1] |
| `assay` CLI | Check status; sign and post reports | [SCF T1] |

Full design: [Technical architecture](docs/ARCHITECTURE.md).

## Demo (testnet)

Every hash below returns `SUCCESS` from Stellar RPC and opens on
stellar.expert. Figures are demo values on testnet.

**Registry v0** (2026-10-13), full list in [docs/TESTNET.md](docs/TESTNET.md):

| Step | Evidence |
|---|---|
| Registry deployed | [`CBUEGKQJ…`](https://stellar.expert/explorer/testnet/contract/CBUEGKQJNSJI2FXEIEK6B3RK52MZLNZECDEQPOFER32Z5UCSMCSJRRRF) |
| Asset registered, two attestors added | [`40b73719…`](https://stellar.expert/explorer/testnet/tx/40b73719f45e96319e265bc8647f55dbe49ff6036f3ace017fdca302ed371b5b), [`e5f2c736…`](https://stellar.expert/explorer/testnet/tx/e5f2c73639c3bae007e2555a8f787cf441c5be09feffe444608683adea1ea1a2), [`53530b26…`](https://stellar.expert/explorer/testnet/tx/53530b26a33e9c57aea427469a4d9de24d2ec63dc55967ab0ad394775172ebda) |
| Supply witness posted | [`be6b3d5e…`](https://stellar.expert/explorer/testnet/tx/be6b3d5e36e8cf1709ee4e335a27a183f4683a560442d94bf4bd05134a3327da) |
| Reports posted at 101% and 99.5% | [`4ae73fa4…`](https://stellar.expert/explorer/testnet/tx/4ae73fa4159b27313e2c9d3859e9e384b9a318e5e02d3ca0ea2e715b4c5166d1), [`29e90be6…`](https://stellar.expert/explorer/testnet/tx/29e90be6970bf6df443be32d2edaf917b2aa4d9df1bd9d3fabf857644663715c) |
| Status read (no transaction) | coverage 9950 bps (the lower report), 2 attestors, disagreement 150 bps, no supply mismatch |

**Circuit breaker** (2026-10-14), full run in [docs/DEMO-RUN.md](docs/DEMO-RUN.md).
Guard [`CCVEIPYT…`](https://stellar.expert/explorer/testnet/contract/CCVEIPYTY6RRVJZONFZL4WVGN66H5IU77H3HSSQJKH7I7WSDL6DIEKDV),
demo pool [`CCQP73IM…`](https://stellar.expert/explorer/testnet/contract/CCQP73IMYC67UUF7R4OIHMEWRZWMUZO2TQEKCTOFGYZ6NUFZT6JNKY33)
(TESTNET DEMO ONLY), on a fresh registry
[`CBMTQWT7…`](https://stellar.expert/explorer/testnet/contract/CBMTQWT7DZOPHQ6WJZN4OYZIFJ6XLSZYRCYJZVOWMWHMQR4TZ3MCOG4U):

| Step | Report | Borrow |
|---|---|---|
| Report at 101% → borrow succeeded | [`bda39ffb…`](https://stellar.expert/explorer/testnet/tx/bda39ffb24e5206e40d9714ade368b3527c94c323bc21143424632bfedcc8563) | [`84fefd06…`](https://stellar.expert/explorer/testnet/tx/84fefd066c50f3627370fa75c8f48f58cdf19d9b6d5ce24e8623e02969ebdc71) |
| Report at 92% → borrow rejected (UNDERCOVERED) | [`773163e0…`](https://stellar.expert/explorer/testnet/tx/773163e0283f3fd6cf5742daca99c6a6e257b6a782bade2e2f82234232171c7e) | [`84d31e35…`](https://stellar.expert/explorer/testnet/tx/84d31e3575e94e9bad7d07d3462c2773f71c9d07e0cb7ec35b2cbf8621c09e3e) |
| Report at 100.4% → borrow succeeded | [`2956390b…`](https://stellar.expert/explorer/testnet/tx/2956390b147a4e953d565259f350fded678dee11c7f11b82da054dd1c9f1076d) | [`683abc8d…`](https://stellar.expert/explorer/testnet/tx/683abc8d34f922ec60d22930638f252c151f1ff787a787b4bfb5d3f36d2a8b52) |
| No new report past the 90 s max age → borrow rejected (STALE) | — | [`ec2c5847…`](https://stellar.expert/explorer/testnet/tx/ec2c58476bb4bb7a12f977dd97b6143b5fe3400b42231bd1d0ad0254c3cbfa15) |

Reproduce both runs: `make e2e` (testnet only; deploys fresh contracts).

Board on real mainnet data: **TODO** (address pending) · Census method:
[docs/CENSUS-v0.md](docs/CENSUS-v0.md)

## Quickstart

```bash
corepack enable && pnpm install
make test        # fmt, clippy, contract tests, JS lint/build/typecheck/tests
make e2e         # testnet demos (needs funded testnet identities)
make audit-scan  # cargo audit, clippy, pnpm audit → docs/audit/dep-scan/

# Sign a reserve report with a key in your stellar CLI keystore
pnpm -r build
pnpm --filter @garkuwa/assay-scripts sign-report "$(realpath report.json)" --key <identity>
```

The `assay` CLI (`assay check`, `assay report sign/post`) is planned for
SCF T1.

## Repository layout

```
assay/
├── contracts/registry     reserve registry
├── contracts/guard        circuit breaker
├── contracts/demo-pool    TESTNET DEMO ONLY
├── crates/assay-stellar   CLI (assay)
├── packages/sdk           @garkuwa/assay-sdk
├── scripts/               testnet demos, sign-report
└── docs/                  architecture, census, testnet evidence, SEP draft,
                           prior art, pilots, demo script, audit/
```

Related repositories: [assay-indexer](https://github.com/Garkuwa-Studio/assay-indexer) ·
[assay-board](https://github.com/Garkuwa-Studio/assay-board)

## Prior art

Assay builds alongside RedStone and Chainlink (oracles), Reflector
(prices), attest.so (attestations), PaltaLabs (Soroswap, DeFindex) and
Octarine (RWA liquidity). See [docs/PRIOR-ART.md](docs/PRIOR-ART.md).

## For issuers

Want your asset's page to show a structured, signed report? See
[docs/PILOT.md](docs/PILOT.md) or open an issue tagged `issuer`. Found an
error on the board? Use the correction link on your asset page.

## Contributing, security, licence

PRs welcome ([CONTRIBUTING.md](CONTRIBUTING.md), DCO). Report
vulnerabilities privately per [SECURITY.md](SECURITY.md). Apache-2.0.
"Assay" and "Garkuwa Labs" are names of Garkuwa Labs.

---

*Garkuwa* is Hausa for **shield**. An *assay* is the test that proves what a
metal really contains.
