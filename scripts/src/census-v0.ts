// Census v0 — THROWAWAY research script for demo sprint D0. Not product code;
// the real census lives in assay-indexer (D1).
//
// Method:
//   1. Top N classic assets on Stellar mainnet by trustline count, from the
//      public stellar.expert API (sort=trustlines). Native XLM and Soroban
//      contract tokens (C... IDs, no classic issuer) are skipped and counted.
//   2. For each asset, the issuer's home_domain is read from Horizon (the
//      ledger), not from stellar.expert.
//   3. https://<home_domain>/.well-known/stellar.toml is fetched and parsed.
//      The CURRENCIES entry matching code + issuer is selected, following a
//      `toml = "..."` link if the entry points to a separate currency file.
//   4. If attestation_of_reserve is present, the link is requested (HEAD,
//      falling back to GET) and the HTTP status, content type and
//      Last-Modified header are recorded.
//   5. Supply is computed from the Horizon asset record (formula in
//      docs/STACK-STATUS.md) for context only.
//
// Polite: sequential requests, >= 1s between requests to the same host,
// identifying User-Agent, 15s timeouts. Public data only.
//
// Usage: tsx src/census-v0.ts [N] [out.json]
//        tsx src/census-v0.ts @ids.txt [out.json]   (explicit CODE-ISSUER list, one per line)

import { readFileSync, writeFileSync } from "node:fs";
import { parse as parseToml } from "smol-toml";

const UA = "AssayCensus/0.1 (+https://github.com/Garkuwa-Studio/assay)";
const HORIZON = "https://horizon.stellar.org";
const EXPERT = "https://api.stellar.expert/explorer/public";
const TIMEOUT_MS = 15_000;
const ARG = process.argv[2] ?? "50";
const ID_FILE = ARG.startsWith("@") ? ARG.slice(1) : null;
const N = ID_FILE ? 0 : Number(ARG);
const OUT = process.argv[3] ?? "census-v0.json";

const lastHit = new Map<string, number>();
async function politeFetch(url: string, init: RequestInit = {}): Promise<Response> {
  const host = new URL(url).host;
  const wait = (lastHit.get(host) ?? 0) + 1000 - Date.now();
  if (wait > 0) await new Promise((r) => setTimeout(r, wait));
  lastHit.set(host, Date.now());
  return fetch(url, {
    ...init,
    redirect: "follow",
    signal: AbortSignal.timeout(TIMEOUT_MS),
    headers: { "user-agent": UA, ...(init.headers ?? {}) },
  });
}

type Json = Record<string, unknown>;
const errMsg = (e: unknown) =>
  e instanceof Error
    ? e.cause instanceof Error
      ? `${e.message}: ${e.cause.message}`
      : e.message
    : String(e);

interface LinkCheck {
  method: "HEAD" | "GET";
  status: number | null;
  ok: boolean;
  final_url: string | null;
  content_type: string | null;
  last_modified: string | null;
  error: string | null;
}

async function checkLink(url: string): Promise<LinkCheck> {
  const read = (method: "HEAD" | "GET", r: Response): LinkCheck => ({
    method,
    status: r.status,
    ok: r.ok,
    final_url: r.url,
    content_type: r.headers.get("content-type"),
    last_modified: r.headers.get("last-modified"),
    error: null,
  });
  try {
    const head = await politeFetch(url, { method: "HEAD" });
    if (head.ok) return read("HEAD", head);
  } catch {
    // fall through to GET
  }
  try {
    const get = await politeFetch(url, { method: "GET" });
    await get.body?.cancel();
    return read("GET", get);
  } catch (e) {
    return {
      method: "GET",
      status: null,
      ok: false,
      final_url: null,
      content_type: null,
      last_modified: null,
      error: errMsg(e),
    };
  }
}

interface TomlResult {
  status: number | null;
  final_url: string | null;
  data: Json | null;
  error: string | null;
}

async function fetchToml(url: string): Promise<TomlResult> {
  try {
    const r = await politeFetch(url);
    if (!r.ok) {
      await r.body?.cancel();
      return { status: r.status, final_url: r.url, data: null, error: `HTTP ${r.status}` };
    }
    const text = await r.text();
    try {
      return { status: r.status, final_url: r.url, data: parseToml(text) as Json, error: null };
    } catch (e) {
      return { status: r.status, final_url: r.url, data: null, error: `parse: ${errMsg(e)}` };
    }
  } catch (e) {
    return { status: null, final_url: null, data: null, error: errMsg(e) };
  }
}

const sum = (...xs: (string | undefined)[]) =>
  xs.reduce((a, x) => a + (x ? BigInt(x.replace(".", "")) : 0n), 0n); // 7-decimal strings → stroops

