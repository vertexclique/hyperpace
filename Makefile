# Top-level Makefile scaffolded by vertexia init. One entrypoint for setup,
# build, test, the gate, and infrastructure. Adjust paths (INFRA_DIR) to the
# repo. Stack steps no-op when the tool or manifest is absent.
# House rule: infra changes are documented in docs/infra (make infra-log).

SHELL := /usr/bin/env bash
INFRA_DIR ?= infra
DATE := $(shell date +%Y-%m-%d)

.PHONY: help setup build test gate fmt lint doctor \
        tf-fmt tf-validate tf-plan tf-apply infra-log clean

help: ## list targets
	@grep -hE '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | sort | awk 'BEGIN{FS=":.*?## "}{printf "  %-14s %s\n",$$1,$$2}'

setup: ## install deps for whatever stacks are present
	@[ -f Cargo.toml ]      && cargo fetch || true
	@[ -f package.json ]    && { command -v npm >/dev/null && (npm ci || npm install) || true; } || true
	@[ -f pyproject.toml ]  && { command -v pip >/dev/null && pip install -e . || true; } || true

build: ## build whatever stacks are present
	@[ -f Cargo.toml ]   && cargo build --workspace || true
	@[ -f package.json ] && npm run build --if-present || true

test gate: ## run the stack-adaptive gate (format, lint, doc-warnings, tests)
	@vertexia gate .

fmt: ## format in place (scope to your code, never a vendored tree)
	@[ -f Cargo.toml ]   && cargo fmt || true
	@[ -f package.json ] && npm run format --if-present || true
	@command -v ruff >/dev/null && ruff format . || true

lint: ## lint, warnings as errors
	@[ -f Cargo.toml ] && cargo clippy --workspace --all-targets -- -D warnings || true
	@command -v ruff >/dev/null && ruff check . || true

doctor: ## vertexia + toolchain status
	@vertexia doctor .

# ---- infrastructure (documented, env-checked) --------------------------------

tf-fmt: ## terraform fmt
	@cd $(INFRA_DIR) && terraform fmt -recursive

tf-validate: ## terraform init -backend=false + validate
	@cd $(INFRA_DIR) && terraform init -backend=false -input=false >/dev/null && terraform validate

tf-plan: tf-validate ## check env, then terraform plan to a saved file
	@echo "env check:"; : "$${AWS_REGION:?set AWS_REGION}" ; echo "  AWS_REGION=$$AWS_REGION ok"
	@cd $(INFRA_DIR) && terraform plan -out=tfplan.$(DATE)
	@echo "plan saved: $(INFRA_DIR)/tfplan.$(DATE). Review it, then: make tf-apply"

tf-apply: ## apply the saved plan, then log it (infra traceability)
	@cd $(INFRA_DIR) && terraform apply tfplan.$(DATE)
	@echo "applied. Now document it: make infra-log"

infra-log: ## scaffold a new dated infra change entry from the template
	@mkdir -p docs/infra
	@cp -n docs/infra/TEMPLATE.md docs/infra/$(DATE)-change.md 2>/dev/null || true
	@echo "edit docs/infra/$(DATE)-change.md: what changed, how applied, env vars + checks, verification, rollback"

clean: ## remove build output
	@rm -rf target ui/build ui/node_modules/.cache
