// reserve-report-v0: canonical form, signing and verification.
// Specification: docs/SEP-DRAFT-reserve-report.md.
//
//   payload   = JCS(report without "signature")      RFC 8785, UTF-8
//   signature = base64(SEP-53 signMessage(payload))  Ed25519 over
//               SHA-256("Stellar Signed Message:\n" + payload)

import { Keypair } from "@stellar/stellar-sdk";

export const REPORT_SEP = "reserve-report-v0";

export interface ReserveReportV0 {
  sep: typeof REPORT_SEP;
  /** `CODE:ISSUER` */
  asset: string;
  /** RFC 3339 date-time the figures are as of. */
  as_of: string;
  ledger?: number;
  reserves: { type?: string; amount: string; unit: string }[];
  liabilities: { supply: string; unit: string };
  attestor?: { name?: string; stellar_key?: string; role?: string; domain?: string };
  evidence?: { uri: string; sha256?: string }[];
  /** base64 Ed25519 signature (SEP-53) over the canonical payload. */
  signature?: string;
  [extra: string]: unknown;
}

// A high surrogate not followed by a low one, or a low one not preceded by a high one.
const LONE_SURROGATE = /[\uD800-\uDBFF](?![\uDC00-\uDFFF])|(?<![\uD800-\uDBFF])[\uDC00-\uDFFF]/;
const wellFormed = (s: string) => !LONE_SURROGATE.test(s);

type Json = null | boolean | number | string | Json[] | { [k: string]: Json };

/**
 * RFC 8785 JSON Canonicalization Scheme. Object keys sorted by UTF-16 code
 * units, no whitespace, ECMAScript number and string serialization.
 * Throws on values JSON cannot represent and on lone surrogates.
 */
export function canonicalize(value: unknown): string {
  return ser(value as Json);
}

function ser(v: Json): string {
  if (v === null || typeof v === "boolean") return JSON.stringify(v);
  if (typeof v === "number") {
    if (!Number.isFinite(v)) throw new TypeError("JCS: non-finite number");
    return JSON.stringify(v);
  }
  if (typeof v === "string") {
    if (!wellFormed(v)) throw new TypeError("JCS: lone surrogate in string");
    return JSON.stringify(v);
  }
  if (Array.isArray(v)) return `[${v.map(ser).join(",")}]`;
  if (typeof v === "object") {
    const keys = Object.keys(v)
      .filter((k) => v[k] !== undefined)
      .sort();
    for (const k of keys) {
      if (!wellFormed(k)) throw new TypeError("JCS: lone surrogate in key");
    }
    return `{${keys.map((k) => `${JSON.stringify(k)}:${ser(v[k]!)}`).join(",")}}`;
  }
  throw new TypeError(`JCS: cannot serialize ${typeof v}`);
}

/** A copy of the report without its `signature`. */
export function withoutSignature(report: ReserveReportV0): ReserveReportV0 {
  const copy = { ...report };
  delete copy.signature;
  return copy;
}

/** The bytes that are signed: the canonical report without `signature`. */
export function signingPayload(report: ReserveReportV0): Uint8Array {
  return new TextEncoder().encode(canonicalize(withoutSignature(report)));
}

/**
 * Returns a copy of the report signed with `keypair`. Sets
 * `attestor.stellar_key` to the keypair's public key first, so the key is
 * covered by the signature.
 */
export function signReport(report: ReserveReportV0, keypair: Keypair): ReserveReportV0 {
  const unsigned: ReserveReportV0 = {
    ...withoutSignature(report),
    attestor: { ...report.attestor, stellar_key: keypair.publicKey() },
  };
  const sig = keypair.signMessage(signingPayload(unsigned));
  return { ...unsigned, signature: Buffer.from(sig).toString("base64") };
}

export type VerifyResult =
  | { ok: true; key: string }
  | { ok: false; reason: "unsigned" | "no_key" | "bad_key" | "bad_signature" | "key_not_allowed" };

/**
 * Verifies a report's signature against `attestor.stellar_key`. When
 * `allowedKeys` is given (keys listed in stellar.toml), the key must be one
 * of them.
 */
export function verifyReport(report: ReserveReportV0, allowedKeys?: string[]): VerifyResult {
  if (!report.signature) return { ok: false, reason: "unsigned" };
  const key = report.attestor?.stellar_key;
  if (!key) return { ok: false, reason: "no_key" };
  let kp: Keypair;
  try {
    kp = Keypair.fromPublicKey(key);
  } catch {
    return { ok: false, reason: "bad_key" };
  }
  if (allowedKeys && !allowedKeys.includes(key)) return { ok: false, reason: "key_not_allowed" };
  const sig = Buffer.from(report.signature, "base64");
  if (sig.length !== 64) return { ok: false, reason: "bad_signature" };
  return kp.verifyMessage(signingPayload(report), new Uint8Array(sig))
    ? { ok: true, key }
    : { ok: false, reason: "bad_signature" };
}
