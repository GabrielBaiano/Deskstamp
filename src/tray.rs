use crate::config::WatermarkConfig;
use crate::ipc::{send_command, IpcCommand};
use ksni::blocking::TrayMethods;
use ksni::menu::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct DeskstampTray {
    pub active: Arc<AtomicBool>,
}

impl ksni::Tray for DeskstampTray {
    const MENU_ON_ACTIVATE: bool = true;

    fn id(&self) -> String {
        "io.github.gabrielbaiano.Deskstamp".to_string()
    }

    fn title(&self) -> String {
        "Deskstamp".to_string()
    }

    fn icon_name(&self) -> String {
        "io.github.gabrielbaiano.Deskstamp".to_string()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        static ICON_PNG: &[u8] = include_bytes!("../data/icons/hicolor/64x64/apps/io.github.gabrielbaiano.Deskstamp.png");
        if let Ok(pix) = tiny_skia::Pixmap::decode_png(ICON_PNG) {
            let width = pix.width() as i32;
            let height = pix.height() as i32;
            let mut data = pix.data().to_vec();
            for pixel in data.chunks_exact_mut(4) {
                pixel.rotate_right(1);
            }
            vec![ksni::Icon { width, height, data }]
        } else {
            vec![]
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        let current = self.active.load(Ordering::Relaxed);
        let new_state = !current;
        self.active.store(new_state, Ordering::Relaxed);
        let mut cfg = WatermarkConfig::load();
        cfg.active = new_state;
        let _ = cfg.save();
        let _ = send_command(&IpcCommand::Toggle);
    }

    fn secondary_activate(&mut self, _x: i32, _y: i32) {
        self.activate(_x, _y);
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        let is_active = self.active.load(Ordering::Relaxed);

        vec![
            // 1. Watermark Toggle
            CheckmarkItem {
                label: if is_active {
                    "✓ Watermark Active".into()
                } else {
                    "Watermark Inactive".into()
                },
                checked: is_active,
                activate: Box::new(|this: &mut Self| {
                    let current = this.active.load(Ordering::Relaxed);
                    let new_state = !current;
                    this.active.store(new_state, Ordering::Relaxed);
                    let mut cfg = WatermarkConfig::load();
                    cfg.active = new_state;
                    let _ = cfg.save();
                    let _ = send_command(&IpcCommand::Toggle);
                }),
                ..Default::default()
            }.into(),

            MenuItem::Separator,

            // 2. Settings Window
            StandardItem {
                label: "Settings...".into(),
                activate: Box::new(|_| {
                    if let Ok(exe) = std::env::current_exe() {
                        let _ = std::process::Command::new(exe).arg("settings").spawn();
                    }
                }),
                ..Default::default()
            }.into(),


            MenuItem::Separator,

            // 4. Quit
            StandardItem {
                label: "Quit Deskstamp".into(),
                activate: Box::new(|_| {
                    let _ = send_command(&IpcCommand::Quit);
                    std::process::exit(0);
                }),
                ..Default::default()
            }.into(),
        ]
    }
}

pub fn spawn_tray(active: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        let tray = DeskstampTray { active };
        if let Ok(_handle) = tray.spawn() {
            loop {
                std::thread::sleep(std::time::Duration::from_secs(3600));
            }
        }
    });
}
