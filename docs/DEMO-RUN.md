# Circuit-breaker demo run

> **TESTNET DEMO ONLY.** The lending pool is a deliberately tiny demo (no
> interest, no liquidation, fixed 1:1 prices) that exists only to show Assay
> Guard. Figures are demo values, not a real asset's reserves.

Run on 2026-10-14 by `scripts/src/demo-circuit-breaker.ts`. Each borrow asks
Assay Guard `is_healthy(ADEMO, policy)` first. A rejected borrow moves
nothing and records the guard's reason in a successful transaction (the pool
returns `Rejected(reason)` and emits `borrow_rejected`), so every step has a
transaction to open.

Demo policy: minimum coverage 100%, max report age **90 seconds** (so the
stale step can happen live), at least 1 attestor, disagreement up to 5%,
supply mismatch up to 0.5%. Borrow limit 50% of collateral.

## The story

| Step | Result | Transactions (report, then borrow) |
|---|---|---|
| Fresh report at 101% | borrow succeeded | [`bda39ffb…`](https://stellar.expert/explorer/testnet/tx/bda39ffb24e5206e40d9714ade368b3527c94c323bc21143424632bfedcc8563), [`84fefd06…`](https://stellar.expert/explorer/testnet/tx/84fefd066c50f3627370fa75c8f48f58cdf19d9b6d5ce24e8623e02969ebdc71) |
| Attestor posts 92% | borrow rejected (UNDERCOVERED) | [`773163e0…`](https://stellar.expert/explorer/testnet/tx/773163e0283f3fd6cf5742daca99c6a6e257b6a782bade2e2f82234232171c7e), [`84d31e35…`](https://stellar.expert/explorer/testnet/tx/84d31e3575e94e9bad7d07d3462c2773f71c9d07e0cb7ec35b2cbf8621c09e3e) |
| Fresh report at 100.4% | borrow succeeded | [`2956390b…`](https://stellar.expert/explorer/testnet/tx/2956390b147a4e953d565259f350fded678dee11c7f11b82da054dd1c9f1076d), [`683abc8d…`](https://stellar.expert/explorer/testnet/tx/683abc8d34f922ec60d22930638f252c151f1ff787a787b4bfb5d3f36d2a8b52) |
| No new report past the 90-second max age | borrow rejected (STALE) | [`ec2c5847…`](https://stellar.expert/explorer/testnet/tx/ec2c58476bb4bb7a12f977dd97b6143b5fe3400b42231bd1d0ad0254c3cbfa15) |

Reasons are the guard's: `OK`, `NO_REPORT`, `STALE`, `TOO_FEW_ATTESTORS`, `UNDERCOVERED`, `DISAGREEMENT`, `SUPPLY_MISMATCH`.

## Checked after the run

- All 22 transactions return `SUCCESS` from Stellar RPC and open on
  stellar.expert.
- The two successful borrows' events include the DUSD `transfer`; the two
  rejected borrows emit only `borrow_rejected`, with reason `Undercovered`
  and `Stale`, and no transfer.
- Final position (`position(borrower)`): 1,000 ADEMO collateral, 200 DUSD
  debt (two borrows of 100).
- The ADEMO token contract was deployed during D3; its transaction is in
  [TESTNET.md](TESTNET.md).

## Contracts

| | |
|---|---|
| Registry | [`CBMTQWT7DZOPHQ6WJZN4OYZIFJ6XLSZYRCYJZVOWMWHMQR4TZ3MCOG4U`](https://stellar.expert/explorer/testnet/contract/CBMTQWT7DZOPHQ6WJZN4OYZIFJ6XLSZYRCYJZVOWMWHMQR4TZ3MCOG4U) |
| Assay Guard | [`CCVEIPYTY6RRVJZONFZL4WVGN66H5IU77H3HSSQJKH7I7WSDL6DIEKDV`](https://stellar.expert/explorer/testnet/contract/CCVEIPYTY6RRVJZONFZL4WVGN66H5IU77H3HSSQJKH7I7WSDL6DIEKDV) |
| Demo pool (TESTNET DEMO ONLY) | [`CCQP73IMYC67UUF7R4OIHMEWRZWMUZO2TQEKCTOFGYZ6NUFZT6JNKY33`](https://stellar.expert/explorer/testnet/contract/CCQP73IMYC67UUF7R4OIHMEWRZWMUZO2TQEKCTOFGYZ6NUFZT6JNKY33) |
| ADEMO (demo RWA token, collateral) | [`CC22EDRUYLGMSF4YOP77GKP6O6SQQT6WY2VP6VHNHALTOBPGXNIKRWTB`](https://stellar.expert/explorer/testnet/contract/CC22EDRUYLGMSF4YOP77GKP6O6SQQT6WY2VP6VHNHALTOBPGXNIKRWTB) |
| DUSD (demo stablecoin) | [`CDKWYXW5W5ZOK3FDZFHNGXHNOO6YISS7DBGCSDQTQINVMH7THUTDWDGL`](https://stellar.expert/explorer/testnet/contract/CDKWYXW5W5ZOK3FDZFHNGXHNOO6YISS7DBGCSDQTQINVMH7THUTDWDGL) |

## Setup transactions

| Step | Transactions |
|---|---|
| Deploy registry `CBMTQWT7DZOPHQ6WJZN4OYZIFJ6XLSZYRCYJZVOWMWHMQR4TZ3MCOG4U` | [`c9ea09dd…`](https://stellar.expert/explorer/testnet/tx/c9ea09dd7c59dee639cf6931c389004e9e3736328b6e33e005f1c639d36cac46) |
| Deploy guard `CCVEIPYTY6RRVJZONFZL4WVGN66H5IU77H3HSSQJKH7I7WSDL6DIEKDV` | [`178e373a…`](https://stellar.expert/explorer/testnet/tx/178e373a112bc1f6ead6d7d75099b805779704779abf0b769526538d93313c5b), [`2e7b3222…`](https://stellar.expert/explorer/testnet/tx/2e7b32225b520b5b2ab5d9574add96d94e14cf3510bae38abd44bdf3798aaf54) |
| ADEMO token contract `CC22EDRUYLGMSF4YOP77GKP6O6SQQT6WY2VP6VHNHALTOBPGXNIKRWTB` (collateral) | already deployed |
| DUSD token contract `CDKWYXW5W5ZOK3FDZFHNGXHNOO6YISS7DBGCSDQTQINVMH7THUTDWDGL` (demo stablecoin) | [`49832143…`](https://stellar.expert/explorer/testnet/tx/49832143b79b78f1c941297e1ab1ead7e967c0eb09c5d126413141e07d66d645) |
| Deploy demo pool `CCQP73IMYC67UUF7R4OIHMEWRZWMUZO2TQEKCTOFGYZ6NUFZT6JNKY33` (TESTNET DEMO ONLY) | [`39981565…`](https://stellar.expert/explorer/testnet/tx/39981565a1d6f851985ab593ed54179d627cd867e83c2953e3d1b4327eebb36c), [`93248a20…`](https://stellar.expert/explorer/testnet/tx/93248a20b2be6fd4555252012a3c25ae8ff376759f3c1452d9cf574c8963c19d) |
| Registry: register ADEMO (unit USD) | [`4688e7db…`](https://stellar.expert/explorer/testnet/tx/4688e7db7183b9dc2921f6797c825f8aa8495ee58c3f26717b578515a5dfbacd) |
| Registry: add attestor | [`bd422369…`](https://stellar.expert/explorer/testnet/tx/bd42236981ae9c6de60e291222783dfe45e17a027b105d1c07e56c670431d825) |
| Registry: set supply witness | [`36ae64a4…`](https://stellar.expert/explorer/testnet/tx/36ae64a43d731066403068e4500ed77d147beb6e9e3bf19ed18143eb83c58c75) |
| Registry: witness posts supply 1,000,000 ADEMO | [`14b903e2…`](https://stellar.expert/explorer/testnet/tx/14b903e28913397dbe655a70084915dcb50c28311f5086ea95b0847ca14d8111) |
| Borrower trustline ADEMO | [`f1387896…`](https://stellar.expert/explorer/testnet/tx/f1387896adbfc3a662cdb9e92c7bd93e676b521ad494220480d5743f1661342a) |
| Borrower trustline DUSD | [`af6760b8…`](https://stellar.expert/explorer/testnet/tx/af6760b864706bb5332a0385e3feef24d2dc660caaa929bc72693d13ca1a5417) |
| Issuer mints 1,000 ADEMO to the borrower | [`33dec226…`](https://stellar.expert/explorer/testnet/tx/33dec22683e2e59496c05298a4c80a5ee5198c75f9e743604fd661205b01fe8d) |
| Fund the pool with 100,000 DUSD | [`016129dd…`](https://stellar.expert/explorer/testnet/tx/016129ddad01416f01eea0fa4bc5e14d35ca2dcb2841e16a218316248a146557) |
| Borrower deposits 1,000 ADEMO as collateral | [`aa27d961…`](https://stellar.expert/explorer/testnet/tx/aa27d9618d48b3411d9980a0d52fe162a08a38248084a5a70a93d49a2188d2a9) |

Receipt: [`audit/e2e-receipts/circuit-breaker-2026-10-14.json`](audit/e2e-receipts/circuit-breaker-2026-10-14.json).

Reproduce: `pnpm --filter @garkuwa/assay-scripts demo:circuit-breaker`
(deploys new contracts; needs the testnet identities named in the script).
