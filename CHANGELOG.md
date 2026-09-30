# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog and follows Conventional Commits.

## [0.2.0] - 2026-09-30

### Feat
- Add per-workspace custom stamps and dynamic `{workspace}` / `{workspace_num}` tokens (requested by @johnblommers in #1).
- Add discreet corner watermark HUD with customizable position (`top_left`, `top_right`, `bottom_left`, `bottom_right`) and margins.
- Add macOS Sonoma-inspired glass pill container (`corner_style = "capsule"`) with dynamic font scaling, border, and background opacity controls.
- Add screen/display badges (`{screen}`, `{display}`, `{screen_name}`) for multi-monitor setups.
- Add smart cursor dodge (`corner_mouse_dodge`): watermark automatically evades to an alternate corner upon mouse hover via Wayland pointer frames and input region tracking.
- Add single-line dynamic horizontal connectors between text repetitions in grid mode.
- Add font family selector and custom typography support in Settings Studio.
- Add reset buttons in Settings Studio (quick header reset and full factory default restore).
- Add input debouncing (40ms) in Settings Studio to prevent compositor event queue exhaustion during slider manipulation.

### Fix
- Eliminate desktop background flickering under Wayland compositors (COSMIC/Mutter) via sub-surface damage tracking (`wl_surface.damage_buffer`) instead of damaging the entire screen.
- Optimize ARGB byte-swapping memory operations by restricting row transforms strictly to dirty bounding-box scanlines (~40 KB instead of 8.3 MB per frame).
- Fix rendering failure when running without an icon or image (`show_icon = false` or missing file path).
- Fix missing application icon in Wayland task switchers and docks by binding `io.github.gabrielbaiano.Deskstamp` app_id in ViewportBuilder.

### Docs
- Document damage tracking and Wayland compositor compatibility in `TODO.md`.
- Track upcoming v0.3.0 roadmap focusing on visual bug fixes and settings menu layout improvements.

### Chore / Test
- Add unit test coverage for corner capsule rendering, token contexts, font family resolution, and image-less watermark pipelines.
- Bump crate version to `0.2.0`.

---

## [0.1.0] - 2026-09-23

### Feat
- Native Wayland screen watermark overlay utility via `zwlr_layer_shell_v1`.
- Input pass-through using empty Wayland input regions.
- Dynamic tokens for `{user}`, `{hostname}`, `{date}`, and `{time}`.
- Settings GUI built with `egui` for interactive configuration.
- StatusNotifier tray indicator with desktop notifications.
- Monochrome and custom PNG/JPEG logo watermark support.

### Chore / Test
- Initial release packaging with `.deb` and standalone `.tar.gz` distribution scripts.
