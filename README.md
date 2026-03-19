# Jump10

[![Play Online](https://img.shields.io/badge/Play%20Online-GitHub%20Pages-brightgreen)](https://romaninsh.github.io/jump10/)
[![Original Game](https://img.shields.io/badge/Original%20Game-Facebook%20Reel-blue)](https://www.facebook.com/reel/626324263838416)

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

## Adding levels

1. Create a new file in `levels/` (e.g. `levels/03.txt`) — 60 columns wide, 13 rows tall
2. Add `include_str!("../levels/03.txt"),` to the `LEVELS` array in `src/level.rs`

See the **Tiles** section above for available tile characters. Every level needs a `$` (player start) and `*` (goal).

## Requirements

- Rust with `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- Python 3 (for local web server)