async function main() {
  const isClassic = (r: Json) => /^[^-]+-G[A-Z2-7]{55}(-\d)?$/.test(String(r.asset));
  let listUrl: string;
  let records: Json[];
  if (ID_FILE) {
    listUrl = `file:${ID_FILE}`;
    records = readFileSync(ID_FILE, "utf8")
      .split("\n")
      .map((l) => l.trim())
      .filter(Boolean)
      .map((asset) => ({ asset }));
  } else {
    listUrl = `${EXPERT}/asset?sort=trustlines&order=desc&limit=${N + 20}`;
    records = ((await (await politeFetch(listUrl)).json()) as { _embedded: { records: Json[] } })
      ._embedded.records;
  }
  const skipped = records.filter((r) => r.asset !== "XLM" && !isClassic(r));
  const ranked = records.filter(isClassic).slice(0, ID_FILE ? undefined : N);
  const checkedAt = new Date().toISOString();
  const latest = (await (await politeFetch(HORIZON)).json()) as Json;

  const rows = [];
  for (const [i, rec] of ranked.entries()) {
    const [code, issuer] = String(rec.asset).split("-") as [string, string];
    const trustlines = (rec.trustlines as Json | undefined)?.total ?? rec.trustlines ?? null;
    process.stderr.write(`${i + 1}/${ranked.length} ${code}-${issuer.slice(0, 6)}… `);

    const acct = (await (await politeFetch(`${HORIZON}/accounts/${issuer}`)).json()) as Json;
    const homeDomain = (acct.home_domain as string | undefined) || null;

    const assetRes = (await (
      await politeFetch(`${HORIZON}/assets?asset_code=${code}&asset_issuer=${issuer}`)
    ).json()) as { _embedded: { records: Json[] } };
    const a = assetRes._embedded.records[0] as Json | undefined;
    const b = (a?.balances ?? {}) as Record<string, string>;
    const supplyStroops = a
      ? sum(
          b.authorized,
          b.authorized_to_maintain_liabilities,
          b.unauthorized,
          a.claimable_balances_amount as string,
          a.liquidity_pools_amount as string,
          a.contracts_amount as string,
        )
      : null;

    let toml: TomlResult | null = null;
    let currency: Json | null = null;
    let currencyVia: string | null = null;
    if (homeDomain) {
      toml = await fetchToml(`https://${homeDomain}/.well-known/stellar.toml`);
      const currencies = (toml.data?.CURRENCIES as Json[] | undefined) ?? [];
      for (const c of currencies) {
        if (c.code === code && c.issuer === issuer) {
          currency = c;
          currencyVia = "stellar.toml";
          break;
        }
        if (typeof c.toml === "string" && !c.code) {
          const linked = await fetchToml(c.toml);
          const lc = linked.data as Json | null;
          if (lc?.code === code && lc?.issuer === issuer) {
            currency = lc;
            currencyVia = c.toml;
            break;
          }
        }
      }
    }

    const aor =
      typeof currency?.attestation_of_reserve === "string" ? currency.attestation_of_reserve : null;
    const link = aor ? await checkLink(aor) : null;

    rows.push({
      rank: i + 1,
      code,
      issuer,
      trustlines,
      home_domain: homeDomain,
      toml_status: toml?.status ?? null,
      toml_final_url: toml?.final_url ?? null,
      toml_error: toml?.error ?? null,
      currency_listed: currency != null,
      currency_via: currencyVia,
      status: currency?.status ?? null,
      is_asset_anchored: currency?.is_asset_anchored ?? null,
      anchor_asset_type: currency?.anchor_asset_type ?? null,
      anchor_asset: currency?.anchor_asset ?? null,
      attestation_of_reserve: aor,
      has_redemption_instructions: typeof currency?.redemption_instructions === "string",
      collateral_addresses: Array.isArray(currency?.collateral_addresses)
        ? currency.collateral_addresses.length
        : 0,
      reserve_link: link,
      supply_stroops: supplyStroops?.toString() ?? null,
    });
    process.stderr.write(
      `${homeDomain ?? "no domain"} ${currency ? "listed" : "-"} ${aor ? `AoR ${link?.status}` : ""}\n`,
    );
  }

  writeFileSync(
    OUT,
    JSON.stringify(
      {
        method:
          "census-v0: top N by trustlines (stellar.expert), home_domain + supply from Horizon, SEP-1 stellar.toml",
        checked_at: checkedAt,
        horizon_ledger: latest.history_latest_ledger,
        protocol: latest.current_protocol_version,
        n: rows.length,
        skipped_non_classic: skipped.map((r) => r.asset),
        source_list: listUrl,
        rows,
      },
      null,
      2,
    ) + "\n",
  );
  process.stderr.write(`wrote ${OUT}\n`);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
