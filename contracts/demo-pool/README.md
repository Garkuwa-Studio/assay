# assay-demo-pool — TESTNET DEMO ONLY

**Not a lending product. Never deploy to mainnet.** No interest, no
liquidation, fixed 1:1 prices, no audit. It exists only to show
[Assay Guard](../guard) working as a circuit breaker on testnet.

- `deposit(user, amount)` takes the collateral token (a demo RWA token).
- `borrow(user, amount)` asks the guard `is_healthy(collateral, policy)`
  first. Healthy: lends the debt token (a demo stablecoin) up to the borrow
  limit and returns `Borrowed`. Unhealthy: moves nothing, emits
  `borrow_rejected` with the guard's reason and returns `Rejected(reason)`.
  The rejection is a successful transaction so the demo has one to link to;
  a production pool would more likely revert.
- `notice()` returns the testnet-only notice. Every event's first topic is
  `testnet_demo`.

Run: [docs/DEMO-RUN.md](../../docs/DEMO-RUN.md).
