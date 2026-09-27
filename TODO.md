# Deskstamp - Bug Tracker & Planned Improvements

Tracking issues discovered during local testing on Wayland / Pop!_OS COSMIC.

---

### [BUG-01] Desktop Background Flickering / Compositor Effects Incompatibility
- **Symptom**: When system desktop animations or effects trigger (e.g. workspace switching, window overview, blur/transparency passes), the desktop background/overlay flickers.
- **Root Cause & Technical Analysis**:
  - `deskstamp` uses `wlr_layer_shell_v1` on `Layer::Overlay` anchored to all 4 edges with a full-screen ARGB surface.
  - Periodic redraws (e.g., 1-second clock timer) allocate a fresh slot from `SlotPool` and damage the entire display rectangle (`0, 0, width, height`), causing the compositor to re-composite full-screen transparency passes continuously.
  - Additionally, `Layer::Overlay` sits above all windows, which conflicts with desktop shell overview zoom/blur animations.
- **Planned Fix**:
  - Implement damage tracking so only altered regions (or static bounding boxes in corner mode) are damaged instead of the entire 4K/1080p display buffer.
  - Skip surface redrawing if watermark text does not have dynamic tokens like `{time}` and config hasn't changed.
  - Provide an option to select `Layer::Bottom` / `Layer::Background` for wallpaper-style stamping without interfering with top-level window animations.

---

### [BUG-02] Daemon Shutdown / Process Lifecycle vs Settings GUI
- **Symptom**: Stopping the watermark app or closing it can leave the daemon terminated while the configuration window stays open, or closing settings can kill the daemon unintentionally.
- **Root Cause & Technical Analysis**:
  - In `src/gui.rs`, the settings window attempts to spawn `deskstamp daemon` via `std::process::Command::spawn()` if not detected. If the user stops the service via CLI (`deskstamp stop`), the GUI still runs independently. Conversely, the settings toggle for "Enable Desktop Watermark" toggles `config.active`, but doesn't handle proper background daemon lifecycle management.
- **Planned Fix**:
  - Cleanly decouple daemon lifecycle from GUI:
    - Add an explicit daemon status pill in the GUI header (`Running` / `Stopped`) with dedicated Start/Stop actions.
    - Graceful handling when daemon exits while settings window is open without freezing IPC calls.
    - Ensure closing GUI window never terminates a running background daemon.

---

### [BUG-03] Missing App Icon in Window Titlebar & Task Switcher
- **Symptom**: The Settings window (`deskstamp settings`) displays a generic Wayland window icon in the dock/app switcher instead of the official Deskstamp brand icon.
- **Root Cause & Technical Analysis**:
  - Wayland compositors (including `cosmic-comp`, `mutter`, and `sway`) bind window icons via the Wayland `app_id` matching a desktop entry file ID (`io.github.gabrielbaiano.Deskstamp.desktop`).
  - `eframe::NativeOptions` in `src/gui.rs` specifies `with_title("Deskstamp Settings")` and `with_icon()`, but does not set `with_app_id("io.github.gabrielbaiano.Deskstamp")`. Under Wayland, client-provided window icons via memory buffers are often ignored in favor of the compositor looking up `app_id` in `/usr/share/applications` or `~/.local/share/applications`.
- **Planned Fix**:
  - Configure `with_app_id("io.github.gabrielbaiano.Deskstamp")` on `egui::ViewportBuilder`.
  - Ensure the `.desktop` file is installed to `~/.local/share/applications/` and the hicolor icons to `~/.local/share/icons/` so the compositor resolves the icon on any desktop environment.
