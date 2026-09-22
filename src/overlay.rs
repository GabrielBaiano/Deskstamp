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
};
use wayland_client::{
    globals::GlobalList,
    protocol::{wl_output, wl_shm, wl_surface},
    Connection, QueueHandle,
};

pub struct OverlayOutput {
    pub output: wl_output::WlOutput,
    pub layer_surface: LayerSurface,
    pub width: u32,
    pub height: u32,
    pub configured: bool,
}

pub struct CosmarkApp {
    pub registry_state: RegistryState,
    pub output_state: OutputState,
    pub compositor_state: CompositorState,
    pub shm: Shm,
    pub layer_shell: LayerShell,
    pub pool: SlotPool,
    pub outputs: Vec<OverlayOutput>,
    pub renderer: WatermarkRenderer,
    pub config: WatermarkConfig,
    pub exit: bool,
    pub single_frame_test: bool,
    pub frame_counter: u32,
}

impl CosmarkApp {
    pub fn new(globals: &GlobalList, qh: &QueueHandle<Self>) -> Result<Self, Box<dyn std::error::Error>> {
        let registry_state = RegistryState::new(globals);
        let output_state = OutputState::new(globals, qh);
        let compositor_state = CompositorState::bind(globals, qh)?;
        let shm = Shm::bind(globals, qh)?;
        let layer_shell = LayerShell::bind(globals, qh)?;
        let pool = SlotPool::new(1920 * 1080 * 4, &shm)?;
        let renderer = WatermarkRenderer::new(None)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e))?;
        let config = WatermarkConfig::load();

        let mut app = Self {
            registry_state,
            output_state,
            compositor_state,
            shm,
            layer_shell,
            pool,
            outputs: Vec::new(),
            renderer,
            config,
            exit: false,
            single_frame_test: false,
            frame_counter: 0,
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
        });
    }

    pub fn draw(&mut self, _qh: &QueueHandle<Self>) {
        for item in &mut self.outputs {
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

            self.renderer.render_to_buffer(canvas, width, height, stride, &self.config);

            let wl_surf = item.layer_surface.wl_surface();
            buffer.attach_to(wl_surf).expect("attach buffer");
            wl_surf.damage_buffer(0, 0, width as i32, height as i32);
            item.layer_surface.commit();
        }

        self.frame_counter += 1;
    }
}

impl ProvidesRegistryState for CosmarkApp {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState];
}

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

delegate_compositor!(CosmarkApp);
delegate_output!(CosmarkApp);
delegate_shm!(CosmarkApp);
delegate_layer!(CosmarkApp);
delegate_registry!(CosmarkApp);
