# Census v0: reserve links on Stellar mainnet

**Checked 2026-10-12** (13:42 and 13:49 UTC), Horizon ledgers 64905349 and
64905442, protocol 29. First public finding of Assay, by Garkuwa Labs.

This census records what asset issuers publish under
[SEP-1](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0001.md)
(v2.7.0) and what happened when we followed each link. It states facts, not
judgments. A missing `attestation_of_reserve` is not a fault: the field is
optional in SEP-1. A failed request is what we observed from one network
location at one time, not proof that a page is down.

Raw data: [`census/census-v0-2026-10-12.json`](census/census-v0-2026-10-12.json),
[`census/census-v0-supplement-2026-10-12.json`](census/census-v0-supplement-2026-10-12.json).
Script: [`scripts/src/census-v0.ts`](../scripts/src/census-v0.ts) (throwaway, v0).

## Summary

| | Top 50 by holders | RWA / stablecoin supplement |
|---|---|---|
| Assets checked | 50 | 12 |
| Asset listed in its issuer's `stellar.toml` | 30 | 8 |
| `is_asset_anchored = true` | 8 | 6 |
| `attestation_of_reserve` present | 3 | 6 |
| …link returned HTTP 2xx to an automated request | 0 | 5 |
| …link returned a structured (JSON) report | 0 | 0 |

**Main findings**

1. **No reserve link we followed returned a machine-readable report.** Every
   link that resolved returned an HTML page. Checking any reserve figure
   today means a person reading a web page.
2. **In the top 50 assets by holders, 3 publish a reserve link and none of
   the 3 returned a page to an automated request:** two returned HTTP 403
   from Cloudflare (to both our identified User-Agent and a browser
   User-Agent) and one entered a 308 redirect loop.
3. **Large issuers' `stellar.toml` files were not reachable at the
   `home_domain` recorded on the ledger for 3 of the 12 supplement assets**
   (Circle EURC: 404 at circle.com; Franklin Templeton BENJI and Paxos PYUSD:
   HTTP 403 from our location). Circle's legacy file at `centre.io` still
   lists USDC with `attestation_of_reserve`; the ledger's `home_domain` for
   the USDC issuer is `circle.com`.
4. **`Last-Modified` is not a report date.** Etherfuse's proof-of-reserves
   page returned a `Last-Modified` equal to the time of our request (the page
   is generated on request). Report age has to come from the report itself.
5. **Lookalike assets exist.** Searching by code returns assets named BENJI
   from three issuers with three different home domains. The indexer must
   key every asset by code **and issuer** and show the issuer's domain.

## Method

**Top 50 by holders.** Classic assets ranked by trustline count from the
public stellar.expert API
(`/explorer/public/asset?sort=trustlines&order=desc`). Native XLM and one
Soroban contract token (`CCA2…F5HR`, no classic issuer) were skipped.
Ranking by trustlines is a holders proxy; it favours widely airdropped
tokens and misses most RWAs, which have few holders.

**Supplement.** Twelve well-known RWA and stablecoin assets, found by code
search on stellar.expert. Where a code search returned several issuers, each
is listed. Inclusion here does not mean an asset is genuine; the issuer
domain column is what to read. List:
[`census/supplement-ids-2026-10-12.txt`](census/supplement-ids-2026-10-12.txt).

**For each asset:**
1. Issuer `home_domain` read from Horizon (`/accounts/{issuer}`), not from
   a third party.
2. `https://{home_domain}/.well-known/stellar.toml` fetched (redirects
   followed) and parsed as TOML. The `[[CURRENCIES]]` entry matching both
   `code` and `issuer` selected; `toml = "…"` links to separate currency
   files followed.
3. If `attestation_of_reserve` is present: HEAD request, then GET if HEAD
   failed. Recorded HTTP status, final URL, `Content-Type`, `Last-Modified`.
4. Requests sequential, at least 1 second apart per host, 15-second timeout,
   User-Agent `AssayCensus/0.1 (+https://github.com/Garkuwa-Studio/assay)`.
   One attempt per URL; failures were re-tried by hand with a browser
   User-Agent (noted above).

