.PHONY: run wasm web

run:
	cargo run

wasm:
	cargo build --target wasm32-unknown-unknown --release
	cp target/wasm32-unknown-unknown/release/jump10-bas.wasm web/

web: wasm
	@echo "Serving at http://localhost:8080"
	python3 -m http.server 8080 -d web
