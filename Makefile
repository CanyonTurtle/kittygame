normal:
	cargo build --release && install target/wasm32-unknown-unknown/release/cart.wasm ./kittygame.wasm && w4 bundle --html docs/index.html kittygame.wasm

size_opt:
	cargo build --release && binaryen-version_114/bin/wasm-opt -Oz -o kittygame.wasm target/wasm32-unknown-unknown/release/cart.wasm && w4 bundle kittygame.wasm --title "kittygame" --html docs/index.html && w4 run kittygame.wasm -n