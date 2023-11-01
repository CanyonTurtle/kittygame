all:
	cargo build --release && install target/wasm32-unknown-unknown/release/cart.wasm ./kittygame.wasm && w4 bundle --html docs/index.html kittygame.wasm