**Limits.** One network location, one point in time, one attempt per URL
(plus manual retries). Supply figures in the JSON use the formula in
[STACK-STATUS.md](STACK-STATUS.md#2-supply) and are for context only.

## Tables

### Top 50 by holders

checked_at 2026-10-12T13:42:10.364Z · Horizon ledger 64905349 · protocol 29 · n=50

| Count | Value |
|---|---|
| Assets checked | 50 |
| Issuer has a home_domain | 46 |
| stellar.toml fetched and parsed | 30 |
| Asset listed in its issuer's stellar.toml | 30 |
| is_asset_anchored = true | 8 |
| attestation_of_reserve present | 3 |
| …link returned HTTP 2xx | 0 |
| …link sent Last-Modified | 0 |
| collateral_addresses listed | 2 |

| # | Asset | Issuer | home_domain | stellar.toml | Listed | Anchored | Type | attestation_of_reserve | Link result (status · type · Last-Modified) |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `USDC` | `GA5Z…KZVN` | circle.com | 404 | no | — | — | — | — |
| 2 | `AQUA` | `GBNZ…AQUA` | aqua.network | 200 | yes | no | — | — | — |
| 3 | `LMNS` | `GBZP…LMNS` | lmns.site | 404 | no | — | — | — | — |
| 4 | `GTN` | `GARF…BA46` | glitzkoin.com | 404 | no | — | — | — | — |
| 5 | `SHX` | `GDST…J6JH` | stronghold.co | 200 | yes | — | — | — | — |
| 6 | `ARST` | `GCSA…I3DG` | pubnet-sep.latamex.com | 200 | yes | — | fiat | [link](https://anchors.relay.vision/audits/ARST-Audit-No-1.pdf) | timeout · — · — |
| 7 | `DMMK` | `GDMM…DZXD` | nugpay.app | 200 | yes | yes | fiat | — | — |
| 8 | `USD` | `GAVA…ELAX` | — | no home_domain | no | — | — | — | — |
| 9 | `SCOP` | `GC6O…H3VQ` | scopuly.com | 200 | yes | — | — | — | — |
| 10 | `ZARZ` | `GARO…BB3U` | zeam.money | 200 | yes | yes | fiat | [link](https://zeam.money/transparency) | 403 · text/html · — |
| 11 | `XRP` | `GBXR…DTD5` | fchain.io | 200 | yes | yes | crypto | — | — |
| 12 | `Yinbi` | `GDTF…EY2E` | yin.bi | 404 | no | — | — | — | — |
| 13 | `yXLM` | `GARD…5T55` | ultracapital.xyz | 200 | yes | yes | crypto | — | — |
| 14 | `MOBI` | `GA6H…JPIH` | mobius.network | 200 | yes | — | — | — | — |
| 15 | `VELO` | `GDM4…2M5M` | — | no home_domain | no | — | — | — | — |
| 16 | `BRAVE` | `GDRE…NTJL` | brave-token.age-shield.com | 200 | yes | no | — | — | — |
| 17 | `SSLX` | `GBHF…37UR` | sslx.sl8.online | 200 | yes | yes | crypto | — | — |
| 18 | `ARS` | `GCYE…DARS` | api.anclap.com | 200 | yes | yes | fiat | — | — |
| 19 | `CENTUS` | `GAKM…MNDZ` | — | no home_domain | no | — | — | — | — |
| 20 | `AFR` | `GBX6…N54W` | afreum.com | 200 | yes | — | — | — | — |
| 21 | `NWC` | `GAAP…7AY2` | numerico.ai | 200 | yes | — | — | — | — |
| 22 | `SLT` | `GCKA…Q4GP` | smartlands.io | timeout | no | — | — | — | — |
| 23 | `XXA` | `GC4H…ADIM` | ixinium.io | 200 | yes | no | — | — | — |
| 24 | `BONUS` | `GCP7…HDUH` | — | no home_domain | no | — | — | — | — |
| 25 | `GLASS` | `GACE…4ZEA` | noob.stellarskull.io | 200 | yes | — | — | — | — |
| 26 | `KALE` | `GBDV…KALE` | kalepail.com | 200 | yes | — | — | — | — |
| 27 | `GDOGE` | `GBJQ…W3C2` | giga-doge.com | DNS not found | no | — | — | — | — |
| 28 | `NODL` | `GB2Y…NODL` | nodle.io | 200, not TOML (HTML) | no | — | — | — | — |
| 29 | `NRV` | `GANR…KKX2` | nrvcoin.in | DNS not found | no | — | — | — | — |
| 30 | `XLMG` | `GCVN…I3YH` | stellargold.net | 200 | yes | no | — | — | — |
| 31 | `YNB` | `GDGW…DOY3` | ynbmeta.xyz | 200, not TOML (HTML) | no | — | — | — | — |
| 32 | `LSP` | `GAB7…24WK` | lumenswap.io | 200 | yes | — | — | — | — |
| 33 | `USDZ` | `GAKT…6XPR` | zeam.money | 200 | yes | yes | crypto | [link](https://zeam.money/transparency) | 403 · text/html · — |
| 34 | `AnonInu` | `GBNW…6CR6` | anoninu.stellarmint.io | 200 | yes | no | — | — | — |
| 35 | `SMRT` | `GC4I…FJKL` | smrt.xlmplay.top | DNS not found | no | — | — | — | — |
| 36 | `HOTDOG` | `GAEN…I5XR` | moon.stellarskull.io | 200 | yes | — | — | — | — |
| 37 | `ANONINU` | `GAB3…RUHX` | anon-inu.com | DNS not found | no | — | — | — | — |
| 38 | `yUSDC` | `GDGT…TTFF` | ultracapital.xyz | 200 | yes | yes | crypto | — | — |
| 39 | `SQL0001` | `GDQM…IY35` | quest.stellar.org | 200 | yes | — | — | — | — |
| 40 | `LIES` | `GCVS…V4Y6` | liecoin.org | TLS cert expired | no | — | — | — | — |
| 41 | `SHAGGY` | `GBC4…C5U3` | shaggy-token.com | DNS not found | no | — | — | — | — |
| 42 | `BEER` | `GBL2…SHNN` | beer.stellarmint.io | 200 | yes | no | — | — | — |
| 43 | `MOONTWINKIES` | `GCDB…VXSQ` | moontwinkies.stellarskull.io | 200 | yes | — | — | — | — |
| 44 | `StellarNFT` | `GALZ…IOZJ` | stellarnft.io | DNS not found | no | — | — | — | — |
| 45 | `DIAMONDHANDS` | `GBIG…XVRP` | diamondhands.stellarskull.io | 200 | yes | — | — | — | — |
| 46 | `NORM` | `GBIB…JPIS` | norm.xlmplay.top | DNS not found | no | — | — | — | — |
| 47 | `SQL0101` | `GDCB…L7AE` | quest.stellar.org | 200 | yes | — | — | — | — |
| 48 | `TERN` | `GDGQ…GP6C` | ternio.io | 404 | no | — | — | — | — |
| 49 | `FRED` | `GCA7…ZLV6` | fredenergy.org | 200 | yes | no | — | — | — |
| 50 | `SQL0102` | `GB76…KBOZ` | quest.stellar.org | 200 | yes | — | — | — | — |

### RWA and stablecoin supplement

checked_at 2026-10-12T13:49:54.709Z · Horizon ledger 64905442 · protocol 29 · n=12

| Count | Value |
|---|---|
| Assets checked | 12 |
| Issuer has a home_domain | 12 |
| stellar.toml fetched and parsed | 8 |
| Asset listed in its issuer's stellar.toml | 8 |
| is_asset_anchored = true | 6 |
| attestation_of_reserve present | 6 |
| …link returned HTTP 2xx | 5 |
| …link sent Last-Modified | 3 |
| collateral_addresses listed | 0 |

| Asset | Issuer | home_domain | stellar.toml | Listed | Anchored | Type | attestation_of_reserve | Link result (status · type · Last-Modified) |
|---|---|---|---|---|---|---|---|---|
| `EURC` | `GDHU…NPP2` | circle.com | 404 | no | — | — | — | — |
| `EURC` | `GAQR…Z7NM` | mykobo.co | 200 | yes | yes | fiat | [link](https://www.mykobo.co/attestations/) | 404 · text/html · — |
| `PYUSD` | `GDQE…U2V5` | token-metadata.paxos.com | 403 | no | — | — | — | — |
| `CETES` | `GCRY…MYWC` | etherfuse.com | 200 | yes | yes | bond | [link](https://app.etherfuse.com/legal/proof-of-reserves) | 200 · text/html · Mon, 12 Oct 2026 13:28:20 GMT |
| `USTRY` | `GCRY…MYWC` | etherfuse.com | 200 | yes | yes | bond | [link](https://app.etherfuse.com/legal/proof-of-reserves) | 200 · text/html · Mon, 12 Oct 2026 13:28:20 GMT |
| `TESOURO` | `GCRY…MYWC` | etherfuse.com | 200 | yes | yes | bond | [link](https://app.etherfuse.com/legal/proof-of-reserves) | 200 · text/html · Mon, 12 Oct 2026 13:28:20 GMT |
| `BENJI` | `GBHN…ZIW5` | www.franklintempleton.com | 403 | no | — | — | — | — |
| `BENJI` | `GAXS…W7JW` | franklintempleton.reallumens.com | DNS not found | no | — | — | — | — |
| `BENJI` | `GCJG…3OKF` | etfs.stellarlistings.org | 200 | yes | — | — | — | — |
| `USDY` | `GAJM…DAZ6` | ondo.finance | 200 | yes | — | — | [link](https://ondo.finance/usdy) | 200 · text/html · — |
| `MGUSD` | `GAIU…JSUZ` | mgusd.moneygram.com | 200 | yes | yes | fiat | — | — |
| `USDGLO` | `GBBS…S6XV` | app.glodollar.org | 200 | yes | yes | fiat | [link](https://brale.xyz/stablecoins/USDGLO) | 200 · text/html · — |

Link notes: ARST's `attestation_of_reserve` (`anchors.relay.vision/…/ARST-Audit-No-1.pdf`)
returned HTTP 308 repeatedly until the client stopped following redirects.
ZARZ and USDZ share `zeam.money/transparency`, which returned HTTP 403 from
Cloudflare to both User-Agents. EURC (mykobo.co) linked to a page that
returned HTTP 404.

## What this means for Assay

- The indexer (D1) needs statuses that separate "we could not fetch it"
  from "it is not there". Proposed: keep `LINK_BROKEN` for 404/410/DNS
  failures and record HTTP status and server for 403s and timeouts, retried
  over several days before any status changes.
- Report age must come from a date inside the report or a manual
  transcription, never from HTTP headers.
- The census in D1 should start from all assets whose issuer has a
  `home_domain`, not from a holders ranking.
