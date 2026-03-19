# Jump10

A retro ASCII platformer reconstructed from a 1993 QBasic game. Built with Rust and macroquad.

## Controls

- **A/D** or **Left/Right arrows** — move
- **W**, **Up arrow**, or **Space** — jump (2 tiles)
- **P** — quit

## Tiles

- `#` — solid wall/platform
- `=` — hatched platform (solid)
- `^` — spikes (death)
- `z` — spring (launches up ~4 tiles)
- `/` — slide left
- `\` — slide right
- `@` — enemy patrol (kills on touch)
- `$` — player start
- `*` — level goal

## Run natively

```
cargo run
```

## Run in browser (WASM)

```
make web
```

This builds the WASM target, copies it to `web/`, and starts a local server at http://localhost:8080.

To just build without serving:

```
make wasm
```

## Requirements

- Rust with `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- Python 3 (for local web server)
