.PHONY: all
all: help

.PHONY: accounts # Create deploy accounts
accounts:
	./scripts/create_accounts.sh

.PHONY: build # Build all contracts and copy wasm files to res/
build:
	./scripts/build.sh

.PHONY: deploy_amm # Deploy amm contract
deploy_amm:
	./scripts/deploy_amm.sh

.PHONY: deploy_fta # Deploy fta contract
deploy_fta:
	./scripts/deploy_fta.sh

.PHONY: deploy_ftb # Deploy ftb contract
deploy_ftb:
	./scripts/deploy_ftb.sh

.PHONY: test # Run unit tests
test:
	cargo test -- --nocapture

.PHONY: clean # Clean build files
clean:
	cargo clean
	rm -rf res/*.wasm

.PHONY: help # Generate list of targets with descriptions
help:
	@grep '^.PHONY: .* #' Makefile | sed 's/\.PHONY: \(.*\) # \(.*\)/\1	\2/' | expand -t20
