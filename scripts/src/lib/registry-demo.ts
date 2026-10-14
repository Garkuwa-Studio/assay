// Pure helpers for the registry demo (tested in test/registry-demo.test.ts).

const DECIMALS = 7n;

/** Whole tokens → integer string with 7 decimals. */
export function toUnits(whole: number): string {
  if (!Number.isSafeInteger(whole) || whole < 0) throw new Error(`bad amount ${whole}`);
  return (BigInt(whole) * 10n ** DECIMALS).toString();
}

/** Status the demo must produce: min(101%, 99.5%), spread 1.5 points. */
export const EXPECTED = {
  coverage_bps: 9950,
  attestor_count: 2,
  disagreement_bps: 150,
  supply_mismatch: false,
} as const;

export function checkStatus(status: Record<string, unknown>): string[] {
  return Object.entries(EXPECTED)
    .filter(([k, v]) => status[k] !== v)
    .map(([k, v]) => `${k} = ${String(status[k])}, expected ${String(v)}`);
}

/** A reserve-report-v0 document (docs/ARCHITECTURE.md §4.4), unsigned in v0. */
export function demoReport(o: {
  asset: string;
  asOf: number;
  ledger: number;
  reserves: number;
  supply: number;
  attestor: string;
  name: string;
}): string {
  const doc = {
    sep: "reserve-report-v0",
    note: "TESTNET DEMO. Figures are illustrative, not a real asset's reserves.",
    asset: o.asset,
    as_of: new Date(o.asOf * 1000).toISOString(),
    ledger: o.ledger,
    reserves: [{ type: "cash", amount: `${o.reserves}.00`, unit: "USD" }],
    liabilities: { supply: `${o.supply}.00`, unit: "ADEMO" },
    attestor: { name: o.name, stellar_key: o.attestor, role: "auditor" },
    evidence: [],
  };
  return `${JSON.stringify(doc, null, 2)}\n`;
}
