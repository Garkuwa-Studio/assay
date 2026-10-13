# Assay

**Open proof of reserves and circuit breakers for real-world assets and
stablecoins on Stellar.** By [Garkuwa Labs](https://github.com/Garkuwa-Studio).

[Website]([URL]) · [Board]([URL]/board) ·
[Architecture (PDF)]([URL]/assay-architecture.pdf) · [Docs]([URL]/docs) ·
[Demo video]([URL])

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
| `contracts/registry` | Multi-attestor reserve reports; conservative status | [Demo sprint] → [SCF T1] |
| `contracts/guard` | `is_healthy(asset, policy)` circuit breaker | [Demo sprint] → [SCF T2] |
| `assay` CLI · `@garkuwa/assay-sdk` | Sign/post reports; check status; wallet badge | [SCF T1] |

Full design: [Technical architecture]([URL]/docs/architecture).

## Demo (testnet)

[Fill as evidence lands. Never list a hash that does not resolve.]

| Step | Evidence |
|---|---|
| Registry deployed | [contract ID] |
| Asset registered, two attestors added | [tx] |
| Supply witness posted | [tx] |
| Report at 101% posted → status healthy | [tx] |
| Borrow on demo pool — succeeded | [tx] |
| Report at 92% posted → borrow rejected (UNDERCOVERED) | [tx] |
| Fresh report → borrow succeeded | [tx] |
| No new report → borrow rejected (STALE) | [tx] |

Board on real mainnet data: [URL]/board · Census method: docs/CENSUS-v0.md

## Quickstart

```bash
cargo install assay-stellar         # once published
assay check CETES-<ISSUER> --network mainnet
assay report sign report.json --key <attestor-key>
assay report post report.json --network testnet
```

## Repository layout

```
assay/
├── contracts/registry     reserve registry
├── contracts/guard        circuit breaker
├── contracts/demo-pool    TESTNET DEMO ONLY
├── crates/assay-stellar   CLI (assay)
├── packages/sdk           @garkuwa/assay-sdk
├── scripts/               testnet demos
└── docs/                  architecture, census, testnet evidence, SEP draft,
                           prior art, pilots, audit/
```

Org: [assay-indexer](../../../assay-indexer) · [assay-board](../../../assay-board)
· [proposals](../../../proposals)

## Prior art

Assay builds alongside RedStone and Chainlink (data providers), Reflector
(prices), attest.so (attestations) and PaltaLabs (dashboards). See
[docs/PRIOR-ART.md](docs/PRIOR-ART.md).

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
