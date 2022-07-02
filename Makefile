.phony: all

all: fmt clippy run

check: fmt clippy build

run:
	cargo run -- PNG_Test.png

fmt:
	cargo fmt

clippy:
	cargo clippy

build:
	cargo build