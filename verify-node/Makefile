
../target/wasm32-unknown-unknown/release/holo_config_verify_node.wasm:
	cargo build --target wasm32-unknown-unknown --release

pkg/holo_config_verify_node.js: \
		../target/wasm32-unknown-unknown/release/holo_config_verify_node.wasm
	#cargo install -f wasm-bindgen-cli # update to 0.2.53
	wasm-bindgen $^ --nodejs --out-dir pkg

test:		pkg/holo_config_verify_node.js
	node test/test_smoke.js
