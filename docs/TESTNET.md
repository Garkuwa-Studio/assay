# Testnet evidence

Contract IDs and transaction hashes from testnet runs. Every hash below was
checked to return `SUCCESS` from Stellar RPC and to open on stellar.expert
before it was listed. Never list a hash that does not resolve.

All figures are **demo values** on **testnet**, not a real asset's reserves.

## Registry v0 (demo sprint D3), 2026-10-13

Protocol 29, soroban-sdk 29.0.0, stellar-cli 28.1.0.

| | |
|---|---|
| Registry contract | [`CBUEGKQJNSJI2FXEIEK6B3RK52MZLNZECDEQPOFER32Z5UCSMCSJRRRF`](https://stellar.expert/explorer/testnet/contract/CBUEGKQJNSJI2FXEIEK6B3RK52MZLNZECDEQPOFER32Z5UCSMCSJRRRF) |
| Wasm SHA-256 | `92f7663d28014231f5d04df01703512ce6d16d652144cffcd81a11bd6bfec865` |
| Demo asset | `ADEMO`, issuer `GDTB33U75MGMILME2RWXBL5CUPZP4W43Y225OLWUOZ7D2YNSBQTUOPKE` |
| Demo asset token contract | [`CC22EDRUYLGMSF4YOP77GKP6O6SQQT6WY2VP6VHNHALTOBPGXNIKRWTB`](https://stellar.expert/explorer/testnet/contract/CC22EDRUYLGMSF4YOP77GKP6O6SQQT6WY2VP6VHNHALTOBPGXNIKRWTB) |
| Contract admin | `GDG23ZJYA4DPRZIWFMEKNDCMFZ2NGT56CM63D4JHU57W4FSOEN4QAGWN` |
| Asset admin (demo issuer) | `GDTB33U75MGMILME2RWXBL5CUPZP4W43Y225OLWUOZ7D2YNSBQTUOPKE` |
| Attestor A | `GBQK5IGFFJUEMJJ6G4ZRU2N4ZBUUEV2OFYUQSEXLOI5NMV6FHJ6F7ZPC` |
| Attestor B | `GAXIC2DSICRV5WBIY24B6TBOOP6JXN3ZWOT5QQ2CIWQQCVVH7ADL6EA6` |
| Supply witness | `GAVPDR3LVSJLKAO75FOLTRGVNQAOHOHVZUDKUPOE5IVWU2XRB4WIZZPJ` |

### Transactions

| Step | Transaction |
|---|---|
| Deploy ADEMO token contract | [`020daf0d…`](https://stellar.expert/explorer/testnet/tx/020daf0deb2b808668dee52d4eee27182507b540981c3ce9a47c61c2ee639c56) |
| Upload registry wasm | [`f1836fe3…`](https://stellar.expert/explorer/testnet/tx/f1836fe39e87a2afdb02a10e8a78d4738ae75cba1f33af35175ed5ea491ad379) |
| Deploy registry (constructor sets admin) | [`e5575a56…`](https://stellar.expert/explorer/testnet/tx/e5575a562259020d1bad2ed74282b9997ef2f65d202199f3b82fcf9c7ab921cf) |
| `register_asset` ADEMO, unit USD, max age 35 days, tolerance 50 bps | [`40b73719…`](https://stellar.expert/explorer/testnet/tx/40b73719f45e96319e265bc8647f55dbe49ff6036f3ace017fdca302ed371b5b) |
| `add_attestor` A | [`e5f2c736…`](https://stellar.expert/explorer/testnet/tx/e5f2c73639c3bae007e2555a8f787cf441c5be09feffe444608683adea1ea1a2) |
| `add_attestor` B | [`53530b26…`](https://stellar.expert/explorer/testnet/tx/53530b26a33e9c57aea427469a4d9de24d2ec63dc55967ab0ad394775172ebda) |
| `set_supply_witness` | [`febc241e…`](https://stellar.expert/explorer/testnet/tx/febc241e5c9c8dce473944fdf23b3bf947fd8c41a8380b2ce2c6158a4210a79d) |
| `post_supply` 1,000,000 ADEMO at ledger 5181119 | [`be6b3d5e…`](https://stellar.expert/explorer/testnet/tx/be6b3d5e36e8cf1709ee4e335a27a183f4683a560442d94bf4bd05134a3327da) |
| `post_report` attestor A: reserves 1,010,000 USD (101%) | [`4ae73fa4…`](https://stellar.expert/explorer/testnet/tx/4ae73fa4159b27313e2c9d3859e9e384b9a318e5e02d3ca0ea2e715b4c5166d1) |
| `post_report` attestor B: reserves 995,000 USD (99.5%) | [`29e90be6…`](https://stellar.expert/explorer/testnet/tx/29e90be6970bf6df443be32d2edaf917b2aa4d9df1bd9d3fabf857644663715c) |

The reports' documents are in [`testnet/reports/`](testnet/reports/). The
SHA-256 stored on-chain for each matches its file.

### Status read

`status(ADEMO)` straight after the run (a read, so no transaction):

| Field | Value | Why |
|---|---|---|
| `coverage_bps` | 9950 | Lowest of 10100 (A) and 9950 (B) |
| `attestor_count` | 2 | Both reports inside 35 days |
| `disagreement_bps` | 150 | 10100 − 9950 |
| `supply_mismatch` | false | Witnessed supply equals stated liabilities |
| `newest_age_secs` | 80 | Seconds since the reports' `as_of` |
| `supply_age_secs` | 20 | Seconds since the supply post |

Receipt: [`audit/e2e-receipts/registry-2026-10-13.json`](audit/e2e-receipts/registry-2026-10-13.json).

Reproduce: `pnpm --filter @garkuwa/assay-scripts demo:registry` (deploys a
new registry; needs the five testnet identities named in the script).
