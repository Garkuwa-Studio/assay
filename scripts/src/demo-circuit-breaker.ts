// Circuit-breaker demo — TESTNET ONLY. Never point at mainnet.
//
// Deploys a fresh registry, Assay Guard and the TESTNET DEMO ONLY lending
// pool, then runs the D4 story:
//   1. fresh report at 101%    → borrow succeeds
//   2. attestor posts 92%      → borrow rejected (UNDERCOVERED)
//   3. fresh report at 100.4%  → borrow succeeds
//   4. no new report past the demo policy's 90 s max age → rejected (STALE)
// Each step prints its transaction hashes with stellar.expert links. The run
// is written to docs/DEMO-RUN.md and a receipt in docs/audit/e2e-receipts/.
//
// The figures are demo values, not a real asset's reserves.
//
// Needs testnet identities in the stellar CLI keystore (funded by Friendbot):
//   assay-admin, assay-issuer, assay-attestor-a, assay-witness, assay-borrower
//
// Usage: pnpm --filter @garkuwa/assay-scripts demo:circuit-breaker

import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { setTimeout as sleep } from "node:timers/promises";
import { fileURLToPath } from "node:url";
import {
  address,
  assertTestnet,
  expertContract,
  expertTx,
  latestLedger,
  stellar,
} from "./lib/cli.js";
import {
  outcomeText,
  parseOutcome,
  REASON_CODE,
  sameOutcome,
  STORY,
  waitForStale,
  type Outcome,
} from "./lib/pool-demo.js";
import { toUnits } from "./lib/registry-demo.js";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "../..");
const WASM = (name: string) => join(ROOT, `target/wasm32v1-none/release/${name}.wasm`);
const SUPPLY = 1_000_000;
const DEPOSIT = 1_000;
const BORROW = 100;
const POOL_LIQUIDITY = 100_000;
const POLICY = {
  min_coverage_bps: 10_000,
  max_age_secs: 90,
  min_attestors: 1,
  max_disagreement_bps: 500,
  max_supply_mismatch_bps: 50,
};
const LTV_BPS = 5_000;

interface Row {
  part: "setup" | "story";
  step: string;
  tx: string[];
  result?: string | undefined;
}
const rows: Row[] = [];

function record(part: Row["part"], step: string, hashes: string[], result?: string) {
  rows.push({ part, step, tx: hashes, result });
  console.log(`✓ ${step}${result ? ` → ${result}` : ""}`);
  for (const h of hashes) console.log(`    ${expertTx(h)}`);
}

const lastLine = (s: string) => s.split("\n").pop()!;

async function sacId(code: string, issuer: string, source: string): Promise<[string, string[]]> {
  const asset = `${code}:${issuer}`;
  try {
    const r = await stellar(["contract", "asset", "deploy", "--asset", asset, "--source", source]);
    return [lastLine(r.out), r.hashes];
  } catch {
    return [(await stellar(["contract", "id", "asset", "--asset", asset])).out, []];
  }
}

