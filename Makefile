# 
# Test and build HoloFuel Config
# 
SHELL		= bash

# External targets; Uses a nix-shell environment to obtain Holochain runtimes, run tests, etc.
.PHONY: all
all: nix-test

# clean -- remove intermediate build artifacts
clean:
	rm -rf target \
	    holo-config-js/target \
	    holo-config-js/node_modules

# nix-test, nix-install, ...
nix-%:
	nix-shell --pure --run "make $*"


build:
	cargo build --lib --bin holo-config-generate --bin holo-config-derive

build-wasm:
	wasm-pack build \
	    --no-typescript \
	    --target nodejs \
	    --out-dir lib \
	    --out-name holo_config_js \
	    holo-config-js


test:		build test-unit test-e2e test-wasm

# test-wasm: Check/install required npm modules, and run tests
test-wasm:	build-wasm
	cd holo-config-js \
	    && ( [ -x node_modules/.bin/mocha ] || npm ci ) \
	    && npm run test:node

# test-unit -- Run Rust unit tests via Cargo
test-unit:
	RUST_BACKTRACE=1 cargo test \
	    -- --nocapture


# Generate a Config, emit it as JSON, then parse the JSON, and derive Agent and Admin private keys
# This shows that we can:
# - Generate a config and serialize it to JSON
#   - Eg. to configure a new HoloPort
# - Load a JSON Config, and use its Seed material to generate Signing private keys
#   - Eg. on boot-up of a HoloPort

test-e2e:	build
	./target/debug/holo-config-generate  \
	    --email a@b.ca \
	    --password password \
	    --seed-from ./test/etc/machine-id \
	| ./target/debug/holo-config-derive \
	    --email a@b.ca \
	    --password password

	./target/debug/holo-config-generate  \
	    --email a@b.ca \
	    --password password \
	    --encrypt \
	    --seed-from ./test/etc/machine-id \
	| ./target/debug/holo-config-derive \
	    --email a@b.ca \
	    --password password
