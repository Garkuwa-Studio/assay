// Pure helpers for the circuit-breaker demo (tested in test/pool-demo.test.ts).

export type Outcome = { kind: "Borrowed" } | { kind: "Rejected"; reason: string };

/**
 * Reads `borrow`'s return value as printed by `stellar contract invoke`:
 * `"Borrowed"` or `{"Rejected":["Undercovered"]}` (or `{"Rejected":"…"}`).
 */
export function parseOutcome(out: string): Outcome {
  const v = JSON.parse(out) as unknown;
  if (v === "Borrowed") return { kind: "Borrowed" };
  if (Array.isArray(v) && v[0] === "Borrowed") return { kind: "Borrowed" };
  if (v && typeof v === "object" && "Rejected" in v) {
    const r = (v as { Rejected: unknown }).Rejected;
    const reason = Array.isArray(r) ? r[0] : r;
    if (typeof reason === "string") return { kind: "Rejected", reason };
  }
  throw new Error(`unexpected borrow result: ${out}`);
}

/** The guard's reason names as written in the spec. */
export const REASON_CODE: Record<string, string> = {
  Ok: "OK",
  NoReport: "NO_REPORT",
  Stale: "STALE",
  TooFewAttestors: "TOO_FEW_ATTESTORS",
  Undercovered: "UNDERCOVERED",
  Disagreement: "DISAGREEMENT",
  SupplyMismatch: "SUPPLY_MISMATCH",
};

export const outcomeText = (o: Outcome) =>
  o.kind === "Borrowed"
    ? "borrow succeeded"
    : `borrow rejected (${REASON_CODE[o.reason] ?? o.reason})`;

export interface StoryStep {
  title: string;
  /** Reserves as a share of 1,000,000 liabilities, or null for no new report. */
  reserves: number | null;
  expect: Outcome;
}

/** The D4 story (11-demo-sprint-prompts.md). */
export const STORY: StoryStep[] = [
  {
    title: "Fresh report at 101%",
    reserves: 1_010_000,
    expect: { kind: "Borrowed" },
  },
  {
    title: "Attestor posts 92%",
    reserves: 920_000,
    expect: { kind: "Rejected", reason: "Undercovered" },
  },
  {
    title: "Fresh report at 100.4%",
    reserves: 1_004_000,
    expect: { kind: "Borrowed" },
  },
  {
    title: "No new report past the 90-second max age",
    reserves: null,
    expect: { kind: "Rejected", reason: "Stale" },
  },
];

export const sameOutcome = (a: Outcome, b: Outcome) =>
  a.kind === b.kind && (a.kind === "Borrowed" || a.reason === (b as { reason: string }).reason);

/** Seconds to wait so a report posted with `asOf` is older than `maxAge`. */
export function waitForStale(asOf: number, maxAge: number, nowSecs: number, margin = 15): number {
  return Math.max(0, asOf + maxAge + margin - nowSecs);
}