async function main() {
  await assertTestnet();
  const ids = [
    "assay-admin",
    "assay-issuer",
    "assay-attestor-a",
    "assay-witness",
    "assay-borrower",
  ];
  const [admin, issuer, attestor, witness, borrower] = await Promise.all(ids.map(address));

  // --- Setup --------------------------------------------------------------------
  await stellar(["contract", "build"], { cwd: ROOT, local: true });
  const wasmSha = Object.fromEntries(
    ["assay_registry", "assay_guard", "assay_demo_pool"].map((n) => [
      n,
      createHash("sha256")
        .update(readFileSync(WASM(n)))
        .digest("hex"),
    ]),
  );

  const deploy = async (name: string, args: string[]) => {
    const r = await stellar([
      "contract",
      "deploy",
      "--wasm",
      WASM(name),
      "--source",
      "assay-admin",
      "--",
      ...args,
    ]);
    return [lastLine(r.out), r.hashes] as const;
  };

  const [registry, regTx] = await deploy("assay_registry", ["--admin", admin!]);
  record("setup", `Deploy registry \`${registry}\``, regTx);
  const [guard, guardTx] = await deploy("assay_guard", ["--registry", registry]);
  record("setup", `Deploy guard \`${guard}\``, guardTx);

  const [ademo, ademoTx] = await sacId("ADEMO", issuer!, "assay-issuer");
  record("setup", `ADEMO token contract \`${ademo}\` (collateral)`, ademoTx);
  const [dusd, dusdTx] = await sacId("DUSD", admin!, "assay-admin");
  record("setup", `DUSD token contract \`${dusd}\` (demo stablecoin)`, dusdTx);

  const [pool, poolTx] = await deploy("assay_demo_pool", [
    "--admin",
    admin!,
    "--guard",
    guard,
    "--collateral",
    ademo,
    "--debt",
    dusd,
    "--policy",
    JSON.stringify(POLICY),
    "--ltv_bps",
    String(LTV_BPS),
  ]);
  record("setup", `Deploy demo pool \`${pool}\` (TESTNET DEMO ONLY)`, poolTx);

  const invoke = async (id: string, source: string, fn: string, args: string[]) =>
    stellar([
      "contract",
      "invoke",
      "--id",
      id,
      "--source",
      source,
      "--send",
      "yes",
      "--",
      fn,
      ...args,
    ]);

  record(
    "setup",
    "Registry: register ADEMO (unit USD)",
    (
      await invoke(registry, "assay-admin", "register_asset", [
        "--asset",
        ademo,
        "--asset_admin",
        issuer!,
        "--unit",
        "USD",
        "--max_age_secs",
        String(35 * 24 * 60 * 60),
        "--mismatch_tolerance_bps",
        "50",
      ])
    ).hashes,
  );
  record(
    "setup",
    "Registry: add attestor",
    (
      await invoke(registry, "assay-issuer", "add_attestor", [
        "--asset",
        ademo,
        "--attestor",
        attestor!,
      ])
    ).hashes,
  );
  record(
    "setup",
    "Registry: set supply witness",
    (
      await invoke(registry, "assay-issuer", "set_supply_witness", [
        "--asset",
        ademo,
        "--witness",
        witness!,
      ])
    ).hashes,
  );
  record(
    "setup",
    `Registry: witness posts supply ${SUPPLY.toLocaleString("en")} ADEMO`,
    (
      await invoke(registry, "assay-witness", "post_supply", [
        "--asset",
        ademo,
        "--amount",
        toUnits(SUPPLY),
        "--ledger",
        String((await latestLedger()) - 1),
      ])
    ).hashes,
  );

  for (const line of [`ADEMO:${issuer}`, `DUSD:${admin}`]) {
    const r = await stellar([
      "tx",
      "new",
      "change-trust",
      "--source",
      "assay-borrower",
      "--line",
      line,
    ]);
    record("setup", `Borrower trustline ${line.split(":")[0]}`, r.hashes);
  }
  record(
    "setup",
    `Issuer mints ${DEPOSIT.toLocaleString("en")} ADEMO to the borrower`,
    (await invoke(ademo, "assay-issuer", "mint", ["--to", borrower!, "--amount", toUnits(DEPOSIT)]))
      .hashes,
  );
  record(
    "setup",
    `Fund the pool with ${POOL_LIQUIDITY.toLocaleString("en")} DUSD`,
    (await invoke(dusd, "assay-admin", "mint", ["--to", pool, "--amount", toUnits(POOL_LIQUIDITY)]))
      .hashes,
  );
  record(
    "setup",
    `Borrower deposits ${DEPOSIT.toLocaleString("en")} ADEMO as collateral`,
    (
      await invoke(pool, "assay-borrower", "deposit", [
        "--user",
        borrower!,
        "--amount",
        toUnits(DEPOSIT),
      ])
    ).hashes,
  );

  // --- Story -------------------------------------------------------------------
  console.log("\nThe story:");
  let lastAsOf = 0;
  const outcomes: Outcome[] = [];
  for (const [i, s] of STORY.entries()) {
    const hashes: string[] = [];
    if (s.reserves != null) {
      lastAsOf = Math.max(lastAsOf + 1, Math.floor(Date.now() / 1000) - 10);
      const report = {
        reserves: toUnits(s.reserves),
        reserves_unit: "USD",
        liabilities: toUnits(SUPPLY),
        as_of: lastAsOf,
        ledger: (await latestLedger()) - 1,
        uri: "https://github.com/Garkuwa-Studio/assay/blob/main/docs/DEMO-RUN.md",
        sha256: createHash("sha256")
          .update(`demo report ${i + 1} ${lastAsOf}`)
          .digest("hex"),
      };
      const r = await invoke(registry, "assay-attestor-a", "post_report", [
        "--asset",
        ademo,
        "--attestor",
        attestor!,
        "--report",
        JSON.stringify(report),
      ]);
      hashes.push(...r.hashes);
    } else {
      const wait = waitForStale(lastAsOf, POLICY.max_age_secs, Math.floor(Date.now() / 1000));
      console.log(
        `  waiting ${wait} s for the last report to pass the ${POLICY.max_age_secs} s max age…`,
      );
      await sleep(wait * 1000);
    }
    const b = await invoke(pool, "assay-borrower", "borrow", [
      "--user",
      borrower!,
      "--amount",
      toUnits(BORROW),
    ]);
    hashes.push(...b.hashes);
    const outcome = parseOutcome(lastLine(b.out));
    outcomes.push(outcome);
    record("story", `${i + 1}. ${s.title}`, hashes, outcomeText(outcome));
  }

  const position = JSON.parse(
    lastLine(
      (
        await stellar([
          "contract",
          "invoke",
          "--id",
          pool,
          "--source",
          "assay-borrower",
          "--",
          "position",
          "--user",
          borrower!,
        ])
      ).out,
    ),
  ) as unknown;
  const mismatches = STORY.map((s, i) =>
    sameOutcome(s.expect, outcomes[i]!)
      ? null
      : `step ${i + 1}: expected ${outcomeText(s.expect)}, got ${outcomeText(outcomes[i]!)}`,
  ).filter((x): x is string => x != null);

  // --- Records ---------------------------------------------------------------------
  const date = new Date().toISOString().slice(0, 10);
  const receipt = {
    kind: "circuit-breaker-demo",
    network: "testnet",
    date,
    contracts: {
      registry: {
        id: registry,
        wasm_sha256: wasmSha.assay_registry,
        url: expertContract(registry),
      },
      guard: { id: guard, wasm_sha256: wasmSha.assay_guard, url: expertContract(guard) },
      demo_pool: { id: pool, wasm_sha256: wasmSha.assay_demo_pool, url: expertContract(pool) },
      ademo: { id: ademo, url: expertContract(ademo) },
      dusd: { id: dusd, url: expertContract(dusd) },
    },
    roles: { admin, asset_admin: issuer, attestor, witness, borrower },
    policy: POLICY,
    ltv_bps: LTV_BPS,
    steps: rows.map((r) => ({ ...r, urls: r.tx.map(expertTx) })),
    final_position: position,
    matches_expected: mismatches.length === 0,
  };
  writeFileSync(
    join(ROOT, `docs/audit/e2e-receipts/circuit-breaker-${date}.json`),
    `${JSON.stringify(receipt, null, 2)}\n`,
  );
  writeFileSync(join(ROOT, "docs/DEMO-RUN.md"), demoRunMd(date, receipt.contracts, rows));
  console.log(`\nwrote docs/DEMO-RUN.md and docs/audit/e2e-receipts/circuit-breaker-${date}.json`);
  if (mismatches.length) {
    console.error(mismatches.join("\n"));
    process.exit(1);
  }
}

