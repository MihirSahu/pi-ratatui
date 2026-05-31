# Raspberry Pi Display

A small terminal dashboard for a monitor attached to a Raspberry Pi 4B.

The first screen renders a Clawd-inspired mascot that walks around the terminal.
It uses terminal cell background colors, which keeps the pixel-art shape more
consistent than Unicode block characters across different terminal fonts.

## Run

```bash
cargo run
```

Press `q`, `Esc`, or `Ctrl-C` to exit.

## Raspberry Pi

Install Rust on the Pi, clone this repo, then run:

```bash
cargo run --release
```

Release mode is recommended for faster startup on the Pi.

## Development

Format and check before committing:

```bash
cargo fmt
cargo check
```
