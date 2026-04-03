.PHONY: all build-agent build-zkvm build-contracts test clean

all: build-agent build-contracts

build-agent:
	cd agent && cargo build --release

build-zkvm:
	cd zkvm && cargo build --release

build-contracts:
	cd contracts && forge build

test-agent:
	cd agent && cargo test

test-contracts:
	cd contracts && forge test

deploy-contracts:
	cd contracts && forge script script/Deploy.s.sol --rpc-url $(RPC_URL) --broadcast

run-agent:
	cd agent && cargo run

clean:
	cd agent && cargo clean
	cd zkvm && cargo clean
	cd contracts && forge clean

check:
	cd agent && cargo check
	cd contracts && forge build
