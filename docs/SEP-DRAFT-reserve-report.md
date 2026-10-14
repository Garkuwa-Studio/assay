## Preamble

```
SEP: To Be Assigned
Title: Signed Reserve Reports
Author: Garkuwa Labs (Assay)
Status: Draft
Created: 2026-10-14
Updated: 2026-10-14
Version: 0.0.1
Discussion: TBD (to be opened in stellar/stellar-protocol GitHub Discussions)
```

> Pre-SEP draft, ready to post for discussion. Not yet submitted to
> stellar/stellar-protocol. Feedback welcome before it becomes a pull request.

## Simple Summary

A JSON format for reserve reports that machines can read and check, signed
with a Stellar key. An issuer's `stellar.toml` can point
`attestation_of_reserve` at one of these documents instead of (or as well
as) a PDF.

## Abstract

[SEP-1](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0001.md)
v2.7.0 defines an optional `attestation_of_reserve` field on each currency:
"URL to attestation or other proof, evidence, or verification of reserves,
such as third-party audits." It does not define what the URL returns. This
SEP defines a document, `reserve-report-v0`, with the reserves and
liabilities as stated, the date they are as of, who states them, links to
the evidence, and an Ed25519 signature made with
[SEP-53](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0053.md)
over the document's [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785)
canonical form. It also defines two optional `stellar.toml` fields so
readers can check that the signing key belongs to an attestor the issuer
accepts.

## Motivation

