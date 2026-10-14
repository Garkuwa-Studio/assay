// Pure helpers for check-links (tested in test/links.test.ts).

export interface Link {
  file: string;
  line: number;
  target: string;
}

const MD_LINK = /\[[^\]]*\]\(([^)\s]+)(?:\s+"[^"]*")?\)/g;
const BARE_URL = /(?<![("[`])\bhttps?:\/\/[^\s)>\]`"'|]+/g;

/** Markdown link targets and bare URLs, with line numbers. Code blocks are skipped. */
export function extractLinks(file: string, text: string): Link[] {
  const out: Link[] = [];
  let inFence = false;
  text.split("\n").forEach((raw, i) => {
    if (/^\s*```/.test(raw)) {
      inFence = !inFence;
      return;
    }
    if (inFence) return;
    // Inline code is not a link.
    const line = raw.replace(/`[^`]*`/g, "");
    const seen = new Set<string>();
    for (const m of line.matchAll(MD_LINK)) {
      seen.add(m[1]!);
      out.push({ file, line: i + 1, target: m[1]! });
    }
    for (const m of line.matchAll(BARE_URL)) {
      const t = m[0].replace(/[.,;:]+$/, "");
      if (!seen.has(t)) out.push({ file, line: i + 1, target: t });
    }
  });
  return out;
}

export type Kind = "tx" | "contract" | "web" | "local" | "anchor" | "mailto";

const TX = /stellar\.expert\/explorer\/(testnet|public)\/tx\/([0-9a-f]{64})/;
const CONTRACT = /stellar\.expert\/explorer\/(testnet|public)\/contract\/(C[A-Z2-7]{55})/;

export function classify(target: string): { kind: Kind; network?: string; id?: string } {
  const tx = TX.exec(target);
  if (tx) return { kind: "tx", network: tx[1]!, id: tx[2]! };
  const c = CONTRACT.exec(target);
  if (c) return { kind: "contract", network: c[1]!, id: c[2]! };
  if (/^https?:\/\//.test(target)) return { kind: "web" };
  if (target.startsWith("mailto:")) return { kind: "mailto" };
  if (target.startsWith("#")) return { kind: "anchor" };
  return { kind: "local" };
}

/** GitHub-style heading slug. */
export function slugify(heading: string): string {
  return heading
    .trim()
    .toLowerCase()
    .replace(/[^\p{L}\p{N}\s-]/gu, "")
    .replace(/\s/g, "-");
}

/** Links that need the Garkuwa-Studio repos to be pushed and public. */
export const needsPublicRepo = (url: string) =>
  /^https:\/\/(github\.com|raw\.githubusercontent\.com)\/Garkuwa-Studio\//.test(url);
