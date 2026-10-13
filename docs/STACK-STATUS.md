# Stack status

Verified 2026-10-12 against live sources (demo sprint D0). Every claim links
its source. **Unverified** means we looked and could not confirm it from a
primary source; do not build on it without checking again.

## 1. Stack

### Network

| | Mainnet | Testnet | Source |
|---|---|---|---|
| Protocol | **29** | **29** | Horizon root `current_protocol_version`; RPC `getVersionInfo` / `getNetwork` |
| stellar-core | 29.0.0 | 29.0.0 | Horizon root `core_version` |
| Horizon | 29.0.0 | 29.0.0 | `https://horizon.stellar.org`, `https://horizon-testnet.stellar.org` |
| Stellar RPC | 29.0.0 (build 2026-09-22) | 29.0.0 | RPC `getVersionInfo` |
| Latest ledger at check | 64905297 | 5157737 | Horizon root |

### Toolchain

| Package | Version | Released | Notes |
|---|---|---|---|
| soroban-sdk (Rust) | 29.0.0 | 2026-10-07 | MSRV 1.91; repo pins Rust 1.94.0 ([crates.io](https://crates.io/crates/soroban-sdk)) |
| stellar-cli | 28.1.0 | 2026-09-26 | MSRV 1.93 ([crates.io](https://crates.io/crates/stellar-cli)) |
| @stellar/stellar-sdk (JS) | 17.2.1 | 2026-10-01 | ([npm](https://www.npmjs.com/package/@stellar/stellar-sdk)) |

### Free endpoints

Listed publicly by Stellar's [RPC providers page](https://developers.stellar.org/docs/data/apis/rpc/providers)
("can be used for development, testing, and production"); no rate limits are
published there. Endpoints marked ✓ answered `getVersionInfo` on 2026-10-12.

| Provider | Mainnet RPC | Testnet RPC |
|---|---|---|
| SDF | — (no public mainnet RPC listed) | `https://soroban-testnet.stellar.org` ✓ |
| sorobanrpc.com | `https://mainnet.sorobanrpc.com` ✓ | — |
| Gateway | `https://soroban-rpc.mainnet.stellar.gateway.fm` ✓ | `https://soroban-rpc.testnet.stellar.gateway.fm` |
| Nodies | `https://stellar-soroban-public.nodies.app` | `https://stellar-soroban-testnet-public.nodies.app` |
| OnFinality | `https://stellar.api.onfinality.io/public` | — |
| Lightsail | `https://rpc.lightsail.network/` (archive: `archive-rpc.lightsail.network`) | — |
| Ankr | `https://rpc.ankr.com/stellar_soroban` (archive) | — |

Horizon: SDF's `horizon.stellar.org` and `horizon-testnet.stellar.org` both
answered. Testnet friendbot: `https://friendbot.stellar.org/` (from RPC
`getNetwork`).

## 2. Supply

**Can a Soroban contract read a classic asset's total supply through the
Stellar Asset Contract? No.**

- The SAC implements the SEP-41 token interface (`allowance`, `approve`,
  `balance`, `transfer`, `transfer_from`, `burn`, `burn_from`, `decimals`,
  `name`, `symbol`) plus admin functions (`set_admin`, `admin`,
  `set_authorized`, `authorized`, `mint`, `clawback`, `trust`). None returns
  a total supply.
  ([Stellar Asset Contract docs](https://developers.stellar.org/docs/tokens/stellar-asset-contract))
- Classic balances live in trustlines (64-bit), contract balances in
  contract data entries (128-bit) — same source. **Inference, not verified
  from protocol spec:** the ledger has no entry that stores a per-asset
  total; Horizon computes totals during ingestion.

→ Supply has to be brought on-chain by a **supply witness** (see
[D0-RECOMMENDATION.md](D0-RECOMMENDATION.md)).

### Off-chain formula

From the Horizon asset record
([`/assets?asset_code=…&asset_issuer=…`](https://developers.stellar.org/docs/data/apis/horizon/api-reference/resources/assets/object)),
all amounts as 7-decimal strings:

```
supply = balances.authorized
       + balances.authorized_to_maintain_liabilities
       + balances.unauthorized
       + claimable_balances_amount
       + liquidity_pools_amount
       + contracts_amount
```

- Amounts locked in DEX offers are still inside trustline balances, so they
  are counted once.
- The issuer account holds no balance of its own asset, so nothing to
  subtract.
- **Ledger pinning:** `/assets` has no ledger parameter. Record Horizon's
  `history_latest_ledger` immediately before and after the read; if they
  differ, re-read. For exact historical supply, use Hubble or Galexie.

**Cross-check (USDC, `GA5Z…KZVN`):** formula = 376,112,799.58; stellar.expert
`supply` = 376,311,244.84 (difference 198,445.26, 0.053%).

**Reconciliation (D1, 2026-10-12, reads in parallel, Horizon ledgers
64905729–64905730):**

| Asset | Contracts holding | Formula | stellar.expert | Difference |
|---|---|---|---|---|
| USDC | 53,028 | 376,193,906.94 | 376,392,843.81 | +198,936.87 (+0.053%) |
| CETES | 36 | 58,377,175.11 | 58,416,541.38 | +39,366.27 (+0.067%) |
| USTRY | 39 | 11,513,904.72 | 11,470,087.13 | −43,817.59 (−0.381%) |
| AQUA | 267 | 99,927,569,420.97 | 99,928,735,460.13 | +1,166,039.16 (+0.001%) |
| EURC (Circle) | 93 | 4,515,319.38 | 4,515,595.81 | +276.43 (+0.006%) |
| USDGLO | 10 | 253,603.22 | 253,603.22 | +0.0009 |
| MGUSD | 4 (1.50 held) | 25,245,869.52 | 25,245,869.52 | 0 |

Findings:
- The gap is **not timing**: USDC's gap was the same to within 0.3% of
  itself across reads hours apart.
- Assets with negligible Soroban contract holdings match **exactly**
  (MGUSD, USDGLO). Gaps appear with contract activity.
- Gaps go **both ways** (USTRY is lower at stellar.expert), so "archived
  contract balances missing from Horizon" cannot be the only cause.
- Conclusion: the two sources count **contract-held balances** differently.
  Neither is shown to be right. Trustline, claimable-balance and pool
  amounts agree.

**Decision stands: supply is not published.** Resolving it needs a third,
independent count of SAC contract balances at a pinned ledger — Hubble
(BigQuery, contract_data at a ledger) or enumerating SAC balance entries via
RPC `getLedgerEntries`. Option for D2 (needs a decision): publish supply
only for assets whose `contracts_amount` is below 0.01% of the total, where
both sources agree.

## 3. SEP-1 fields for anchored assets

[SEP-1 v2.7.0](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0001.md)
(updated 2025-01-16). `[[CURRENCIES]]` fields relevant to Assay, verbatim
descriptions abbreviated. **All are optional** except `code` (and `issuer` for
classic assets / `contract` for Soroban tokens).

| Field | Type | Meaning |
|---|---|---|
| `code` | string ≤12 | Token code. Required. |
| `issuer` | `G…` | Issuing account. Required for classic assets. |
| `contract` | `C…` | SEP-41 token contract. Required for non-classic tokens. |
| `status` | string | `live`, `dead`, `test` or `private`. |
| `is_asset_anchored` | boolean | `true` if the token can be redeemed for an underlying asset. |
| `anchor_asset_type` | string | `fiat`, `crypto`, `nft`, `stock`, `bond`, `commodity`, `realestate`, `other`. |
| `anchor_asset` | string | What it is anchored to, e.g. USD. |
| `attestation_of_reserve` | url | "URL to attestation or other proof, evidence, or verification of reserves, such as third-party audits." |
| `redemption_instructions` | string | How to redeem. |
| `collateral_addresses` | list | For anchored crypto tokens: addresses holding the backing. |
| `collateral_address_messages` | list | Messages stating those funds back the asset. |
| `collateral_address_signatures` | list | Signatures of those messages by each address's key. |
| `regulated` / `approval_server` | bool / url | SEP-8 regulated assets. |

**Locating the file:** the issuer account's `home_domain` →
`https://{home_domain}/.well-known/stellar.toml`. Servers must send
`Access-Control-Allow-Origin: *`. A currency entry may instead be
`toml = "https://DOMAIN/.well-known/CURRENCY.toml"`, a separate file holding
that currency's fields.

**Correction to earlier project notes:** `attestation_of_reserve` is
optional, not required.

## 4. Census

See [CENSUS-v0.md](CENSUS-v0.md).

## 5. Oracles on Stellar

| Provider | On Stellar today | Proof of reserve on Stellar | Source |
|---|---|---|---|
| **RedStone** | Live since March 2026 (Soroban, Rust; audited by Zellic and Veridise per coverage). SEP-40 feeds; 9 assets live as of 2026-06-04: XLM, USDC, EURC, PYUSD, CETES, USTRY, TESOURO, SolvBTC, xSolvBTC (USST in progress). | **None found.** RedStone's 2026-08-26 Stellar report lists price feeds only; NAV figures there come from issuers. | [RedStone blog 2026-06-04](https://www.redstone.finance/blog/reliability-at-scale-redstone-and-the-data-standard-for-stellars-rwa-moment/), [RedStone report 2026-08-26](https://www.redstone.finance/blog/from-issuance-to-markets-pricing-rwas-on-stellar/) |
| **Chainlink** | Stellar joined Chainlink Scale and announced Data Feeds, Data Streams and CCIP ([Stellar blog](https://stellar.org/blog/foundation-news/stellar-to-join-chainlink-scale-and-adopt-data-feeds-data-streams-and-ccip-to-power-next-gen-defi-applications)). **Unverified:** whether Data Feeds are live on Stellar mainnet — a search summary cited a Chainlink changelog entry dated 2026-09-29, but we could not open it and Chainlink's addresses page did not show Stellar. Stellar's oracle page does not list Chainlink. | **None found.** Chainlink PoR exists on other chains; nothing for Stellar. | [Stellar oracle providers](https://developers.stellar.org/docs/data/oracles/oracle-providers) |
| **Reflector** | SEP-40 price feeds (Stellar DEX, external CEX/DEX, FX), mainnet and testnet; threshold webhook subscriptions. | No | Same |
| **Band** | Mainnet `CCQX…GG5M`, testnet `CBRV…JJMF`. | No | Same |
| **DIA** | Testnet only (USDC, BTC, DIA). | No | Same |
| **Pyth** | Pull model, subscription; mainnet verifier `CACZ…3HCH`. | No | Same |

**Public proof-of-reserve roadmap for Stellar: none found.**

## 6. "Reserve-verified" stablecoins

Cointelegraph reported "roughly $438 million in reserve-verified stablecoins
currently issued on Stellar" and RWA market cap of **$3.996B as of
2026-08-29**, citing "a Dune Analytics dashboard maintained by Stellar"
([via TradingView](https://www.tradingview.com/news/cointelegraph:69c1131e9094b:0-stellar-tokenized-rwa-market-more-than-quadruples-to-nearly-4b/)).
**Not found:** the dashboard URL, who maintains the "reserve-verified" list,
and the criteria.

Question for the Stellar Dev Discord:

> Stellar's Dune dashboard reports about $438M in "reserve-verified"
> stablecoins. Could someone point us to the dashboard, and tell us who
> maintains the reserve-verified list and what qualifies an asset (for
> example: an `attestation_of_reserve` link in stellar.toml, a named
> auditor, a report under a certain age)? We're building an open
> proof-of-reserves board for Stellar and want to use the same definition
> or explain clearly where ours differs.

## 7. Lending markets and how they stop new exposure

| Protocol | RWA collateral | How it pauses | Verified from |
|---|---|---|---|
| **Blend v2** | Yes; RWA pools held "just above $2M in TVL" (RedStone, 2026-08-26) out of Stellar DeFi TVL of $259.13M. | **Pool-wide, immediate:** `set_status(pool_status)` — admin-only (`admin.require_auth()`). Status 2 = admin on-ice (no new borrowing), 4 = admin frozen (no supply or borrowing); 3 = on-ice lifts admin states. **Per reserve, delayed:** `ReserveConfig.enabled = false` blocks `Supply`, `SupplyCollateral` and `Borrow` for that reserve (`ReserveDisabled`), but changes go through `queue_set_reserve`, which has a **one-week timelock** unless the pool is in setup (status 6). Withdrawals, repayments and liquidations stay open. | Source: [blend-contracts-v2 @ ba22b48](https://github.com/blend-capital/blend-contracts-v2) `pool/src/contract.rs` (`set_status`), `pool/src/pool/reserve.rs:148`, `pool/src/pool/config.rs:78-82` |
| Blend + **pool-status-admin** (Zenith) | — | A contract that becomes a Blend pool's admin and keeps only `set_status`; cannot change reserves, rates or admin; not upgradeable; built on OpenZeppelin Stellar Contracts v0.7.0 (`Ownable`). MIT. No audit mentioned. | [README](https://github.com/zenith-protocols/pool-status-admin) (code not reviewed) |
| **Templar** | Yes: CETES, USTRY, deJAAA, deJTRSY and others per announcements. Uses NEAR MPC / Chain Signatures for custody. | **Not found.** | Announcements only |
| **Liqvid** | Unclear. A testnet lending deployment appears in [XOXNO/rs-lending-xlm](https://github.com/XOXNO/rs-lending-xlm). | **Not found.** | Unverified |
| **Octarine** | Not a lending market: RWA liquidity auctions and planned liquidation auctions for lending protocols (Blend, Aquarius, DeFindex). Settlement contract on testnet. | n/a | [SCF project page](https://communityfund.stellar.org/project/octarine-vwl) |
| **DeFindex** | Vaults that route into strategies (e.g. Blend). | Vault-level, per strategy: `pause_strategy`, `unpause_strategy` and `rescue`, callable by the Emergency Manager **or** Manager role (`check_role` → `require_auth`). | Source: [paltalabs/defindex @ a0fcabf](https://github.com/paltalabs/defindex) `apps/contracts/vault/src/lib.rs:431,497,527`, `access.rs` |

**Implication:** for Blend, the only immediate lever is pool-wide
`set_status`. A guard that pauses one RWA reserve cannot act in under a week
through Blend's own controls. The realistic integration is: an RWA-only pool
(or one RWA per pool) whose status admin is a narrowly scoped keeper.

## 8. Building blocks

**attest.so.** Soroban contracts (authority resolver + core attestation
protocol) and a TypeScript SDK (`@attest.so/sdk`); schemas named like
`did:attest:identity` ([npm README](https://unpkg.com/attest.so@1.1.2/README.md)).
**Unverified:** contract interfaces and schema encoding. Assessment: an
attest.so attestation could carry a signed reserve report, but Assay's
registry must compute status on-chain (min coverage across fresh attestors,
staleness, supply mismatch), which needs the report fields in our own
storage. Option for T1: accept reports posted directly, and optionally mirror
or ingest attest.so attestations under an Assay schema.

**OpenZeppelin Stellar smart accounts** ([policies](https://docs.openzeppelin.com/stellar-contracts/accounts/policies),
[context rules](https://docs.openzeppelin.com/stellar-contracts/accounts/context-rules),
docs track the `main` branch, no version stated):
- Context rule types: `Default`, `CallContract(Address)`,
  `CreateContract(BytesN<32>)`. **Scoping is per contract, not per
  function**; nothing documented scopes a rule to one function name.
- `Policy` trait: `enforce`, `install`, `uninstall` (associated
  `AccountParams`); up to 5 policies per rule. Built-ins: simple threshold,
  weighted threshold, spending limit (a forum report says spending limit
  only meters transfers).
- **Implication for the scoped keeper (T2-D1):** a rule
  `CallContract(blend_pool)` would allow *any* call to the pool. Limiting
  the keeper to `set_status` needs a custom policy that checks the function
  name and arguments in `enforce`, or a wrapper contract like
  pool-status-admin that exposes only `set_status`. The wrapper is simpler
  and already exists.
