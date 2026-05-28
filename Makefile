setup:
	cargo install wasm-pack
	npm install -g http-server

build:
	RUSTFLAGS='--cfg getrandom_backend="wasm_js"' wasm-pack build --target web --out-dir pkg

serve:
	http-server -p 8000

run: build serve

