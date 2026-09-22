# 🗺️ Deskstamp Roadmap

This roadmap outlines the planned development milestones for **Deskstamp**.

---

## 🏆 Core Objectives

- Provide the most lightweight, zero-latency desktop watermarking tool on Linux.
- Integrate deeply with Pop!_OS COSMIC and standard Wayland desktop protocols.
- Deliver seamless packaging for Flathub, Pop!_Shop, and distribution package managers.

---

## 📅 Milestones

### Phase 1: Core Engine & Wayland Layer-Shell (Completed)
- [x] Native `wlr-layer-shell-unstable-v1` overlay client.
- [x] Input pass-through via empty Wayland input region (`set_input_region(None)`).
- [x] 2D matrix transformations with rotation, spacing, and staggered tile layouts.
- [x] Dynamic tokens (`{user}`, `{hostname}`, `{date}`, `{time}`).
- [x] UNIX domain socket IPC interface for runtime `toggle` and `reload`.

### Phase 2: Vector Graphics & Effects (In Progress)
- [ ] SVG logo and image watermark rendering via `resvg`.
- [ ] Adaptive background luminance detection (invertible contrast).
- [ ] Micro-dot forensic steganographic mode for anti-leak tracking.

### Phase 3: Desktop Integration & GUI
- [ ] Dedicated `libcosmic` settings window with live interactive canvas.
- [ ] Native COSMIC panel applet for quick menu control.
- [ ] Global keyboard shortcut integration via XDG Desktop Portals.

### Phase 4: App Store Distribution
- [ ] Automated CI/CD GitHub Actions release workflow.
- [ ] Flathub manifest submission.
- [ ] Native `.deb` packages for Pop!_OS / Ubuntu.
