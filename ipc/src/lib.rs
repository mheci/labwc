use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::{atomic::AtomicBool, atomic::Ordering, Arc};
use std::thread;
use tracing::{error, info};

pub struct IpcServer {
    socket_path: PathBuf,
    running: Arc<AtomicBool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcCommand {
    Ping,
    ReloadConfig,
    Quit,
    FocusWindow(u64),
    CloseWindow(u64),
    MinimizeWindow(u64),
    MaximizeWindow(u64),
    FullscreenWindow(u64),
    SwitchWorkspace(String),
    ExecuteCommand(String),
    GetWindows,
    GetWorkspaces,
    GetOutputs,
    GetConfig,
    SetConfig { key: String, value: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcResponse {
    Ok,
    Error(String),
    Windows(Vec<WindowInfo>),
    Workspaces(Vec<String>),
    Outputs(Vec<OutputInfo>),
    Config(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub id: u64,
    pub title: String,
    pub app_id: String,
    pub workspace: String,
    pub geometry: (i32, i32, i32, i32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputInfo {
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub scale: f64,
    pub enabled: bool,
}

impl IpcServer {
    pub fn new(runtime_dir: &str) -> Self {
        let socket_path = PathBuf::from(runtime_dir).join("labwc-rs-ipc.sock");
        Self {
            socket_path,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)?;
        }

        let listener = UnixListener::bind(&self.socket_path)?;
        let running = Arc::clone(&self.running);
        running.store(true, Ordering::Relaxed);
        let path = self.socket_path.clone();

        thread::spawn(move || {
            info!("IPC server listening on {:?}", path);
            for stream in listener.incoming() {
                if !running.load(Ordering::Relaxed) {
                    break;
                }
                match stream {
                    Ok(stream) => Self::handle_client(stream),
                    Err(e) => error!("IPC accept error: {e}"),
                }
            }
        });
        Ok(())
    }

    fn handle_client(mut stream: UnixStream) {
        let mut reader = BufReader::new(stream.try_clone().unwrap_or_else(|_| unreachable!()));
        let mut line = String::new();
        if reader.read_line(&mut line).is_err() {
            return;
        }
        let response = match serde_json::from_str::<IpcCommand>(line.trim()) {
            Ok(IpcCommand::Ping) => IpcResponse::Ok,
            Ok(IpcCommand::GetWindows) => IpcResponse::Windows(Vec::new()),
            Ok(IpcCommand::GetWorkspaces) => {
                IpcResponse::Workspaces(vec!["1".into(), "2".into(), "3".into(), "4".into()])
            }
            Ok(IpcCommand::Quit) => {
                IpcResponse::Error("Quit requires authentication via LABWC_PID check".into())
            }
            Ok(IpcCommand::ExecuteCommand(_)) => IpcResponse::Error(
                "ExecuteCommand blocked: use spawn_async_no_shell via config".into(),
            ),
            Ok(_) => IpcResponse::Ok,
            Err(e) => IpcResponse::Error(format!("invalid command: {e}")),
        };
        let payload = serde_json::to_string(&response).unwrap_or_default();
        let _ = writeln!(stream, "{payload}");
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
        let _ = std::fs::remove_file(&self.socket_path);
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_command_serialization() {
        let cmd = IpcCommand::Ping;
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("Ping"));
    }

    #[test]
    fn test_ipc_response_serialization() {
        let resp = IpcResponse::Ok;
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("Ok"));
    }

    #[test]
    fn test_window_info() {
        let w = WindowInfo {
            id: 42,
            title: "test".into(),
            app_id: "test.app".into(),
            workspace: "1".into(),
            geometry: (0, 0, 800, 600),
        };
        let json = serde_json::to_string(&w).unwrap();
        let parsed: WindowInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, 42);
        assert_eq!(parsed.title, "test");
    }

    #[test]
    fn test_server_new_and_stop() {
        let server = IpcServer::new("/tmp");
        server.stop();
    }
}
