# Deskstamp - Bug Tracker & Planned Improvements

Tracking issues discovered during local testing on Wayland / Pop!_OS COSMIC.

---

### [BUG-01] Desktop Background Flickering / Compositor Effects Incompatibility (Resolved in v0.2.0)
- **Status**: Resolved
- **Solution**:
  - Implemented bounding-box damage tracking (`wl_surface.damage_buffer`) instead of damaging the entire screen on redraws.
  - Restricted little-endian ARGB byte swapping in SHM memory exclusively to dirty bounding box scanlines (~40 KB instead of 8.3 MB per frame).
  - Skipped redraw cycles when dynamic time tokens are absent and config is unchanged.

---

### [BUG-02] Daemon Shutdown / Process Lifecycle vs Settings GUI (Target: v0.3.0)
- **Status**: Planned for v0.3.0
- **Symptom**: Stopping the watermark app or closing it can leave the daemon terminated while the configuration window stays open, or closing settings can kill the daemon unintentionally.
- **Root Cause & Technical Analysis**:
  - In `src/gui.rs`, the settings window attempts to spawn `deskstamp daemon` via `std::process::Command::spawn()` if not detected. If the user stops the service via CLI (`deskstamp stop`), the GUI still runs independently.
- **Planned Fix**:
  - Cleanly decouple daemon lifecycle from GUI:
    - Add an explicit daemon status pill in the GUI header (`Running` / `Stopped`) with dedicated Start/Stop actions.
    - Graceful handling when daemon exits while settings window is open without freezing IPC calls.
    - Comprehensive UI menu layout enhancements and visual glitch fixes.

---

### [BUG-03] Missing App Icon in Window Titlebar & Task Switcher (Resolved in v0.2.0)
- **Status**: Resolved
- **Solution**:
  - Configured `.with_app_id("io.github.gabrielbaiano.Deskstamp")` on `egui::ViewportBuilder`.
  - Matched desktop entry `io.github.gabrielbaiano.Deskstamp.desktop` and hicolor icons for native Wayland compositor task switcher resolution.

