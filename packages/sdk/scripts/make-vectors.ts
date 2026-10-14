// Writes test-vectors/reserve-report-v0.json. The key is derived from a fixed
// public seed: TEST VECTORS ONLY, never use it for anything else.
//   pnpm --filter @garkuwa/assay-sdk vectors

import { writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { Keypair } from "@stellar/stellar-sdk";
import { signingPayload, signReport, type ReserveReportV0 } from "../src/report.js";

export const VECTOR_SEED_HEX = "a1".repeat(32);

export const VECTOR_REPORT: ReserveReportV0 = {
  sep: "reserve-report-v0",
  asset: "DEMO:GBQK5IGFFJUEMJJ6G4ZRU2N4ZBUUEV2OFYUQSEXLOI5NMV6FHJ6F7ZPC",
  as_of: "2026-09-30T23:59:59Z",
  ledger: 5181119,
  reserves: [
    { type: "cash", amount: "600000.00", unit: "USD" },
    { type: "government_bond", amount: "410000.00", unit: "USD" },
  ],
  liabilities: { supply: "1000000.00", unit: "DEMO" },
  attestor: { name: "Example Attestor (test vector)", role: "auditor" },
  evidence: [
    {
      uri: "https://example.org/reserves-2026-09.pdf",
      sha256: "0".repeat(64),
    },
  ],
};

export function makeVectors() {
  const kp = Keypair.fromRawEd25519Seed(Buffer.from(VECTOR_SEED_HEX, "hex"));
  const signed = signReport(VECTOR_REPORT, kp);
  const payload = new TextDecoder().decode(signingPayload(signed));
  return {
    note: "TEST VECTORS ONLY. The key comes from a published seed; never use it for real reports.",
    scheme: "payload = JCS(report without signature); signature = base64(SEP-53 Ed25519)",
    seed_hex: VECTOR_SEED_HEX,
    public_key: kp.publicKey(),
    signed_report: signed,
    canonical_payload: payload,
    payload_sha256: createHash("sha256").update(payload).digest("hex"),
    sep53_message_hash: createHash("sha256")
      .update(Buffer.concat([Buffer.from("Stellar Signed Message:\n"), Buffer.from(payload)]))
      .digest("hex"),
  };
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const out = new URL("../test-vectors/reserve-report-v0.json", import.meta.url);
  writeFileSync(out, `${JSON.stringify(makeVectors(), null, 2)}\n`);
  console.log(`wrote ${out.pathname}`);
}
