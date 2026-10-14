// Signs a reserve-report-v0 document (docs/SEP-DRAFT-reserve-report.md).
//
// The key stays in the stellar CLI's keystore: this script asks
// `stellar message sign` (SEP-53) to sign the canonical payload and never
// reads the secret.
//
// Usage: pnpm --filter @garkuwa/assay-scripts sign-report <report.json> --key <identity> [--out signed.json]

import { spawn } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { parseArgs } from "node:util";
import {
  REPORT_SEP,
  signingPayload,
  verifyReport,
  withoutSignature,
  type ReserveReportV0,
} from "@garkuwa/assay-sdk";
import { address } from "./lib/cli.js";

/** Runs `stellar message sign` with the payload on stdin; returns the base64 signature. */
function cliSign(identity: string, payload: Uint8Array): Promise<string> {
  return new Promise((resolve, reject) => {
    const p = spawn("stellar", ["message", "sign", "--sign-with-key", identity, "--quiet"]);
    let out = "";
    let err = "";
    p.stdout.on("data", (d: Buffer) => (out += d.toString()));
    p.stderr.on("data", (d: Buffer) => (err += d.toString()));
    p.on("error", reject);
    p.on("close", (code) =>
      code === 0 ? resolve(out.trim()) : reject(new Error(`stellar message sign failed: ${err}`)),
    );
    p.stdin.end(Buffer.from(payload));
  });
}

async function main() {
  const { values, positionals } = parseArgs({
    allowPositionals: true,
    options: { key: { type: "string" }, out: { type: "string" } },
  });
  const file = positionals[0];
  if (!file || !values.key) {
    throw new Error("usage: sign-report <report.json> --key <identity> [--out signed.json]");
  }
  const report = JSON.parse(readFileSync(file, "utf8")) as ReserveReportV0;
  if (report.sep !== REPORT_SEP) throw new Error(`not a ${REPORT_SEP} document`);

  // The key goes inside the signed content, so it cannot be swapped later.
  const unsigned: ReserveReportV0 = {
    ...withoutSignature(report),
    attestor: { ...report.attestor, stellar_key: await address(values.key) },
  };
  const signed: ReserveReportV0 = {
    ...unsigned,
    signature: await cliSign(values.key, signingPayload(unsigned)),
  };

  const check = verifyReport(signed);
  if (!check.ok) throw new Error(`signature did not verify: ${check.reason}`);

  const out = values.out ?? file.replace(/\.json$/, ".signed.json");
  writeFileSync(out, `${JSON.stringify(signed, null, 2)}\n`);
  console.log(`signed by ${check.key}\nwrote ${out}`);
}

main().catch((e: unknown) => {
  console.error(e instanceof Error ? e.message : e);
  process.exit(1);
});
