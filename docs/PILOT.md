# Pilots

Two one-page guides: one for issuers and attestors, one for lending
protocols. Both pilots run on **testnet** first and cost nothing. Questions:
open an issue on [Garkuwa-Studio/assay](https://github.com/Garkuwa-Studio/assay/issues).

---

## A. Issuers and attestors: publish a signed reserve report in 30 minutes

**What you get:** your asset's reserves shown on the Assay board as a
machine-readable, signed report, with its date and figures, instead of
"a report a person has to read". Lending protocols can check it
automatically.

**What changes for you:** you publish one small JSON file next to your
existing report and add two lines to your `stellar.toml`. Your current PDF
or web page stays as it is. The format is a draft SEP:
[SEP-DRAFT-reserve-report.md](SEP-DRAFT-reserve-report.md).

### Steps

**1. Write the report (10 min).** One file per asset, with the figures from
your latest attestation:

```json
{
  "sep": "reserve-report-v0",
  "asset": "YOURCODE:GYOURISSUER…",
  "as_of": "2026-09-30T23:59:59Z",
  "reserves": [{ "type": "cash", "amount": "1010000.00", "unit": "USD" }],
  "liabilities": { "supply": "1000000.00", "unit": "YOURCODE" },
  "attestor": { "name": "Your firm", "role": "auditor", "domain": "yourfirm.com" },
  "evidence": [{ "uri": "https://yourfirm.com/reports/2026-09.pdf", "sha256": "…" }]
}
```

Amounts are strings. `as_of` is the date the figures are true for. Get the
PDF's hash with `shasum -a 256 report.pdf`.

**2. Sign it (5 min).** With the [stellar CLI](https://developers.stellar.org/docs/tools/cli)
and a key you keep for reserve reports (not your issuing key):

```bash
stellar keys generate reserve-signer          # once; the secret stays in your keystore
git clone https://github.com/Garkuwa-Studio/assay && cd assay && pnpm install && pnpm -r build
pnpm --filter @garkuwa/assay-scripts sign-report "$(realpath ../report.json)" --key reserve-signer
```

(The script runs from `scripts/`, hence the absolute path.) This writes
`report.signed.json` next to your report. The signature uses SEP-53, so any tool
that signs SEP-53 messages works; the script only puts your key inside the
report and asks `stellar message sign` to sign it.

**3. Publish it (10 min).** Host `report.signed.json` over HTTPS with
`Content-Type: application/json` and `Access-Control-Allow-Origin: *`, the
same way you serve `stellar.toml`.

**4. Point to it (5 min).** In the issuer's `stellar.toml`:

```toml
[[CURRENCIES]]
code = "YOURCODE"
issuer = "GYOURISSUER…"
attestation_of_reserve = "https://yourdomain.com/reserves/latest.json"
attestation_of_reserve_signers = ["GYOURSIGNINGKEY…"]
```

If a separate firm signs, it adds its key to its own `stellar.toml`:
`RESERVE_REPORT_KEYS = ["GYOURSIGNINGKEY…"]`.

**Done.** The board picks it up at the next weekly census. On testnet, we
can also register your asset in the registry contract so you can see the
on-chain status.

**Good to know:** a signature says who made the statement, not that it is
true. Reports are public. Each new report needs a later `as_of`.

---

## B. Lending protocols: add one `is_healthy` call

**What you get:** a way to stop new exposure to an RWA automatically when
its reserve reports go missing, go stale, or show a shortfall, with a
readable reason. The guard is read-only: it never moves funds or touches
your contracts.

### The call

```rust
use soroban_sdk::{contractclient, contracttype, Address, Env};

#[contracttype]
pub struct Policy {
    pub min_coverage_bps: u32,        // 10_000 = 100%
    pub max_age_secs: u64,            // e.g. 35 days
    pub min_attestors: u32,           // at least 1
    pub max_disagreement_bps: u32,    // e.g. 500 = 5 points
    pub max_supply_mismatch_bps: u32, // e.g. 50 = 0.5%
}

#[contracttype]
#[derive(Clone, Copy, PartialEq)]
pub enum Reason { Ok, NoReport, Stale, TooFewAttestors, Undercovered, Disagreement, SupplyMismatch }

#[contractclient(name = "GuardClient")]
pub trait Guard {
    fn is_healthy(env: Env, asset: Address, policy: Policy) -> (bool, Reason);
}

// In borrow() or supply_collateral(), before accepting the RWA:
let (healthy, reason) = GuardClient::new(&env, &guard_id).is_healthy(&rwa_token, &policy);
if !healthy {
    // Your choice: revert with your own error, or record the rejection.
}
```

Full working example: [`contracts/demo-pool`](../contracts/demo-pool)
(testnet demo only) and its run, [DEMO-RUN.md](DEMO-RUN.md).

### What the reasons mean

| Reason | When |
|---|---|
| `NoReport` | No report has ever been posted, or the asset isn't registered. |
| `Stale` | Reports exist, but none is newer than `max_age_secs`. |
| `TooFewAttestors` | Fewer fresh attestors than `min_attestors`. |
| `Undercovered` | The lowest fresh coverage is below `min_coverage_bps`. |
| `Disagreement` | Attestors' coverage figures differ by more than `max_disagreement_bps`. |
| `SupplyMismatch` | On-chain supply is above what a report says is owed, beyond the tolerance. |

Missing data is always unhealthy; there is no setting to change that.

### If you can't change your contracts

A keeper can call your existing admin action instead (for example, a Blend
pool's `set_status` to stop new borrowing) through an account allowed to
call **only** that function on **only** that pool. Blend's per-reserve
switch has a one-week timelock, so the immediate option is pool-wide
([STACK-STATUS §7](STACK-STATUS.md#7-lending-markets-and-how-they-stop-new-exposure)).
We will build this with the first protocol pilot.

### Pilot steps

1. Choose a testnet pool and an RWA, and the policy you'd want.
2. We register the asset in the testnet registry and post supply.
3. You add the call (or we set up the keeper) and run your tests against
   the testnet guard: `CCVEIPYTY6RRVJZONFZL4WVGN66H5IU77H3HSSQJKH7I7WSDL6DIEKDV`.
4. We run the same four-step story as [DEMO-RUN.md](DEMO-RUN.md) on your
   pool and publish the transactions, with your agreement.
