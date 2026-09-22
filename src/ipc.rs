use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum IpcCommand {
    Toggle,
    Reload,
    Status,
    Quit,
    SetOpacity { value: f32 },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IpcResponse {
    pub success: bool,
    pub message: String,
    pub active: Option<bool>,
}

pub fn get_socket_path() -> PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(runtime_dir).join("deskstamp.sock")
}

pub fn send_command(cmd: &IpcCommand) -> Result<IpcResponse, Box<dyn std::error::Error>> {
    let path = get_socket_path();
    let mut stream = UnixStream::connect(path)?;
    let msg = serde_json::to_string(cmd)? + "\n";
    stream.write_all(msg.as_bytes())?;

    let mut response_str = String::new();
    stream.read_to_string(&mut response_str)?;
    let res: IpcResponse = serde_json::from_str(&response_str)?;
    Ok(res)
}

pub fn bind_listener() -> Result<UnixListener, std::io::Error> {
    let path = get_socket_path();
    if path.exists() {
        let _ = std::fs::remove_file(&path);
    }
    UnixListener::bind(path)
}
