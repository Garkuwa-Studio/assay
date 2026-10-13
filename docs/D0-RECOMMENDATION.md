# D0 recommendation: supply witness and first pilots

2026-10-12. Based on [STACK-STATUS.md](STACK-STATUS.md) and
[CENSUS-v0.md](CENSUS-v0.md).

## 1. Supply witness

**Problem.** Coverage needs reserves ÷ supply. A Soroban contract cannot read
a classic asset's total supply (the SAC has no `total_supply`, and SEP-41
does not define one). Issuer reports state their own liabilities, which is
the number we most need checked independently.

**Recommendation: registered witnesses post supply at a ledger; the registry
uses the larger figure.**

- `set_supply_witness(asset, witness)` by the asset admin; v0 allows one
  witness, T1 allows several.
- `post_supply(asset, amount, ledger)` with `witness.require_auth()`.
  Rejected if `ledger` is in the future or not newer than the last post.
- The witness is an assay-indexer job computing the Horizon formula
  (STACK-STATUS §2), reading `history_latest_ledger` before and after and
  retrying until they match. Posts hourly, and immediately on a change of
  more than 0.5%.
- Status math stays conservative:
  `liabilities = max(report.liabilities, latest fresh witnessed supply)`.
  `supply_mismatch` is set when they differ by more than a policy threshold.
  A witnessed supply older than the policy's max age counts as missing,
  never as zero.
- In T1, allow several witnesses (Assay, the issuer, an independent party)
  and use the **maximum** fresh value, so one wrong or compromised witness
  can only make status more cautious, never less.

**Rejected alternatives.**
- *Read supply on-chain:* not possible (above).
- *Trust the report's liabilities alone:* self-reported; the witness is the
  independent check.
- *Wrap issuance in a contract that counts mints and burns:* classic issuers
  can still mint by paying from the issuer account, so the count could miss
  supply. Possible only for new contract-native tokens.

**Open item for D1:** our formula and stellar.expert differ by 0.053% for
USDC. Resolve that (pinned ledger, archived contract balances) before the
witness posts anything.

## 2. First three pilot targets

Chosen from census facts: anchored, a reserve link that resolved, and a
direct line to lending use. These are invitations, not assessments.

1. **Etherfuse (CETES, USTRY, TESOURO). Issuer pilot.** Declares
   `is_asset_anchored = true`, type `bond`. Its `attestation_of_reserve`
   page returned HTTP 200. Its assets have RedStone SEP-40 price feeds and
   are named as collateral on Templar. A monthly signed JSON report beside
   the existing page would make these the first assets a contract can check.
2. **A Blend RWA pool admin. Protocol pilot.** Blend RWA pools hold about
   $2M, the use case Assay exists for. Blend's only immediate lever is
   pool-wide `set_status`; per-reserve changes take a week. Proposed
   integration: a keeper modelled on Zenith's pool-status-admin that sets
   admin on-ice (status 2) when the guard returns a reason other than OK.
   Who administers each RWA pool is to be identified in D1 from on-chain
   pool config.
3. **Brale (issuer platform behind USDGLO). Issuer/attestor pilot.** USDGLO
   is anchored to USD, and its reserve link (brale.xyz) returned HTTP 200.
   Brale issues for several stablecoin brands, so one integration of a
   signed report could cover several assets.

**Next outreach after these:** Zeam (ZARZ, USDZ: links published but
returned 403 to automated requests, so a structured report would help) and
MoneyGram (MGUSD: anchored, no `attestation_of_reserve` field yet).

## 3. Decisions (2026-10-12)

1. **Approved:** the supply-witness design above, for D3 (registry v0).
2. **Approved:** pilot targets Etherfuse (issuer), a Blend RWA pool admin
   (protocol), Brale/USDGLO (issuer and attestor).
3. **Approved:** the 0.053% supply gap blocks publishing supply. The board
   shows supply as unknown until D1 reconciles the formula with an
   independent source at a pinned ledger.
4. **Open:** post the "reserve-verified" question (STACK-STATUS §6) in the
   Stellar Dev Discord, from the founder's account.
