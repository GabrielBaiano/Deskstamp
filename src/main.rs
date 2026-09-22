use deskstamp::config::WatermarkConfig;
use deskstamp::ipc::{bind_listener, get_socket_path, send_command, IpcCommand, IpcResponse};
use deskstamp::overlay::CosmarkApp;
use deskstamp::renderer::WatermarkRenderer;
use deskstamp::tray::spawn_tray;
use smithay_client_toolkit::reexports::calloop::{
    channel::{channel, Channel, Sender},
    timer::{TimeoutAction, Timer},
    EventLoop,
};
use smithay_client_toolkit::reexports::calloop_wayland_source::WaylandSource;
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tiny_skia::{Color, Pixmap, PixmapMut, PixmapPaint, Transform};
use wayland_client::{globals::registry_queue_init, Connection};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("menu");

    match command {
        "menu" | "quick" => {
            deskstamp::gui::run_quick_menu()?;
        }
        "settings" | "studio" | "gui" => {
            deskstamp::gui::run_gui()?;
        }
        "toggle" => {
            let res = send_command(&IpcCommand::Toggle)?;
            println!("{}", res.message);
        }
        "reload" => {
            let res = send_command(&IpcCommand::Reload)?;
            println!("{}", res.message);
        }
        "status" => match send_command(&IpcCommand::Status) {
            Ok(res) => println!("{}", res.message),
            Err(_) => println!("Deskstamp daemon is not currently running."),
        },
        "preview" => {
            let output_file = args.get(2).map(|s| s.as_str()).unwrap_or("deskstamp_preview.png");
            let cfg = WatermarkConfig::load();
            let renderer = WatermarkRenderer::new(cfg.font_path.as_deref())
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e))?;

            let width = 1920;
            let height = 1080;
            let mut pixmap = Pixmap::new(width, height).unwrap();
            // Dark wallpaper tone for realistic preview
            pixmap.fill(Color::from_rgba8(26, 29, 36, 255));

            let mut temp_buf = vec![0u8; (width * height * 4) as usize];
            renderer.render_to_buffer(&mut temp_buf, width, height, width * 4, &cfg);

            let temp_pixmap = PixmapMut::from_bytes(&mut temp_buf, width, height).unwrap();
            pixmap.draw_pixmap(0, 0, temp_pixmap.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
            pixmap.save_png(output_file)?;
            println!("Preview saved to: {}", output_file);
        }
        "config" => {
            let cfg = WatermarkConfig::load();
            println!("{}", serde_json::to_string_pretty(&cfg)?);
        }
        "test" | "daemon" => {
            let is_test = command == "test";
            run_overlay(is_test)?;
        }
        _ => {
            eprintln!("Usage: deskstamp [gui | daemon | test | toggle | reload | status | preview | config]");
        }
    }

    Ok(())
}

struct SocketGuard;
impl Drop for SocketGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(get_socket_path());
    }
}

fn run_overlay(is_test: bool) -> Result<(), Box<dyn std::error::Error>> {
    // If another daemon is already responding to IPC, do not spawn a duplicate overlay
    if !is_test && send_command(&IpcCommand::Status).is_ok() {
        println!("Deskstamp daemon is already active. Reloading config...");
        let _ = send_command(&IpcCommand::Reload);
        return Ok(());
    }

    let _socket_guard = SocketGuard;

    let conn = Connection::connect_to_env()?;
    let (globals, event_queue) = registry_queue_init(&conn)?;
    let qh = event_queue.handle();

    let mut event_loop: EventLoop<CosmarkApp> = EventLoop::try_new()?;
    let loop_handle = event_loop.handle();
    WaylandSource::new(conn.clone(), event_queue).insert(loop_handle.clone())?;

    let mut app = CosmarkApp::new(&globals, &qh)?;

    // Start System Tray in background
    let active_atomic = Arc::new(AtomicBool::new(app.config.active));
    if !is_test {
        spawn_tray(active_atomic.clone());
    }

    // Channel for IPC / Control commands
    let (tx, rx): (Sender<IpcCommand>, Channel<IpcCommand>) = channel();
    let qh_clone = qh.clone();
    let active_clone = active_atomic.clone();

    loop_handle.insert_source(rx, move |event, _, app: &mut CosmarkApp| {
        match event {
            smithay_client_toolkit::reexports::calloop::channel::Event::Msg(cmd) => match cmd {
                IpcCommand::Toggle => {
                    app.config.active = !app.config.active;
                    active_clone.store(app.config.active, Ordering::Relaxed);
                    let _ = app.config.save();
                    app.draw(&qh_clone);
                }
                IpcCommand::Reload => {
                    app.config = WatermarkConfig::load();
                    app.renderer.update_font(app.config.font_path.as_deref(), &app.config.font_family);
                    active_clone.store(app.config.active, Ordering::Relaxed);
                    app.draw(&qh_clone);
                }
                IpcCommand::SetOpacity { value } => {
                    app.config.opacity = value;
                    app.draw(&qh_clone);
                }
                IpcCommand::Quit => {
                    app.exit = true;
                }
                IpcCommand::Status => {}
            },
            smithay_client_toolkit::reexports::calloop::channel::Event::Closed => {}
        }
    })?;

    // Background thread to listen to UNIX socket for IPC
    let running = Arc::new(AtomicBool::new(true));
    let running_thread = running.clone();
    std::thread::spawn(move || {
        if let Ok(listener) = bind_listener() {
            listener.set_nonblocking(true).ok();
            while running_thread.load(Ordering::Relaxed) {
                if let Ok((mut stream, _)) = listener.accept() {
                    let mut buf = [0u8; 1024];
                    if let Ok(n) = stream.read(&mut buf) {
                        if n > 0 {
                            if let Ok(cmd) = serde_json::from_slice::<IpcCommand>(&buf[..n]) {
                                let _ = tx.send(cmd);
                                let res = IpcResponse {
                                    success: true,
                                    message: "Command sent to deskstamp daemon".to_string(),
                                    active: None,
                                };
                                let _ = serde_json::to_writer(&mut stream, &res);
                            }
                        }
                    }
                }
                std::thread::sleep(Duration::from_millis(40));
            }
        }
    });

    // Dynamic timer if clock tokens are present
    let qh_timer = qh.clone();
    let timer = Timer::from_duration(Duration::from_secs(1));

    loop_handle.insert_source(timer, move |_, _, app: &mut CosmarkApp| {
        let has_dynamic_time = app.config.text.contains("{time");
        if app.config.active && has_dynamic_time {
            app.draw(&qh_timer);
            TimeoutAction::ToDuration(Duration::from_secs(app.config.update_interval_secs.max(1)))
        } else {
            TimeoutAction::ToDuration(Duration::from_secs(2))
        }
    })?;

    // If running in test mode, set a 5-second shutdown timer
    if is_test {
        println!("Test mode activated: Running watermark overlay with click-through for 5 seconds...");
        let exit_timer = Timer::from_duration(Duration::from_secs(5));
        loop_handle.insert_source(exit_timer, |_, _, app: &mut CosmarkApp| {
            println!("Test completed. Exiting.");
            app.exit = true;
            TimeoutAction::Drop
        })?;
    } else {
        println!("Deskstamp daemon running. Socket at: {:?}", get_socket_path());
    }

    while !app.exit {
        event_loop.dispatch(Some(Duration::from_millis(80)), &mut app)?;
    }

    running.store(false, Ordering::Relaxed);
    let _ = std::fs::remove_file(get_socket_path());
    Ok(())
}
