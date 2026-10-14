import { describe, expect, it } from "vitest";
import { classify, extractLinks, needsPublicRepo, slugify } from "../src/lib/links.js";

describe("extractLinks", () => {
  it("finds markdown links and bare URLs with line numbers", () => {
    const md = "See [a](docs/A.md) and https://example.org/x.\n\n[b](https://b.org/y)";
    expect(extractLinks("f.md", md)).toEqual([
      { file: "f.md", line: 1, target: "docs/A.md" },
      { file: "f.md", line: 1, target: "https://example.org/x" },
      { file: "f.md", line: 3, target: "https://b.org/y" },
    ]);
  });
  it("skips code blocks and inline code", () => {
    const md = "```\n[x](nope.md) https://nope.org\n```\n`https://also-nope.org` [ok](ok.md)";
    expect(extractLinks("f.md", md).map((l) => l.target)).toEqual(["ok.md"]);
  });
  it("does not double-count a URL inside a markdown link", () => {
    expect(extractLinks("f.md", "[x](https://a.org/p)")).toHaveLength(1);
  });
});

describe("classify", () => {
  const h = "a".repeat(64);
  const c = `C${"A".repeat(55)}`;
  it("recognizes stellar.expert transactions and contracts", () => {
    expect(classify(`https://stellar.expert/explorer/testnet/tx/${h}`)).toEqual({
      kind: "tx",
      network: "testnet",
      id: h,
    });
    expect(classify(`https://stellar.expert/explorer/public/contract/${c}`)).toEqual({
      kind: "contract",
      network: "public",
      id: c,
    });
  });
  it("sorts the rest", () => {
    expect(classify("https://x.org").kind).toBe("web");
    expect(classify("docs/A.md#b").kind).toBe("local");
    expect(classify("#heading").kind).toBe("anchor");
    expect(classify("mailto:a@b.c").kind).toBe("mailto");
  });
});

describe("slugify", () => {
  it("matches GitHub heading anchors", () => {
    expect(slugify('6. "Reserve-verified" stablecoins')).toBe("6-reserve-verified-stablecoins");
    expect(slugify("7. Lending markets and how they stop new exposure")).toBe(
      "7-lending-markets-and-how-they-stop-new-exposure",
    );
    expect(slugify("9. Roadmap")).toBe("9-roadmap");
  });
});

describe("needsPublicRepo", () => {
  it("flags only the org's GitHub links", () => {
    expect(needsPublicRepo("https://github.com/Garkuwa-Studio/assay")).toBe(true);
    expect(
      needsPublicRepo("https://raw.githubusercontent.com/Garkuwa-Studio/assay/main/x.json"),
    ).toBe(true);
    expect(needsPublicRepo("https://github.com/stellar/stellar-protocol")).toBe(false);
  });
});
