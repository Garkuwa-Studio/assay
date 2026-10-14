# Demo video script (3 minutes)

For the SCF submission. Four parts: the problem (30 s), the board on real
mainnet data (60 s), the circuit breaker on testnet (60 s), and what the
grant funds (30 s). Every figure below has a source; say it as written.

**Before recording**

- `BOARD` = the board's address. **TODO:** set once the domain is live on
  Cloudflare.
- Open these tabs in order:
  1. `BOARD/board/`
  2. `BOARD/board/CETES-GCRYUGD5NVARGXT56XEZI5CIFCQETYHAPQQTHO2O3IQZTHDH4LATMYWC/`
  3. `BOARD/board/method/`
  4. [DEMO-RUN.md](DEMO-RUN.md) on GitHub
  5. Step 2's borrow: https://stellar.expert/explorer/testnet/tx/84d31e3575e94e9bad7d07d3462c2773f71c9d07e0cb7ec35b2cbf8621c09e3e
  6. Step 4's borrow: https://stellar.expert/explorer/testnet/tx/ec2c58476bb4bb7a12f977dd97b6143b5fe3400b42231bd1d0ad0254c3cbfa15
- Terminal ready to run `pnpm --filter @garkuwa/assay-scripts demo:circuit-breaker`
  (optional; it takes about 4 minutes, so show the recorded run instead and
  mention it can be re-run).
- Screen at 1440 px wide, browser zoom 110%, no other tabs or notifications.

---

## 0:00–0:30 · The problem

**On screen:** title card "Assay: open proof of reserves for Stellar", then
the board's summary numbers.

**Say:**

> Stellar has about four billion dollars of tokenized real-world assets.
> Lending markets accept about two million dollars of them as collateral.
> One reason: a risk team can't watch the backing. Issuers can link a
> reserve report from their stellar.toml, but it's optional, it's usually a
> PDF, and nothing reads it. In our census, of 270 assets that claim
> backing, 41 link a report, and none of those reports can be read by a
> machine.

**Sources:** $3.996B RWA market cap on 2026-08-29 (Cointelegraph citing
Stellar's Dune dashboard,
[STACK-STATUS §6](STACK-STATUS.md#6-reserve-verified-stablecoins)); Blend
RWA pools "just above $2M in TVL" (RedStone, 2026-08-26,
[STACK-STATUS §7](STACK-STATUS.md#7-lending-markets-and-how-they-stop-new-exposure));
270 / 41 / 0 from the 2026-10-12 census snapshot.

---

## 0:30–1:30 · The board, on real mainnet data

**On screen, tab 1 (board):**

1. Point to the summary: assets checked, links that work, reports dated in
   the last 35 days.
2. Change *Status* to "Report to read", then back to "All". Show page
   2 of the table.

> This is the board. Every week it checks every Stellar asset that claims
> backing: does the issuer link a reserve report, does the link work, how
> old is the report, and what does it say. These are facts with sources,
> not ratings.

**Tab 2 (CETES asset page):**

3. Show the stellar.toml fields, the reserve link and its HTTP result, and
   "report age: unknown".

> Here's Etherfuse's CETES. The issuer links a proof-of-reserves page and
> it loads, but it's a page a person has to read, so we can't tell its date
> or check its figures automatically. Every page has a link for the issuer
> to correct us.

**Tab 3 (method):**

4. Scroll past the rules, stop at "Limits".

> The method is public, and so is the data: the indexer is open source, and
> every weekly snapshot is published.

---

## 1:30–2:30 · The circuit breaker, on testnet

**On screen, tab 4 (DEMO-RUN.md):** the story table.

> Now the part that protects lending markets. Attestors post signed reserve
> reports to a registry contract. A guard contract turns them into one
> answer: is this asset healthy? Here a demo lending pool asks the guard
> before every borrow. This pool is a testnet demo only.

5. Row 1: "Report at 101%: the borrow goes through."
6. Row 2, then open **tab 5**. Show the event `borrow_rejected` with
   reason `Undercovered` and no transfer.

> The attestor posts 92%. The next borrow is rejected, and the reason is on
> chain: undercovered. No funds moved.

7. Row 3: "A fresh report at 100.4%: borrowing resumes."
8. Row 4, then open **tab 6**. Show reason `Stale`.

> Then the reports stop. For the demo, the policy allows reports up to 90
> seconds old. Once the last one passes that, the borrow is rejected again:
> stale. Missing or old data is never treated as healthy.

---

## 2:30–3:00 · What the grant funds

**On screen:** the roadmap table from [ARCHITECTURE.md §9](ARCHITECTURE.md#9-roadmap).

> The grant turns this into something protocols can rely on. First, the
> board and indexer v1, the registry v1 and the signed report standard,
> which is already drafted as a SEP. Then guard pilots with two attestors
> and one lending protocol. Then an audit and mainnet. Everything is open
> source under Apache 2.0. Thanks for watching.

**End card:** repository link, board address, "Facts, not ratings".

---

**After recording:** put the video link in the README (**TODO**) and
check that every tab above still loads.
