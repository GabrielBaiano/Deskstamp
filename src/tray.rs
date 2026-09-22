use ksni::blocking::TrayMethods;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct DeskstampTray {
    pub active: Arc<AtomicBool>,
}

impl ksni::Tray for DeskstampTray {
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

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        vec![
            StandardItem {
                label: if self.active.load(Ordering::Relaxed) {
                    "Disable Watermark".to_string()
                } else {
                    "Enable Watermark".to_string()
                },
                activate: Box::new(|this: &mut Self| {
                    let current = this.active.load(Ordering::Relaxed);
                    this.active.store(!current, Ordering::Relaxed);
                    let _ = crate::ipc::send_command(&crate::ipc::IpcCommand::Toggle);
                }),
                ..Default::default()
            }.into(),
            MenuItem::Separator,
            StandardItem {
                label: "Open Settings".into(),
                activate: Box::new(|_| {
                    if let Ok(exe) = std::env::current_exe() {
                        let _ = std::process::Command::new(exe).arg("gui").spawn();
                    }
                }),
                ..Default::default()
            }.into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit Deskstamp".into(),
                activate: Box::new(|_| {
                    let _ = crate::ipc::send_command(&crate::ipc::IpcCommand::Quit);
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
