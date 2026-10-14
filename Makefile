.PHONY: build test e2e audit-scan fmt lint

DATE := $(shell date -u +%Y-%m-%d)
SCAN_DIR := docs/audit/dep-scan

build:
	cargo build --workspace
	stellar contract build
	pnpm -r build

test:
	cargo fmt --all -- --check
	cargo clippy --workspace --all-targets -- -D warnings
	cargo test --workspace
	pnpm lint
	pnpm -r build
	pnpm typecheck
	pnpm test

# TESTNET ONLY. Deploys fresh contracts and runs both demos; the scripts
# refuse any network but testnet. Needs the testnet identities named in
# scripts/src/demo-registry.ts and scripts/src/demo-circuit-breaker.ts.
e2e: build
	pnpm --filter @garkuwa/assay-scripts demo:registry
	pnpm --filter @garkuwa/assay-scripts demo:circuit-breaker

# Dependency and lint scans, saved as dated evidence. Needs cargo-audit
# (cargo install cargo-audit --locked). Findings go in
# docs/audit/remediation-log.md.
audit-scan:
	mkdir -p $(SCAN_DIR)
	cargo audit > $(SCAN_DIR)/$(DATE)-cargo-audit.txt 2>&1 || true
	cargo clippy --workspace --all-targets -- -D warnings > $(SCAN_DIR)/$(DATE)-clippy.txt 2>&1 || true
	pnpm audit > $(SCAN_DIR)/$(DATE)-pnpm-audit.txt 2>&1 || true
	# Keep local paths out of committed evidence.
	sed -i.bak "s#$(HOME)#~#g; s#$(CURDIR)#.#g" $(SCAN_DIR)/$(DATE)-*.txt && rm -f $(SCAN_DIR)/*.bak
	@tail -n 3 $(SCAN_DIR)/$(DATE)-cargo-audit.txt $(SCAN_DIR)/$(DATE)-clippy.txt $(SCAN_DIR)/$(DATE)-pnpm-audit.txt

fmt:
	cargo fmt --all
	pnpm format
