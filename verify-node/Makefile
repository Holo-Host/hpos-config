# 
# Test and build Holo-config verify-node example
# 
#     Tests and wasm-packs a Node-compatible subset of holo-config's Config and Admin Key APIs,
# sufficient to verify AdminPublicKey signatures on messages, from a NodeJS application.
# 
SHELL	= /bin/bash

# nix-test, nix-install, ...
nix-%:
	nix-shell ../shell.nix --pure --run "make $*"

# Internal targets; assumes a nix-shell environment
.PHONY: all test test-unit
all: pkg/holo_config_verify_node.js

pkg/holo_config_verify_node.js: src/lib.rs
	wasm-pack build --scope holo-host --out-dir pkg --target nodejs

test: test-unit all
	node test/test_smoke.js

test-unit:
	RUST_BACKTRACE=1 cargo test -- --nocapture

