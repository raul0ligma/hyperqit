-include .env
export


run-strat:
	cargo run --bin strat

run-deployer:
	cargo run --bin deployer
	
run-dex:
	cargo run --bin dex

run-multisig:
	cargo run --bin multisig

debug:
	cargo build

debug-run: debug
	./target/debug/$(shell cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].name')

build:
	cargo build --release

run-prod: build
	./target/release/$(shell cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].name')

lint:
	cargo clippy
fix:
	cargo clippy --fix

all: lint test build

clean:
	cargo clean

test:
	cargo test

.PHONY: run debug debug-run build run-prod lint all clean test fix
