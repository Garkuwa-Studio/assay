import { readFileSync } from "node:fs";
import { Keypair } from "@stellar/stellar-sdk";
import { describe, expect, it } from "vitest";
import {
  canonicalize,
  signingPayload,
  signReport,
  verifyReport,
  type ReserveReportV0,
} from "../src/index.js";
import { makeVectors, VECTOR_REPORT, VECTOR_SEED_HEX } from "../scripts/make-vectors.js";

const vectors = JSON.parse(
  readFileSync(new URL("../test-vectors/reserve-report-v0.json", import.meta.url), "utf8"),
);
const testKey = () => Keypair.fromRawEd25519Seed(Buffer.from(VECTOR_SEED_HEX, "hex"));

describe("canonicalize (RFC 8785)", () => {
  it("sorts keys by UTF-16 code units (RFC 8785 §3.2.3 example)", () => {
    const input = {
      "€": "Euro Sign",
      "\r": "Carriage Return",
      דּ: "Hebrew Letter Dalet With Dagesh",
      "1": "One",
      "😀": "Emoji: Grinning Face",
      "\u0080": "Control",
      ö: "Latin Small Letter O With Diaeresis",
    };
    // Compared as a string: JSON.parse would move the integer-like key "1" first.
    const order = ["\r", "1", "\u0080", "\u00f6", "\u20ac", "\ud83d\ude00", "\ufb33"] as const;
    const expected = `{${order.map((k) => `${JSON.stringify(k)}:${JSON.stringify(input[k])}`).join(",")}}`;
    expect(canonicalize(input)).toBe(expected);
  });
  it("serializes numbers like ECMAScript and drops whitespace", () => {
    expect(canonicalize({ b: [1e30, 4.5, 0.002, 1e-7, -0], a: { z: true, y: null } })).toBe(
      '{"a":{"y":null,"z":true},"b":[1e+30,4.5,0.002,1e-7,0]}',
    );
  });
  it("escapes strings minimally", () => {
    expect(canonicalize({ s: 'q"\\\n\u0001é€' })).toBe('{"s":"q\\"\\\\\\n\\u0001é€"}');
  });
  it("is independent of key order", () => {
    expect(canonicalize({ a: 1, b: { d: 2, c: 3 } })).toBe(
      canonicalize({ b: { c: 3, d: 2 }, a: 1 }),
    );
  });
  it("rejects what JSON cannot represent", () => {
    expect(() => canonicalize({ n: Number.NaN })).toThrow(/non-finite/);
    expect(() => canonicalize({ n: Infinity })).toThrow(/non-finite/);
    expect(() => canonicalize({ s: "\ud800" })).toThrow(/surrogate/);
    expect(() => canonicalize({ f: () => 1 })).toThrow();
  });
});

describe("signReport / verifyReport", () => {
  it("round-trips and puts the key under the signature", () => {
    const kp = Keypair.random();
    const signed = signReport(VECTOR_REPORT, kp);
    expect(signed.attestor?.stellar_key).toBe(kp.publicKey());
    expect(verifyReport(signed)).toEqual({ ok: true, key: kp.publicKey() });
    expect(verifyReport(signed, [kp.publicKey()])).toEqual({ ok: true, key: kp.publicKey() });
  });
  it("is unaffected by key order or formatting", () => {
    const signed = signReport(VECTOR_REPORT, testKey());
    const reordered = JSON.parse(
      JSON.stringify(Object.fromEntries(Object.entries(signed).reverse()), null, 4),
    ) as ReserveReportV0;
    expect(verifyReport(reordered).ok).toBe(true);
  });
  it("detects any change to the figures", () => {
    const signed = signReport(VECTOR_REPORT, testKey());
    const tampered = {
      ...signed,
      liabilities: { ...signed.liabilities, supply: "900000.00" },
    };
    expect(verifyReport(tampered)).toEqual({ ok: false, reason: "bad_signature" });
  });
  it("detects a swapped key", () => {
    const signed = signReport(VECTOR_REPORT, testKey());
    const other = Keypair.random().publicKey();
    const swapped = { ...signed, attestor: { ...signed.attestor, stellar_key: other } };
    expect(verifyReport(swapped)).toEqual({ ok: false, reason: "bad_signature" });
  });
  it("names each failure", () => {
    const signed = signReport(VECTOR_REPORT, testKey());
    expect(verifyReport(VECTOR_REPORT)).toEqual({ ok: false, reason: "unsigned" });
    expect(verifyReport({ ...signed, attestor: {} })).toEqual({ ok: false, reason: "no_key" });
    expect(verifyReport({ ...signed, attestor: { stellar_key: "GBAD" } })).toEqual({
      ok: false,
      reason: "bad_key",
    });
    expect(verifyReport({ ...signed, signature: "AAAA" })).toEqual({
      ok: false,
      reason: "bad_signature",
    });
    expect(verifyReport(signed, [Keypair.random().publicKey()])).toEqual({
      ok: false,
      reason: "key_not_allowed",
    });
  });
  it("signs SEP-53 messages, not raw payloads", () => {
    const kp = testKey();
    const signed = signReport(VECTOR_REPORT, kp);
    const sig = Buffer.from(signed.signature!, "base64");
    // A raw Ed25519 signature over the payload must not verify as the report's.
    expect(kp.verify(Buffer.from(signingPayload(signed)), sig)).toBe(false);
    expect(kp.verifyMessage(signingPayload(signed), new Uint8Array(sig))).toBe(true);
  });
});

describe("test vectors", () => {
  it("match the committed file", () => {
    expect(makeVectors()).toEqual(vectors);
  });
  it("verify", () => {
    expect(verifyReport(vectors.signed_report, [vectors.public_key])).toEqual({
      ok: true,
      key: vectors.public_key,
    });
  });
});
