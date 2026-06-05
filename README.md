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

### Run on the attached monitor

Build the release binary:

```bash
cargo build --release
```

Create a systemd service that owns `tty1`:

```bash
sudo nano /etc/systemd/system/pi-tui.service
```

Use this service file, adjusting `WorkingDirectory` and `ExecStart` if the repo
is not in `/home/pi/Downloads/pi-ratatui`:

```ini
[Unit]
Description=Pi TUI Dashboard
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=pi
WorkingDirectory=/home/pi/Downloads/pi-ratatui
ExecStart=/home/pi/Downloads/pi-ratatui/target/release/Raspberry-Pi-Display

StandardInput=tty
StandardOutput=tty
StandardError=journal
TTYPath=/dev/tty1
TTYReset=yes
TTYVHangup=yes
TTYVTDisallocate=yes

Restart=always
RestartSec=2

[Install]
WantedBy=multi-user.target
```

Enable and start it:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now pi-tui
```

After editing the service, reload systemd before restarting:

```bash
sudo systemctl daemon-reload
sudo systemctl restart pi-tui
```

Check status and logs:

```bash
systemctl status pi-tui
journalctl -u pi-tui -f
```

### Console font

The dashboard size depends on the Linux console font because it renders with
terminal cells. Configure a larger font with:

```bash
sudo dpkg-reconfigure console-setup
```

Recommended choices:

- Encoding: `UTF-8`
- Character set: `Guess optimal character set`
- Font: `TerminusBold`
- Font size: `16x32`

Apply the font immediately:

```bash
sudo systemctl restart console-setup
sudo systemctl restart pi-tui
```

For a one-off test on `tty1`, stop the dashboard, set the font, then start it:

```bash
sudo systemctl stop pi-tui
sudo setfont /usr/share/consolefonts/Uni2-TerminusBold32x16.psf.gz -C /dev/tty1
sudo systemctl start pi-tui
```

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
