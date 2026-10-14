// Registry demo — TESTNET ONLY. Never point at mainnet.
//
// Deploys contracts/registry, registers a demo asset (ADEMO, unit USD),
// adds two attestors, sets a supply witness, posts supply, posts two
// reports (101% and 99.5%) and reads status. Every transaction hash is
// printed with a stellar.expert link and saved as a receipt in
// docs/audit/e2e-receipts/.
//
// The figures are demo values, not a real asset's reserves.
//
// Needs testnet identities in the stellar CLI keystore (funded by Friendbot):
//   assay-admin, assay-issuer, assay-attestor-a, assay-attestor-b, assay-witness
//
// Usage: pnpm --filter @garkuwa/assay-scripts demo:registry

import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  address,
  assertTestnet,
  expertContract,
  expertTx,
  latestLedger,
  stellar,
} from "./lib/cli.js";
import { checkStatus, demoReport, EXPECTED, toUnits } from "./lib/registry-demo.js";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "../..");
const WASM = join(ROOT, "target/wasm32v1-none/release/assay_registry.wasm");
const REPORT_DIR = "docs/testnet/reports";
const RAW_BASE = "https://raw.githubusercontent.com/Garkuwa-Studio/assay/main";
const MAX_AGE_SECS = 35 * 24 * 60 * 60;
const TOLERANCE_BPS = 50;
const SUPPLY = 1_000_000;

interface Step {
  step: string;
  tx: string[];
}
const steps: Step[] = [];

function record(step: string, hashes: string[]) {
  steps.push({ step, tx: hashes });
  console.log(`✓ ${step}`);
  for (const h of hashes) console.log(`    ${expertTx(h)}`);
}

async function main() {
  await assertTestnet();
  const [admin, issuer, attA, attB, witness] = await Promise.all(
    ["assay-admin", "assay-issuer", "assay-attestor-a", "assay-attestor-b", "assay-witness"].map(
      address,
    ),
  );

  // 1. Build and deploy the registry.
  await stellar(["contract", "build", "--package", "assay-registry"], { cwd: ROOT, local: true });
  const wasmSha = createHash("sha256").update(readFileSync(WASM)).digest("hex");
  const deploy = await stellar([
    "contract",
    "deploy",
    "--wasm",
    WASM,
    "--source",
    "assay-admin",
    "--",
    "--admin",
    admin!,
  ]);
  const registry = deploy.out.split("\n").pop()!;
  record(`deploy registry ${registry}`, deploy.hashes);

  // 2. The demo asset's token contract (deploys once; reused after).
  const assetId = `ADEMO:${issuer}`;
  let sac: string;
  try {
    const r = await stellar([
      "contract",
      "asset",
      "deploy",
      "--asset",
      assetId,
      "--source",
      "assay-issuer",
    ]);
    sac = r.out.split("\n").pop()!;
    record(`deploy ADEMO token contract ${sac}`, r.hashes);
  } catch {
    sac = (await stellar(["contract", "id", "asset", "--asset", assetId])).out;
    console.log(`✓ ADEMO token contract already deployed: ${sac}`);
  }

  const invoke = (source: string, fn: string, args: string[]) =>
    stellar(["contract", "invoke", "--id", registry, "--source", source, "--", fn, ...args]);

  // 3. Writes.
  record(
    "register_asset ADEMO (unit USD, max age 35 days, tolerance 50 bps)",
    (
      await invoke("assay-admin", "register_asset", [
        "--asset",
        sac,
        "--asset_admin",
        issuer!,
        "--unit",
        "USD",
        "--max_age_secs",
        String(MAX_AGE_SECS),
        "--mismatch_tolerance_bps",
        String(TOLERANCE_BPS),
      ])
    ).hashes,
  );
  for (const [name, a] of [
    ["attestor A", attA!],
    ["attestor B", attB!],
  ] as const) {
    record(
      `add_attestor ${name}`,
      (await invoke("assay-issuer", "add_attestor", ["--asset", sac, "--attestor", a])).hashes,
    );
  }
  record(
    "set_supply_witness",
    (await invoke("assay-issuer", "set_supply_witness", ["--asset", sac, "--witness", witness!]))
      .hashes,
  );

  const supplyLedger = (await latestLedger()) - 1;
  record(
    `post_supply ${SUPPLY.toLocaleString("en")} ADEMO at ledger ${supplyLedger}`,
    (
      await invoke("assay-witness", "post_supply", [
        "--asset",
        sac,
        "--amount",
        toUnits(SUPPLY),
        "--ledger",
        String(supplyLedger),
      ])
    ).hashes,
  );

  // 4. Two reserve reports, written as reserve-report-v0 documents whose
  // SHA-256 goes on-chain.
  mkdirSync(join(ROOT, REPORT_DIR), { recursive: true });
  const asOf = Math.floor(Date.now() / 1000) - 60;
  const reportLedger = (await latestLedger()) - 1;
  for (const [label, identity, who, reserves] of [
    ["a", "assay-attestor-a", attA!, 1_010_000],
    ["b", "assay-attestor-b", attB!, 995_000],
  ] as const) {
    const file = `${REPORT_DIR}/attestor-${label}.json`;
    const doc = demoReport({
      asset: assetId,
      asOf,
      ledger: reportLedger,
      reserves,
      supply: SUPPLY,
      attestor: who,
      name: `Demo attestor ${label.toUpperCase()}`,
    });
    writeFileSync(join(ROOT, file), doc);
    const sha = createHash("sha256").update(doc).digest("hex");
    const report = {
      reserves: toUnits(reserves),
      reserves_unit: "USD",
      liabilities: toUnits(SUPPLY),
      as_of: asOf,
      ledger: reportLedger,
      uri: `${RAW_BASE}/${file}`,
      sha256: sha,
    };
    record(
      `post_report attestor ${label.toUpperCase()} (${((reserves / SUPPLY) * 100).toFixed(1)}%)`,
      (
        await invoke(identity, "post_report", [
          "--asset",
          sac,
          "--attestor",
          who,
          "--report",
          JSON.stringify(report),
        ])
      ).hashes,
    );
  }

  // 5. Read status (simulation only, no transaction).
  const status = JSON.parse(
    (await invoke("assay-admin", "status", ["--asset", sac])).out,
  ) as Record<string, unknown>;
  console.log("\nstatus(ADEMO):", status);
  const problems = checkStatus(status);

  const date = new Date().toISOString().slice(0, 10);
  const receipt = {
    kind: "registry-demo",
    network: "testnet",
    date,
    registry: { contract: registry, wasm_sha256: wasmSha, url: expertContract(registry) },
    asset: { code: "ADEMO", issuer, contract: sac },
    roles: { admin, asset_admin: issuer, attestors: [attA, attB], witness },
    steps: steps.map((s) => ({ ...s, urls: s.tx.map(expertTx) })),
    status,
    expected: EXPECTED,
    matches_expected: problems.length === 0,
  };
  const out = join(ROOT, `docs/audit/e2e-receipts/registry-${date}.json`);
  writeFileSync(out, `${JSON.stringify(receipt, null, 2)}\n`);
  console.log(`\nreceipt: ${out}`);
  if (problems.length) {
    console.error(`status differs from expected: ${problems.join("; ")}`);
    process.exit(1);
  }
}

main().catch((e: unknown) => {
  console.error(e instanceof Error ? e.message : e);
  process.exit(1);
});
