<p align="center">
  <img src="data/icons/io.github.gabrielbaiano.Deskstamp.png" alt="Deskstamp Logo" width="160"/>
</p>

<h1 align="center">Deskstamp</h1>

<p align="center">
  <strong>Native desktop watermark overlay for Linux & Pop!_OS COSMIC.</strong><br>
  <em>Real-time screen watermarking with seamless click-through and zero idle CPU overhead.</em>
</p>

<p align="center">
  <a href="https://github.com/GabrielBaiano/Deskstamp"><img src="https://img.shields.io/badge/Desktop-Pop!_OS%20COSMIC-teal?style=flat-square&logo=linux" alt="Pop!_OS COSMIC"></a>
  <a href="https://wayland.freedesktop.org/"><img src="https://img.shields.io/badge/Display%20Server-Wayland-orange?style=flat-square" alt="Wayland"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Language-Rust%202021-red?style=flat-square&logo=rust" alt="Rust"></a>
</p>

---

## Overview

**Deskstamp** is a lightweight screen watermark overlay utility for **Pop!_OS 24.04 COSMIC** and modern Wayland compositors.

https://github.com/user-attachments/assets/35e61840-b376-4e02-9028-4cd39a365f09

It runs as a persistent transparent layer over your workspace, protecting streams, recordings, and confidential work without post-production video editing:

- **True Wayland click-through:** Uses `zwlr_layer_shell_v1` with an empty input region (`wl_surface.set_input_region(None)`). All clicks, mouse gestures, and keyboard focus pass straight to applications beneath.
- **Zero idle CPU overhead:** Renders once to a shared memory (`shm`) buffer and sleeps. Only redraws on configuration updates or dynamic clock tokens.
- **Dynamic tokens:** Expands `{user}`, `{hostname}`, `{date}`, and customizable `{time:%H:%M:%S}` in real-time.
- **IPC control:** UNIX domain socket allows toggling visibility, reloading settings, and inspecting status on the fly.

---

## Installation

### Pop!_OS / Ubuntu / Debian (`.deb`)
Download the `.deb` package from [Releases](https://github.com/GabrielBaiano/Deskstamp/releases):
```bash
wget https://github.com/GabrielBaiano/Deskstamp/releases/latest/download/deskstamp_0.1.0_amd64.deb
sudo apt install ./deskstamp_0.1.0_amd64.deb
```

### Generic Linux x86_64 (`.tar.gz`)
```bash
wget https://github.com/GabrielBaiano/Deskstamp/releases/latest/download/deskstamp-v0.1.0-x86_64-unknown-linux-gnu.tar.gz
tar -xzf deskstamp-v0.1.0-x86_64-unknown-linux-gnu.tar.gz
cd deskstamp-v0.1.0-x86_64-unknown-linux-gnu
./install.sh
```

### Build from Source
```bash
git clone https://github.com/GabrielBaiano/Deskstamp.git
cd Deskstamp
cargo build --release
install -Dm755 target/release/deskstamp ~/.local/bin/deskstamp
```

---

## Usage

### Test Overlay (5 seconds)
```bash
deskstamp test
```

### Run Daemon
```bash
deskstamp daemon &
```

### Toggle Visibility
Bind this command to a keyboard shortcut in COSMIC Desktop Settings (`Settings -> Keyboard -> Shortcuts`):
```bash
deskstamp toggle
```

### Reload Configuration
```bash
deskstamp reload
```

### Export Watermark Preview Image
```bash
deskstamp preview preview.png
```

---

## Configuration

Deskstamp creates a default configuration file at `~/.config/deskstamp/config.json`:

```json
{
  "text": "CONFIDENTIAL • {user}@{hostname} • {time:%H:%M:%S}",
  "font_size": 22.0,
  "angle_deg": -25.0,
  "opacity": 0.18,
  "color_rgba": [255, 255, 255, 255],
  "spacing_x": 420.0,
  "spacing_y": 220.0,
  "stagger_offset": 210.0,
  "stroke_width": 1.0,
  "stroke_color_rgba": [0, 0, 0, 180],
  "font_path": null,
  "active": true,
  "update_interval_secs": 1
}
```

### Dynamic Tokens

| Token | Output Description | Example |
| :--- | :--- | :--- |
| `{user}` | Current logged-in user | `gabriel` |
| `{hostname}` | Machine system hostname | `pop-os` |
| `{date}` | Current date (`YYYY-MM-DD`) | `2026-09-22` |
| `{time}` | Current time (`HH:MM:SS`) | `14:30:15` |
| `{date:FORMAT}` | Custom Chrono date format | `{date:%b %d, %Y}` → `Sep 22, 2026` |
| `{time:FORMAT}` | Custom Chrono time format | `{time:%I:%M %p}` → `02:30 PM` |

---

## License

Distributed under the **GPL-3.0-or-later** License. See `LICENSE` for details.
