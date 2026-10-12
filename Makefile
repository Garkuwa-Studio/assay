.PHONY: build test e2e audit-scan fmt lint

build:
	cargo build --workspace
	stellar contract build
	pnpm -r build

test:
	cargo fmt --all -- --check
	cargo clippy --workspace --all-targets -- -D warnings
	cargo test --workspace
	pnpm lint
	pnpm typecheck
	pnpm test

e2e:
	@echo "e2e: not yet (testnet demo lands in demo sprint D4)"

audit-scan:
	@echo "audit-scan: not yet (cargo audit + Scout land in T2-D3)"

fmt:
	cargo fmt --all
	pnpm format
