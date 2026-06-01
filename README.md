# Raspberry Pi Display

A small terminal dashboard for a monitor attached to a Raspberry Pi 4B.

The default screen renders a Clawd-inspired mascot walking horizontally above
Raspberry Pi stats:

- CPU temperature
- Network input/output rates
- Available storage

Clawd uses terminal cell background colors, which keeps the pixel-art shape more
consistent than Unicode block characters across different terminal fonts.

## Run

```bash
cargo run
```

Press `q`, `Esc`, or `Ctrl-C` to exit.

Press `c` to toggle a Clawd-coding-on-a-laptop scene.

Press `s` to toggle a suited Clawd scene.

Press `Tab` or `r` to cycle through dashboard, laptop, suited, and full-screen
roaming Clawd scenes.

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

Export sprite contact sheets for visual review:

```bash
cargo run -- --preview-sprites
```

PNG previews are written to `target/sprite-previews/`.
