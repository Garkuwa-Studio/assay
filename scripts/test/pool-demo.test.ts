import { describe, expect, it } from "vitest";
import {
  outcomeText,
  parseOutcome,
  sameOutcome,
  STORY,
  waitForStale,
} from "../src/lib/pool-demo.js";

describe("parseOutcome", () => {
  it("reads both result shapes", () => {
    expect(parseOutcome('"Borrowed"')).toEqual({ kind: "Borrowed" });
    expect(parseOutcome('["Borrowed"]')).toEqual({ kind: "Borrowed" });
    expect(parseOutcome('{"Rejected":["Undercovered"]}')).toEqual({
      kind: "Rejected",
      reason: "Undercovered",
    });
    expect(parseOutcome('{"Rejected":"Stale"}')).toEqual({ kind: "Rejected", reason: "Stale" });
  });
  it("refuses anything else", () => {
    expect(() => parseOutcome('"Paid"')).toThrow(/unexpected/);
    expect(() => parseOutcome("{}")).toThrow(/unexpected/);
  });
});

describe("story", () => {
  it("follows the spec: ok, UNDERCOVERED, ok, STALE", () => {
    expect(STORY.map((s) => outcomeText(s.expect))).toEqual([
      "borrow succeeded",
      "borrow rejected (UNDERCOVERED)",
      "borrow succeeded",
      "borrow rejected (STALE)",
    ]);
  });
  it("compares outcomes by kind and reason", () => {
    expect(sameOutcome({ kind: "Borrowed" }, { kind: "Borrowed" })).toBe(true);
    expect(
      sameOutcome({ kind: "Rejected", reason: "Stale" }, { kind: "Rejected", reason: "Stale" }),
    ).toBe(true);
    expect(
      sameOutcome({ kind: "Rejected", reason: "Stale" }, { kind: "Rejected", reason: "NoReport" }),
    ).toBe(false);
    expect(sameOutcome({ kind: "Borrowed" }, { kind: "Rejected", reason: "Stale" })).toBe(false);
  });
});

describe("waitForStale", () => {
  it("waits past max age plus a margin", () => {
    expect(waitForStale(1000, 90, 1030)).toBe(75);
    expect(waitForStale(1000, 90, 2000)).toBe(0);
  });
});
