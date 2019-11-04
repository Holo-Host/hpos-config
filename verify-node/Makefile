SHELL	= /bin/bash

WBVER	= $(shell sed -nE 's/wasm-bindgen-cli = "([^"]*)"/wasm-bindgen \1/p' < Cargo.toml )
WBREQ	= $(shell wasm-bindgen --version )

.PHONY: wasm-bindgen-version
wasm-bindgen-version:
	@if [[ "$(WBVER)" != "$(WBREQ)" ]]; then \
	    echo "Updating $(WBVER) to $(WBREQ)"; \
	    cargo install -f wasm-bindgen-cli; \
	fi

../target/wasm32-unknown-unknown/release/holo_config_verify_node.wasm: wasm-bindgen-version
	cargo build --target wasm32-unknown-unknown --release

pkg/holo_config_verify_node.js: ../target/wasm32-unknown-unknown/release/holo_config_verify_node.wasm
	wasm-bindgen $^ --nodejs --out-dir pkg

test-unit:
	RUST_BACKTRACE=1 cargo test -- --nocapture

test: test-unit pkg/holo_config_verify_node.js
	node test/test_smoke.js