- **Wallets, explorers and contracts cannot read today's reports.** The
  field points to a PDF or web page with no standard date, figures or
  author. In Assay's census of Stellar mainnet (snapshot 2026-10-12), of
  the 270 assets that claim backing and have either 50+ funded holders or a
  reserve link on their own domain, 41 publish a reserve link, 33 of those
  links return a document, and **none** returns a machine-readable report
  ([method](CENSUS-v0.md),
  [snapshot](https://github.com/Garkuwa-Studio/assay-indexer)).
- **Age matters as much as the figures.** A report from last year says
  little about today. A standard `as_of` field lets anyone show how old the
  latest report is.
- **Who signed matters.** An issuer can already publish a statement about
  its own reserves. A signature by a named attestor's key, which the issuer
  lists in its `stellar.toml`, lets readers tell an auditor's statement from
  the issuer's own.
- **Lending markets need a signal.** A protocol that accepts a tokenized
  asset as collateral can stop new exposure when reports stop or show a
  shortfall, if it can read them.

This SEP standardizes the format only. It does not rate issuers or
attestors, and a valid signature means only that the key holder made the
statement, not that the statement is true.

## Specification

### Document

Served over HTTPS at the URL in `attestation_of_reserve`, with
`Content-Type: application/json`. Servers SHOULD send
`Access-Control-Allow-Origin: *`, as SEP-1 requires for `stellar.toml`.

```json
{
  "sep": "reserve-report-v0",
  "asset": "CODE:GISSUER…",
  "as_of": "2026-09-30T23:59:59Z",
  "ledger": 5181119,
  "reserves": [
    { "type": "cash", "amount": "600000.00", "unit": "USD" },
    { "type": "government_bond", "amount": "410000.00", "unit": "USD" }
  ],
  "liabilities": { "supply": "1000000.00", "unit": "CODE" },
  "attestor": {
    "name": "Example Audit LLP",
    "role": "auditor",
    "domain": "example-audit.com",
    "stellar_key": "GATTESTOR…"
  },
  "evidence": [{ "uri": "https://example-audit.com/reports/2026-09.pdf", "sha256": "…" }],
  "signature": "base64…"
}
```

| Field | Required | Description |
|---|---|---|
| `sep` | Yes | The string `reserve-report-v0`. |
| `asset` | Yes | `CODE:ISSUER`, the asset the report is about. It MUST match the `[[CURRENCIES]]` entry that links to the report. |
| `as_of` | Yes | RFC 3339 date-time, with offset, that the figures are as of. Readers measure the report's age from it. |
| `ledger` | No | A Stellar ledger sequence close to `as_of`, if the attestor measured on-chain supply. |
| `reserves` | Yes | One or more `{type, amount, unit}`. `amount` is a decimal string (`^\d+(\.\d+)?$`); never a JSON number. `unit` is an ISO 4217 code (`USD`) or an asset code. `type` is free text (suggested: `cash`, `government_bond`, `money_market_fund`, `commodity`, `loan`, `crypto`). |
| `liabilities` | Yes | `{supply, unit}`: the amount owed to holders as the attestor states it, as a decimal string, normally in the asset's own unit. |
| `attestor` | No | `name`, `role` (e.g. `auditor`, `custodian`, `issuer`), `domain` (the attestor's home domain) and `stellar_key` (the signing key, a `G…` address). Required when the report is signed. |
| `evidence` | No | Supporting documents: `{uri, sha256}`, where `sha256` is the lowercase hex SHA-256 of the document's bytes. |
| `signature` | No | Base64 Ed25519 signature, as defined below. |

Readers MUST ignore fields they do not recognize. Writers SHOULD NOT add
fields whose meaning would change how the figures are read.

### Canonical form and signature

1. Remove the `signature` field from the document.
2. Serialize the rest with the JSON Canonicalization Scheme
   ([RFC 8785](https://www.rfc-editor.org/rfc/rfc8785)) and encode it as
   UTF-8. This is the **payload**.
3. Sign the payload with SEP-53: Ed25519 over
   `SHA-256("Stellar Signed Message:\n" || payload)` using the key in
   `attestor.stellar_key`.
4. Put the base64 encoding of the 64-byte signature in `signature`.

Because the key sits inside the payload, it cannot be swapped without
breaking the signature. Because of RFC 8785, the signature does not depend
on key order, whitespace or line endings in the served file. Using SEP-53
means any wallet or tool that signs SEP-53 messages can sign a report (for
example `stellar message sign`, stellar-cli 28.1.0), and a report
signature can never be mistaken for a transaction signature.

To verify, a reader repeats steps 1–2 and checks the signature against
`attestor.stellar_key` with SEP-53.

### Keys in stellar.toml

Two optional fields let a reader check that the key belongs to an attestor
the issuer accepts:

**Issuer side**, in the asset's `[[CURRENCIES]]` entry:

| Field | Type | Description |
|---|---|---|
| `attestation_of_reserve_signers` | list of `G…` strings | Keys whose signed reserve reports the issuer accepts for this asset. |

**Attestor side**, at the top level of the attestor's own `stellar.toml`
(at `attestor.domain`):

| Field | Type | Description |
|---|---|---|
| `RESERVE_REPORT_KEYS` | list of `G…` strings | Keys this organization uses to sign reserve reports. |

```toml
# issuer's stellar.toml
[[CURRENCIES]]
code = "CODE"
issuer = "GISSUER…"
is_asset_anchored = true
attestation_of_reserve = "https://issuer.example/reserves/latest.json"
attestation_of_reserve_signers = ["GATTESTOR…"]

# attestor's stellar.toml, at https://example-audit.com/.well-known/stellar.toml
RESERVE_REPORT_KEYS = ["GATTESTOR…"]
```

A report is **attributed** when its signature verifies, its key is in the
issuer's `attestation_of_reserve_signers`, and (if `attestor.domain` is
given) the key is also in that domain's `RESERVE_REPORT_KEYS`. The two lists
together stop an issuer from naming an attestor that never signed, and stop
an attestor's key from being used for an issuer that never accepted it.

### What readers should show

- The figures as stated, with the attestor and the report's age.
- Whether the signature verifies and whether the report is attributed.
- Coverage (reserves ÷ liabilities) only when the units can be compared
  without an assumption: the same unit, or liabilities in the asset and
  reserves in the fiat currency the issuer's `stellar.toml` names in
  `anchor_asset`. Otherwise no coverage, with the reason.
- A report whose `asset` does not match the currency entry, whose `as_of`
  is in the future, or whose signature fails MUST NOT be shown as valid.

## Limitations

- A signature shows who made a statement, not that it is true. An attestor
  can sign wrong figures. Readers can compare attestors and compare stated
  liabilities with on-chain supply, but cannot detect a consistent lie.
- Reserves in a different currency from the asset need an exchange rate,
  which this format does not carry.
- One document covers one asset.

## Backwards Compatibility

Nothing changes for existing issuers. `attestation_of_reserve` stays
optional, and a URL that returns a PDF or web page stays valid; readers
treat it as an unstructured report a person has to read. The two new
`stellar.toml` fields are optional and ignored by readers that do not know
them. Readers detect this format by `Content-Type: application/json` and
`"sep": "reserve-report-v0"`.

## Security Concerns

- **Replay of old reports.** A valid old report stays valid. Readers MUST
  judge freshness from `as_of`, never from when the URL was fetched.
- **Key compromise.** An issuer removes the key from
  `attestation_of_reserve_signers`; reports signed with it stop being
  attributed. Key rotation is an open question below.
- **Domain compromise.** Whoever controls the issuer's domain controls its
  `stellar.toml`, as with every SEP-1 field. The attestor-side list limits
  what an attacker can claim about an attestor's key.

## Test Vectors

From [`packages/sdk/test-vectors/reserve-report-v0.json`](../packages/sdk/test-vectors/reserve-report-v0.json).
The key comes from a published seed (`a1` repeated 32 times):
**test only, never use it for real reports.**

| | |
|---|---|
| Public key | `GC6HZPFVMNRXL6Q5QJBU2RTHETMSG57VHOMANFO5JHJG2DHBEIC2KTOS` |
| SHA-256 of payload | `b60476e1636c0a2cb93cb66c6ee4ee47216cc4a8707ea5cef45291023198f8ed` |
| SEP-53 message hash | `a5444f23c260f57a2202488ca8a7bb31f34f016a8444ba603411478334002cdc` |
| Signature | `EkxLsJgNs1QiUeq7N3B81BxpfFCSN7/fFQOGun25h9RTHzGxPiQ6M9lo/RXn8zHyMRA5aLeG1OQV70lAhFoIDw==` |

Payload:

```
{"as_of":"2026-09-30T23:59:59Z","asset":"DEMO:GBQK5IGFFJUEMJJ6G4ZRU2N4ZBUUEV2OFYUQSEXLOI5NMV6FHJ6F7ZPC","attestor":{"name":"Example Attestor (test vector)","role":"auditor","stellar_key":"GC6HZPFVMNRXL6Q5QJBU2RTHETMSG57VHOMANFO5JHJG2DHBEIC2KTOS"},"evidence":[{"sha256":"0000000000000000000000000000000000000000000000000000000000000000","uri":"https://example.org/reserves-2026-09.pdf"}],"ledger":5181119,"liabilities":{"supply":"1000000.00","unit":"DEMO"},"reserves":[{"amount":"600000.00","type":"cash","unit":"USD"},{"amount":"410000.00","type":"government_bond","unit":"USD"}],"sep":"reserve-report-v0"}
```

The signature verifies with an independent implementation:
`stellar message verify --public-key GC6H… --signature EkxL…` (stellar-cli
28.1.0) on the payload above returns "Signature valid", and fails if any
character of the payload changes.

## Reference Implementation

- `@garkuwa/assay-sdk` (TypeScript, Apache-2.0): `canonicalize`,
  `signingPayload`, `signReport`, `verifyReport`
  ([source](../packages/sdk/src/report.ts)).
- `scripts/src/sign-report.ts`: signs a report with a key held in the
  stellar CLI keystore via `stellar message sign`.
- [assay-indexer](https://github.com/Garkuwa-Studio/assay-indexer) reads
  this format and shows the figures on the Assay board.

## Open Questions

1. **Field names.** `attestation_of_reserve_signers` and
   `RESERVE_REPORT_KEYS` are proposals. Would SEP-1 maintainers prefer them
   in SEP-1 itself or here?
2. **Several assets per document.** Issuers with many assets (or one
   attestor covering several) may prefer one document with a list. v0 keeps
   one asset per document for simple verification.
3. **Key rotation and history.** Should old keys stay listed (with dates) so
   old reports stay attributed?
4. **Report history.** Should the document link to the previous report so
   readers can show a series?
5. **On-chain anchoring.** Assay's registry contract stores a SHA-256 of
   the report. We propose hashing the RFC 8785 form of the full signed
   document, so the hash does not depend on formatting. (Assay's D3 testnet
   demo hashed the served file's bytes; v1 would switch.)
6. **Units and exchange rates.** Should a report be able to state the
   exchange rate it used, and from which source?
7. **Relation to on-chain attestation services.** A signed report could
   also be carried in an attestation protocol on Stellar (for example
   attest.so). Should this SEP name a schema for that?

## References

- SEP-1 v2.7.0, Stellar Info File:
  https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0001.md
- SEP-53 v1.0.0 (Final), Sign and Verify Messages:
  https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0053.md
- RFC 8785, JSON Canonicalization Scheme: https://www.rfc-editor.org/rfc/rfc8785
- Assay architecture, §4.4: [ARCHITECTURE.md](ARCHITECTURE.md)
