wasm:
	cargo build --release --target wasm32-unknown-unknown
	wasm-bindgen --no-typescript --out-name bevy_game --out-dir wasm --target web target/wasm32-unknown-unknown/release/bulletjam.wasm
	cp -r assets wasm/
	# cd wasm && zip --recurse-paths ../bulletjam.zip .
