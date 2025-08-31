

# Build
`RUSTFLAGS='--cfg getrandom_backend="wasm_js"' wasm-pack build --target web --out-dir pkg`

# Serve test site:
`npm install -g http-server`

`http-server -p 8000`

# Run natively
This isnt working at the moment