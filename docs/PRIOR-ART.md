# Prior art

What related projects do, what is live on Stellar, and how Assay fits
alongside them. Checked against each project's own pages on **2026-10-14**;
anything we could not confirm says so. Corrections are welcome: open an
issue or email the maintainers.

Assay's position: **complement oracles, don't compete.** Oracles report
prices and values; Assay reports whether a token's backing is documented,
how recent the documentation is, who signed it, and whether it covers the
supply. A protocol can use both.

## Summary

| Project | What it does | On Stellar today | Proof of reserve on Stellar |
|---|---|---|---|
| RedStone | Oracle: price and NAV feeds; PoR feeds on other chains | Live (mainnet, SEP-40 feeds) | None found |
| Chainlink | Oracle network: Data Feeds, Data Streams, CCIP, PoR feeds on other chains | Adoption announced; live Data Feeds on Stellar not confirmed | None found |
| Reflector | Stellar-native price oracle with node consensus | Live (mainnet and testnet) | No |
| attest.so | Attestation framework: schemas, authorities, resolvers | Soroban contracts and SDK; maturity unclear | No (general-purpose) |
| PaltaLabs | Builders of Soroswap and DeFindex | Live | No |
| Octarine | RFQ liquidity for tokenized RWAs | SCF-funded, in build | No |

## RedStone

A modular oracle that pushes price feeds, including feeds for tokenized
real-world assets. Stellar's developer docs list it as an oracle provider.
RedStone's August 2026 report on Stellar describes 47 price feeds on
Stellar, 18 of them SEP-40 compliant, covering assets such as CETES, USTRY,
TESOURO, USDY, BENJI and PYUSD. RedStone offers proof-of-reserve feeds on
other networks; we found no PoR feed on Stellar.

**How Assay fits:** RedStone tells a protocol what an RWA is worth; Assay
tells it whether the backing behind that value is documented, recent and
signed. A NAV feed can stay flat while reports stop. Assay's registry is
provider-agnostic, so a RedStone PoR feed on Stellar could become one of
several attestors for an asset.

Sources: [Stellar oracle providers](https://developers.stellar.org/docs/data/oracles/oracle-providers) ·
[RedStone on Stellar, 2026-06-04](https://www.redstone.finance/blog/reliability-at-scale-redstone-and-the-data-standard-for-stellars-rwa-moment/) ·
[RedStone report, 2026-08-26](https://www.redstone.finance/blog/from-issuance-to-markets-pricing-rwas-on-stellar/) ·
[RedStone proof of reserve](https://www.redstone.finance/proof-of-reserve)

## Chainlink

Chainlink runs Proof of Reserve feeds on other networks, sourcing reserve
data from auditors, custodians or issuers' APIs, and read like any other
Chainlink data feed. The Stellar Development Foundation announced that
Stellar is joining Chainlink Scale and adopting Data Feeds, Data Streams and
CCIP. **Not confirmed:** we did not find Stellar on Chainlink's feed address
pages, and Stellar's oracle provider page does not list Chainlink. We found
no Chainlink PoR feed for a Stellar asset.

**How Assay fits:** if Chainlink PoR comes to Stellar, its feeds can post
into the same registry as other attestors, and Assay's status shows where
sources agree or differ. Assay also covers the many smaller issuers for whom
a dedicated feed is unlikely: anyone with a Stellar key can sign a report.

Sources: [Chainlink Proof of Reserve docs](https://docs.chain.link/data-feeds/proof-of-reserve) ·
[Stellar joins Chainlink Scale](https://stellar.org/blog/foundation-news/stellar-to-join-chainlink-scale-and-adopt-data-feeds-data-streams-and-ccip-to-power-next-gen-defi-applications) ·
[Stellar oracle providers](https://developers.stellar.org/docs/data/oracles/oracle-providers)

## Reflector

A Stellar-native oracle. Independent nodes run by ecosystem organizations
compute prices from the Stellar DEX, Soroban DEXs and external sources, and
a price is written only when a majority of nodes produce the same
transaction (the contract's admin is a multisig of the nodes). It serves
SEP-40 price feeds, history and TWAP, and offers subscriptions that fire
when a price moves past a threshold.

**How Assay fits:** Reflector's consensus model and threshold triggers are
good precedent for Assay's alerts. Reflector reports market prices, not
reserves, so the two answer different questions.

Sources: [Stellar oracle providers](https://developers.stellar.org/docs/data/oracles/oracle-providers) ·
[reflector.network](https://reflector.network/)

## attest.so (Attest Protocol)

An open-source attestation framework: authorities make verifiable claims
about subjects under schemas, and resolver contracts let other protocols
consume them. It has a Soroban implementation and a TypeScript SDK; listed
use cases include KYC status, proof of humanity and agent authorization.
Sources disagree on how far the Stellar deployment has gone (the package
README lists Stellar as active; an SCF submission describes it as in
development).

**How Assay fits:** attest.so is general-purpose; Assay is specific to
reserves. Assay's registry computes status on-chain (lowest coverage across
fresh attestors, staleness, supply mismatch), which needs the report's
figures in a fixed format. A signed reserve report could still be carried as
an attest.so attestation; the SEP draft lists this as an open question.

Sources: [attest.so README (npm 1.1.2)](https://unpkg.com/attest.so@1.1.2/README.md) ·
[daccred/attest.so](https://github.com/daccred/attest.so)

## PaltaLabs

The team behind Soroswap (a Stellar DEX and liquidity aggregator) and
DeFindex (yield vaults that route into strategies such as Blend). They also
maintain a public Dune dashboard of Stellar DeFi activity. DeFindex vaults
have emergency controls: `pause_strategy`, `unpause_strategy` and `rescue`,
callable by the vault's Emergency Manager or Manager role
([`vault/src/lib.rs`](https://github.com/defindex-io/stellar-contracts) at
commit `256f984`, lines 431–536; development moved there from the archived
`paltalabs/defindex`).

**How Assay fits:** DeFindex is a natural consumer. A vault manager or a
narrowly scoped keeper could pause a strategy that holds an RWA when Assay
Guard reports it unhealthy, using the vault's existing role rather than any
new power.

Sources: [Soroswap](https://soroswap.finance/) ·
[defindex-io/stellar-contracts](https://github.com/defindex-io/stellar-contracts) ·
[Stellar DeFi Dune dashboard](https://dune.com/paltalabs/stellar-defi)

## Octarine

A liquidity layer for tokenized RWAs on Stellar. Funds, T-bills and private
credit are hard to trade on ordinary DEXs because of redemption periods;
Octarine runs RFQ auctions in which selected liquidity providers bid to take
the asset, wait out redemption and earn a premium, while the seller gets
liquidity at once. Awarded $129.7K in SCF #44 and in build. Lending
liquidations and margin calls are among its stated use cases.

**How Assay fits:** both serve protocols that want to accept RWAs safely.
Octarine helps exit a position; Assay helps decide whether to keep taking on
more. A lending market could use Assay Guard to stop new borrowing against
an asset and Octarine to unwind positions in it.

Source: [SCF project page](https://communityfund.stellar.org/project/octarine-vwl)

## Related building blocks

- **Blend v2** pools have one immediate lever, pool-wide `set_status`;
  per-reserve changes go through a one-week timelock
  ([blend-contracts-v2](https://github.com/blend-capital/blend-contracts-v2)).
- **Zenith's pool-status-admin** is a contract that can hold only Blend's
  `set_status`, a model for a scoped keeper
  ([pool-status-admin](https://github.com/zenith-protocols/pool-status-admin)).

Details and line references: [STACK-STATUS.md §7](STACK-STATUS.md).
