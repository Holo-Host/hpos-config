# 
# Test and build Holo-config verify-node example
# 
#     Tests and wasm-packs a Node-compatible subset of holo-config's Config and Admin Key APIs,
# sufficient to verify AdminPublicKey signatures on messages, from a NodeJS application.
# 
SHELL	= bash

# nix-test, nix-build, ...
nix-%:
	nix-shell ../shell.nix --pure --run "make $*"

# Internal targets; assumes a nix-shell environment
.PHONY: all build test test-unit
all: build

build: pkg/holo_config_verify_node.js

pkg/holo_config_verify_node.js:
	wasm-pack build --scope holo-host --out-dir pkg --target nodejs

test: test-unit build
	node test/test_smoke.js

test-unit:
	RUST_BACKTRACE=1 cargo test -- --nocapture

# Run an nginx server w/ a self-signed SSL certificate/key, and test signature verification
test/server/ssl/selfsigned.crt:
	@echo -e \
	  "CA\nBC\nKelowna\nHolo Ltd.\n\nabc123.holohost.net\nperry.kundert@holo.host\n" \
	  | openssl req -x509 -newkey rsa:4096 -out $@ -keyout $@.key -days 365 -nodes

test-server: test/server/nginx.conf test/server/ssl/selfsigned.crt build
	nginx -c $<
