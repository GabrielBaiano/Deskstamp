use crate::config::WatermarkConfig;
use crate::renderer::WatermarkRenderer;
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState, Region},
    delegate_compositor, delegate_layer, delegate_output, delegate_registry, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    shell::{
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        },
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
    seat::{
        SeatHandler, SeatState, Capability,
        pointer::{PointerHandler, PointerEvent, PointerEventKind},
    },
    delegate_seat, delegate_pointer,
};
use wayland_client::{
    globals::GlobalList,
    protocol::{wl_output, wl_shm, wl_surface},
    Connection, QueueHandle, Dispatch,
};
use wayland_protocols::ext::workspace::v1::client::{
    ext_workspace_manager_v1::{self, ExtWorkspaceManagerV1},
    ext_workspace_group_handle_v1::{self, ExtWorkspaceGroupHandleV1},
    ext_workspace_handle_v1::{self, ExtWorkspaceHandleV1},
};


use wayland_client::Proxy;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct WorkspaceData {
    pub name: String,
    pub is_active: bool,
}

pub struct OverlayOutput {
    pub output: wl_output::WlOutput,
    pub layer_surface: LayerSurface,
    pub width: u32,
    pub height: u32,
    pub configured: bool,
    pub last_damage_rect: Option<[i32; 4]>,
}

pub struct CosmarkApp {
    pub registry_state: RegistryState,
    pub output_state: OutputState,
    pub compositor_state: CompositorState,
    pub shm: Shm,
    pub layer_shell: LayerShell,
    pub seat_state: SeatState,
    pub pointers: Vec<wayland_client::protocol::wl_pointer::WlPointer>,
    pub pool: SlotPool,
    pub outputs: Vec<OverlayOutput>,
    pub renderer: WatermarkRenderer,
    pub config: WatermarkConfig,
    pub exit: bool,
    pub single_frame_test: bool,
    pub frame_counter: u32,
    pub workspace_manager: Option<ExtWorkspaceManagerV1>,
    pub workspaces: HashMap<wayland_client::backend::ObjectId, WorkspaceData>,
    pub active_workspace: Option<String>,
    pub active_corner_position: Option<String>,
}

