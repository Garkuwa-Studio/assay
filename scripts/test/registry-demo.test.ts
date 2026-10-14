import { describe, expect, it } from "vitest";
import { txHashes } from "../src/lib/cli.js";
import { checkStatus, demoReport, toUnits } from "../src/lib/registry-demo.js";

describe("toUnits", () => {
  it("adds 7 decimals", () => {
    expect(toUnits(1_000_000)).toBe("10000000000000");
    expect(toUnits(0)).toBe("0");
  });
  it("rejects fractions and negatives", () => {
    expect(() => toUnits(1.5)).toThrow();
    expect(() => toUnits(-1)).toThrow();
  });
});

describe("txHashes", () => {
  it("reads every signed hash from CLI stderr", () => {
    const a = "a".repeat(64);
    const b = "0b".repeat(32);
    const stderr = `ℹ️  Simulating transaction…\nℹ️  Signing transaction: ${a}\n🌎 Sending…\nℹ️  Signing transaction: ${b}\n`;
    expect(txHashes(stderr)).toEqual([a, b]);
    expect(txHashes("no transaction")).toEqual([]);
  });
});

describe("checkStatus", () => {
  it("accepts the expected demo status", () => {
    expect(
      checkStatus({
        coverage_bps: 9950,
        newest_age_secs: 61,
        attestor_count: 2,
        disagreement_bps: 150,
        supply_mismatch: false,
        supply_age_secs: 70,
      }),
    ).toEqual([]);
  });
  it("names each difference", () => {
    expect(
      checkStatus({
        coverage_bps: 10100,
        attestor_count: 1,
        disagreement_bps: 0,
        supply_mismatch: false,
      }),
    ).toEqual([
      "coverage_bps = 10100, expected 9950",
      "attestor_count = 1, expected 2",
      "disagreement_bps = 0, expected 150",
    ]);
  });
});

describe("demoReport", () => {
  it("is labelled as a testnet demo and stable for hashing", () => {
    const o = {
      asset: "ADEMO:GABC",
      asOf: 1_800_000_000,
      ledger: 42,
      reserves: 1_010_000,
      supply: 1_000_000,
      attestor: "GXYZ",
      name: "Demo attestor A",
    };
    const doc = demoReport(o);
    expect(doc).toBe(demoReport(o));
    const parsed = JSON.parse(doc);
    expect(parsed.sep).toBe("reserve-report-v0");
    expect(parsed.note).toMatch(/TESTNET DEMO/);
    expect(parsed.as_of).toBe("2027-01-15T08:00:00.000Z");
    expect(parsed.liabilities).toEqual({ supply: "1000000.00", unit: "ADEMO" });
  });
});
