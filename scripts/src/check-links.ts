// Checks every link in README.md and docs/**/*.md:
//   - stellar.expert transaction links: the hash returns SUCCESS from
//     Stellar RPC on that network;
//   - stellar.expert contract links: the contract exists (stellar.expert API);
//   - web links: return HTTP 2xx (GET, redirects followed, 1 request/s per host);
//   - local links and #anchors: the file and heading exist.
// Issuers' reserve links quoted in CENSUS-v0.md are census data (their
// status was recorded on the census date), so they are reported separately
// and do not fail the check.
// Prints every failure; exits 1 if any. Read-only.
//
// Usage: pnpm --filter @garkuwa/assay-scripts check-links

import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { classify, extractLinks, needsPublicRepo, slugify, type Link } from "./lib/links.js";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "../..");
const RPC: Record<string, string> = {
  testnet: "https://soroban-testnet.stellar.org",
  public: "https://soroban-rpc.mainnet.stellar.gateway.fm",
};
const UA = "AssayLinkCheck/0.1 (+https://github.com/Garkuwa-Studio/assay)";
const CENSUS_DATA = /docs\/CENSUS-v0\.md$/;

/** Where to check a URL whose own page blocks or stalls automated requests. */
function viaApi(url: string): string {
  const npm = /^https:\/\/www\.npmjs\.com\/package\/(.+)$/.exec(url);
  if (npm) return `https://registry.npmjs.org/${npm[1]}`;
  // github.com file pages stall automated clients; the raw file is the same content.
  const blob = /^https:\/\/github\.com\/([^/]+)\/([^/]+)\/blob\/(.+?)(#.*)?$/.exec(url);
  if (blob) return `https://raw.githubusercontent.com/${blob[1]}/${blob[2]}/${blob[3]}`;
  return url;
}

function mdFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((n) => {
    const p = join(dir, n);
    if (statSync(p).isDirectory()) return mdFiles(p);
    return n.endsWith(".md") ? [p] : [];
  });
}

const lastHit = new Map<string, number>();
async function polite(url: string): Promise<void> {
  const host = new URL(url).host;
  const wait = (lastHit.get(host) ?? 0) + 1000 - Date.now();
  if (wait > 0) await new Promise((r) => setTimeout(r, wait));
  lastHit.set(host, Date.now());
}

async function web(url: string): Promise<string | null> {
  for (let attempt = 0; attempt < 3; attempt++) {
    await polite(url);
    try {
      const res = await fetch(viaApi(url), {
        redirect: "follow",
        // Some sites (crates.io) only serve the page to clients that ask for HTML.
        headers: { "user-agent": UA, accept: "text/html,application/json;q=0.9,*/*;q=0.8" },
        signal: AbortSignal.timeout(45_000),
      });
      await res.body?.cancel();
      if (res.ok) return null;
      if (res.status < 500 && res.status !== 429) return `HTTP ${res.status}`;
    } catch (e) {
      if (attempt === 2) return e instanceof Error ? e.message : String(e);
    }
    await new Promise((r) => setTimeout(r, 2000 * (attempt + 1)));
  }
  return "no answer after 3 tries";
}

async function tx(network: string, hash: string): Promise<string | null> {
  const rpc = RPC[network];
  if (!rpc) return `unknown network ${network}`;
  await polite(rpc);
  const res = await fetch(rpc, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method: "getTransaction", params: { hash } }),
  });
  const body = (await res.json()) as { result?: { status?: string } };
  const status = body.result?.status;
  return status === "SUCCESS" ? null : `RPC status ${status ?? "missing"}`;
}

async function contract(network: string, id: string): Promise<string | null> {
  return web(`https://api.stellar.expert/explorer/${network}/contract/${id}`);
}

function headings(file: string): Set<string> {
  const text = readFileSync(file, "utf8").replace(/```[\s\S]*?```/g, "");
  return new Set([...text.matchAll(/^#{1,6}\s+(.+)$/gm)].map((m) => slugify(m[1]!)));
}

function local(link: Link): string | null {
  const [path, anchor] = link.target.split("#") as [string, string | undefined];
  const target = path ? resolve(dirname(link.file), decodeURIComponent(path)) : link.file;
  if (!existsSync(target)) return "file not found";
  if (anchor && target.endsWith(".md") && !headings(target).has(anchor)) {
    return `heading #${anchor} not found`;
  }
  return null;
}

async function main() {
  const files = [join(ROOT, "README.md"), ...mdFiles(join(ROOT, "docs"))];
  const links = files.flatMap((f) => extractLinks(f, readFileSync(f, "utf8")));
  // Check each distinct remote target once.
  const remote = new Map<string, Promise<string | null>>();
  const results: { link: Link; problem: string | null }[] = [];
  for (const link of links) {
    const c = classify(link.target);
    let problem: string | null = null;
    if (c.kind === "local") problem = local(link);
    else if (c.kind === "anchor") {
      problem = headings(link.file).has(link.target.slice(1))
        ? null
        : `heading ${link.target} not found`;
    } else if (c.kind !== "mailto") {
      const key = link.target;
      if (!remote.has(key)) {
        remote.set(
          key,
          c.kind === "tx"
            ? tx(c.network!, c.id!)
            : c.kind === "contract"
              ? contract(c.network!, c.id!)
              : web(link.target),
        );
      }
      problem = await remote.get(key)!;
    }
    results.push({ link, problem });
  }

  const failed = results.filter((r) => r.problem);
  const census = failed.filter((r) => CENSUS_DATA.test(r.link.file));
  const rest = failed.filter((r) => !CENSUS_DATA.test(r.link.file));
  const blocked = rest.filter((r) => needsPublicRepo(r.link.target));
  const real = rest.filter((r) => !needsPublicRepo(r.link.target));
  const count = (k: string) => links.filter((l) => classify(l.target).kind === k).length;
  console.log(
    `checked ${links.length} links in ${files.length} files: ` +
      `${count("tx")} tx, ${count("contract")} contract, ${count("web")} web, ` +
      `${count("local") + count("anchor")} local (${remote.size} distinct remote)`,
  );
  const show = (r: (typeof results)[number]) =>
    `  ${relative(ROOT, r.link.file)}:${r.link.line}  ${r.link.target}  → ${r.problem}`;
  if (blocked.length) {
    console.log(
      `\n${blocked.length} need the Garkuwa-Studio repos pushed and public (expected until then):`,
    );
    for (const r of blocked) console.log(show(r));
  }
  if (census.length) {
    console.log(
      `\n${census.length} issuer links quoted as census data (status recorded in the census):`,
    );
    for (const r of census) console.log(show(r));
  }
  if (real.length) {
    console.log(`\n${real.length} FAILED:`);
    for (const r of real) console.log(show(r));
    process.exit(1);
  }
  console.log("\nno other failures");
}

main().catch((e: unknown) => {
  console.error(e instanceof Error ? e.message : e);
  process.exit(1);
});