impl CosmarkApp {
    pub fn new(globals: &GlobalList, qh: &QueueHandle<Self>) -> Result<Self, Box<dyn std::error::Error>> {
        let registry_state = RegistryState::new(globals);
        let output_state = OutputState::new(globals, qh);
        let compositor_state = CompositorState::bind(globals, qh)?;
        let shm = Shm::bind(globals, qh)?;
        let layer_shell = LayerShell::bind(globals, qh)?;
        let pool = SlotPool::new(1920 * 1080 * 4, &shm)?;
        let config = WatermarkConfig::load();
        let renderer = WatermarkRenderer::new_with_family(config.font_path.as_deref(), &config.font_family)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e))?;
        let mut seat_state = SeatState::new(globals, qh);
        let mut pointers = Vec::new();
        for seat in seat_state.seats() {
            if let Ok(ptr) = seat_state.get_pointer(qh, &seat) {
                pointers.push(ptr);
            }
        }
        let workspace_manager: Option<ExtWorkspaceManagerV1> = globals.bind(qh, 1..=1, ()).ok();

        let mut app = Self {
            registry_state,
            output_state,
            compositor_state,
            shm,
            layer_shell,
            seat_state,
            pointers,
            pool,
            outputs: Vec::new(),
            renderer,
            config,
            exit: false,
            single_frame_test: false,
            frame_counter: 0,
            workspace_manager,
            workspaces: HashMap::new(),
            active_workspace: None,
            active_corner_position: None,
        };

        // Create surfaces for already discovered outputs
        let outputs: Vec<_> = app.output_state.outputs().collect();
        for output in outputs {
            app.setup_surface_for_output(qh, output);
        }

        Ok(app)
    }

    pub fn setup_surface_for_output(&mut self, qh: &QueueHandle<Self>, output: wl_output::WlOutput) {
        let surface = self.compositor_state.create_surface(qh);

        // Click-Through: Create empty region so all mouse clicks and touches pass through to underlying windows
        if let Ok(region) = Region::new(&self.compositor_state) {
            surface.set_input_region(Some(region.wl_region()));
        }

        let layer_surface = self.layer_shell.create_layer_surface(
            qh,
            surface,
            Layer::Overlay,
            Some("deskstamp_overlay"),
            Some(&output),
        );

        // Fill entire screen across all 4 anchors
        layer_surface.set_anchor(Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT);
        layer_surface.set_exclusive_zone(-1);
        layer_surface.set_keyboard_interactivity(KeyboardInteractivity::None);
        layer_surface.commit();

        self.outputs.push(OverlayOutput {
            output,
            layer_surface,
            width: 0,
            height: 0,
            configured: false,
            last_damage_rect: None,
        });
    }

    pub fn trigger_mouse_dodge(&mut self, qh: &QueueHandle<Self>) {
        let current_pos = self.active_corner_position.as_deref().unwrap_or(&self.config.corner_position);
        let next_pos = match current_pos {
            "top_left" => "top_right",
            "top_right" => "top_left",
            "bottom_left" => "bottom_right",
            "bottom_right" => "bottom_left",
            _ => "top_right",
        };
        self.active_corner_position = Some(next_pos.to_string());
        self.draw(qh);
    }

    pub fn draw(&mut self, _qh: &QueueHandle<Self>) {
        let active_ws = self.active_workspace.as_deref();

        // Effective config with corner position override if dodged
        let mut effective_cfg = self.config.clone();
        if effective_cfg.is_corner_mode() && effective_cfg.corner_mouse_dodge {
            if let Some(pos) = &self.active_corner_position {
                effective_cfg.corner_position = pos.clone();
            }
        } else {
            self.active_corner_position = None;
        }

        for (idx, item) in self.outputs.iter_mut().enumerate() {
            if !item.configured || item.width == 0 || item.height == 0 {
                continue;
            }

            let width = item.width;
            let height = item.height;
            let stride = width * 4;

            let (buffer, canvas) = match self.pool.create_buffer(
                width as i32,
                height as i32,
                stride as i32,
                wl_shm::Format::Argb8888,
            ) {
                Ok(res) => res,
                Err(err) => {
                    eprintln!("Failed to allocate shm buffer: {:?}", err);
                    continue;
                }
            };

            let screen_idx = idx + 1;
            let screen_name = format!("Display {}", screen_idx);

            let dirty_rect = self.renderer.render_to_buffer(
                canvas,
                width,
                height,
                stride,
                &effective_cfg,
                active_ws,
                screen_idx,
                Some(&screen_name),
            );

            // Convert tiny-skia's RGBA pixel buffer to Wayland wl_shm ARGB8888 (little-endian BGRA in memory)
            if effective_cfg.is_corner_mode() {
                if let Some([rx, ry, rw, rh]) = dirty_rect {
                    for y in ry..(ry + rh).min(height as i32) {
                        let row_start = (y as usize * stride as usize) + (rx as usize * 4);
                        let row_end = row_start + (rw as usize * 4);
                        if row_end <= canvas.len() {
                            for chunk in canvas[row_start..row_end].chunks_exact_mut(4) {
                                chunk.swap(0, 2);
                            }
                        }
                    }
                }
            } else {
                for chunk in canvas.chunks_exact_mut(4) {
                    chunk.swap(0, 2);
                }
            }

            let wl_surf = item.layer_surface.wl_surface();
            buffer.attach_to(wl_surf).expect("attach buffer");

            if effective_cfg.is_corner_mode() {
                if let Some(new_rect) = dirty_rect {
                    // 1. Damage previous rect if it moved, so compositor clears previous position
                    if let Some(old_rect) = item.last_damage_rect {
                        if old_rect != new_rect {
                            wl_surf.damage_buffer(old_rect[0], old_rect[1], old_rect[2], old_rect[3]);
                        }
                    }
                    // 2. Damage new bounding box only (never full screen)
                    wl_surf.damage_buffer(new_rect[0], new_rect[1], new_rect[2], new_rect[3]);
                    item.last_damage_rect = Some(new_rect);

                    // 3. Update input region for mouse dodge detection or pass-through
                    if effective_cfg.corner_mouse_dodge {
                        if let Ok(region) = Region::new(&self.compositor_state) {
                            let pad = 12;
                            region.add(
                                (new_rect[0] - pad).max(0),
                                (new_rect[1] - pad).max(0),
                                new_rect[2] + pad * 2,
                                new_rect[3] + pad * 2,
                            );
                            wl_surf.set_input_region(Some(region.wl_region()));
                        }
                    } else {
                        if let Ok(region) = Region::new(&self.compositor_state) {
                            wl_surf.set_input_region(Some(region.wl_region()));
                        }
                    }
                } else {
                    if let Some(old_rect) = item.last_damage_rect.take() {
                        wl_surf.damage_buffer(old_rect[0], old_rect[1], old_rect[2], old_rect[3]);
                    }
                    if let Ok(region) = Region::new(&self.compositor_state) {
                        wl_surf.set_input_region(Some(region.wl_region()));
                    }
                }
            } else {
                // Grid mode covers the entire screen
                wl_surf.damage_buffer(0, 0, width as i32, height as i32);
                item.last_damage_rect = Some([0, 0, width as i32, height as i32]);
                if let Ok(region) = Region::new(&self.compositor_state) {
                    wl_surf.set_input_region(Some(region.wl_region()));
                }
            }

            item.layer_surface.commit();
        }

        self.frame_counter += 1;
    }
}