const txLinks = (hs: string[]) =>
  hs.map((h) => `[\`${h.slice(0, 8)}…\`](${expertTx(h)})`).join(", ") || "already deployed";

function demoRunMd(
  date: string,
  c: Record<string, { id: string; url: string }>,
  rs: Row[],
): string {
  const story = rs.filter((r) => r.part === "story");
  const setup = rs.filter((r) => r.part === "setup");
  return `# Circuit-breaker demo run

> **TESTNET DEMO ONLY.** The lending pool is a deliberately tiny demo (no
> interest, no liquidation, fixed 1:1 prices) that exists only to show Assay
> Guard. Figures are demo values, not a real asset's reserves.

Run on ${date} by \`scripts/src/demo-circuit-breaker.ts\`. Each borrow asks
Assay Guard \`is_healthy(ADEMO, policy)\` first. A rejected borrow moves
nothing and records the guard's reason in a successful transaction (the pool
returns \`Rejected(reason)\` and emits \`borrow_rejected\`), so every step has a
transaction to open.

Demo policy: minimum coverage 100%, max report age **90 seconds** (so the
stale step can happen live), at least 1 attestor, disagreement up to 5%,
supply mismatch up to 0.5%. Borrow limit 50% of collateral.

## The story

| Step | Result | Transactions (report, then borrow) |
|---|---|---|
${story.map((r) => `| ${r.step.replace(/^\d+\. /, "")} | ${r.result} | ${txLinks(r.tx)} |`).join("\n")}

Reasons are the guard's: ${Object.values(REASON_CODE)
    .map((r) => `\`${r}\``)
    .join(", ")}.

## Contracts

| | |
|---|---|
| Registry | [\`${c.registry!.id}\`](${c.registry!.url}) |
| Assay Guard | [\`${c.guard!.id}\`](${c.guard!.url}) |
| Demo pool (TESTNET DEMO ONLY) | [\`${c.demo_pool!.id}\`](${c.demo_pool!.url}) |
| ADEMO (demo RWA token, collateral) | [\`${c.ademo!.id}\`](${c.ademo!.url}) |
| DUSD (demo stablecoin) | [\`${c.dusd!.id}\`](${c.dusd!.url}) |

## Setup transactions

| Step | Transactions |
|---|---|
${setup.map((r) => `| ${r.step} | ${txLinks(r.tx)} |`).join("\n")}

Receipt: [\`audit/e2e-receipts/circuit-breaker-${date}.json\`](audit/e2e-receipts/circuit-breaker-${date}.json).

Reproduce: \`pnpm --filter @garkuwa/assay-scripts demo:circuit-breaker\`
(deploys new contracts; needs the testnet identities named in the script).
`;
}

main().catch((e: unknown) => {
  console.error(e instanceof Error ? e.message : e);
  process.exit(1);
});
