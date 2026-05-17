CONTRACTS = nft-contract marketplace-contract atomic-swap-contract royalty-contract
TARGET = wasm32-unknown-unknown
OUT_DIR = target/$(TARGET)/release

.PHONY: build test clean fmt

build:
	cargo build --target $(TARGET) --release

test:
	cargo test

fmt:
	cargo fmt --all

clean:
	cargo clean

# Build a single contract: make build-nft-contract
$(addprefix build-,$(CONTRACTS)):
	cargo build -p $(subst build-,,$@) --target $(TARGET) --release