impl Dispatch<ExtWorkspaceManagerV1, ()> for CosmarkApp {
    fn event(
        state: &mut Self,
        _proxy: &ExtWorkspaceManagerV1,
        event: ext_workspace_manager_v1::Event,
        _data: &(),
        _conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        match event {
            ext_workspace_manager_v1::Event::WorkspaceGroup { .. } => {}
            ext_workspace_manager_v1::Event::Workspace { .. } => {}
            ext_workspace_manager_v1::Event::Done => {
                let new_active = state
                    .workspaces
                    .values()
                    .find(|w| w.is_active)
                    .map(|w| w.name.clone());

                if new_active.is_some() && new_active != state.active_workspace {
                    state.active_workspace = new_active;
                    state.draw(qhandle);
                }
            }
            ext_workspace_manager_v1::Event::Finished => {}
            _ => {}
        }
    }

    wayland_client::event_created_child!(CosmarkApp, ExtWorkspaceManagerV1, [
        0 => (ExtWorkspaceGroupHandleV1, ()),
        1 => (ExtWorkspaceHandleV1, ()),
    ]);
}

impl Dispatch<ExtWorkspaceGroupHandleV1, ()> for CosmarkApp {
    fn event(
        _state: &mut Self,
        _proxy: &ExtWorkspaceGroupHandleV1,
        _event: ext_workspace_group_handle_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ExtWorkspaceHandleV1, ()> for CosmarkApp {
    fn event(
        state: &mut Self,
        proxy: &ExtWorkspaceHandleV1,
        event: ext_workspace_handle_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        let id = proxy.id();
        let ws = state.workspaces.entry(id).or_default();
        match event {
            ext_workspace_handle_v1::Event::Name { name } => {
                ws.name = name;
            }
            ext_workspace_handle_v1::Event::State { state: wayland_client::WEnum::Value(flags) } => {
                ws.is_active = flags.contains(ext_workspace_handle_v1::State::Active);
            }
            ext_workspace_handle_v1::Event::Removed => {
                state.workspaces.remove(&proxy.id());
            }
            _ => {}
        }
    }
}

pub fn detect_fallback_workspace() -> Option<String> {
    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        if let Ok(out) = std::process::Command::new("hyprctl").args(["activeworkspace", "-j"]).output() {
            if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
                if let Some(name) = val.get("name").and_then(|v| v.as_str()) {
                    return Some(name.to_string());
                }
                if let Some(id) = val.get("id").and_then(|v| v.as_i64()) {
                    return Some(id.to_string());
                }
            }
        }
    }
    if std::env::var("SWAYSOCK").is_ok() {
        if let Ok(out) = std::process::Command::new("swaymsg").args(["-t", "get_workspaces"]).output() {
            if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
                if let Some(arr) = val.as_array() {
                    for item in arr {
                        if item.get("focused").and_then(|v| v.as_bool()).unwrap_or(false) {
                            if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                                return Some(name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

impl ProvidesRegistryState for CosmarkApp {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}

impl SeatHandler for CosmarkApp {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: wayland_client::protocol::wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wayland_client::protocol::wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Pointer {
            if let Ok(ptr) = self.seat_state.get_pointer(qh, &seat) {
                self.pointers.push(ptr);
            }
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _seat: wayland_client::protocol::wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Pointer {
            self.pointers.clear();
        }
    }

    fn remove_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: wayland_client::protocol::wl_seat::WlSeat) {}
}

impl PointerHandler for CosmarkApp {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _pointer: &wayland_client::protocol::wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        for event in events {
            match event.kind {
                PointerEventKind::Enter { .. } | PointerEventKind::Motion { .. } => {
                    if self.config.is_corner_mode() && self.config.corner_mouse_dodge {
                        self.trigger_mouse_dodge(qh);
                        break;
                    }
                }
                _ => {}
            }
        }
    }
}

delegate_compositor!(CosmarkApp);
delegate_output!(CosmarkApp);
delegate_shm!(CosmarkApp);
delegate_layer!(CosmarkApp);
delegate_seat!(CosmarkApp);
delegate_pointer!(CosmarkApp);
delegate_registry!(CosmarkApp);

impl OutputHandler for CosmarkApp {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(&mut self, _conn: &Connection, qh: &QueueHandle<Self>, output: wl_output::WlOutput) {
        self.setup_surface_for_output(qh, output);
    }

    fn update_output(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: wl_output::WlOutput) {}

    fn output_destroyed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, output: wl_output::WlOutput) {
        self.outputs.retain(|o| o.output != output);
    }
}

impl CompositorHandler for CosmarkApp {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _scale: i32,
    ) {}

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _transform: wl_output::Transform,
    ) {}

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {}

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {}

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {}
}

impl ShmHandler for CosmarkApp {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl LayerShellHandler for CosmarkApp {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, layer: &LayerSurface) {
        self.outputs.retain(|o| o.layer_surface.wl_surface() != layer.wl_surface());
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        if let Some(item) = self.outputs.iter_mut().find(|o| o.layer_surface.wl_surface() == layer.wl_surface()) {
            item.width = configure.new_size.0;
            item.height = configure.new_size.1;
            item.configured = true;
        }
        self.draw(qh);
    }
}
