use crate::config::Config;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use async_channel::Sender;
use tracing::{info, warn};

pub fn socket_path() -> PathBuf {
    Config::cache_dir().join("pet.sock")
}

pub fn pid_file_path() -> PathBuf {
    Config::cache_dir().join("pet.pid")
}

pub fn record_pet_pid() {
    let path = pid_file_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let pid = std::process::id();
    let _ = std::fs::write(&path, pid.to_string());
}

pub fn remove_pet_pid() {
    let _ = std::fs::remove_file(pid_file_path());
}

pub fn read_pet_pid() -> Option<u32> {
    let content = std::fs::read_to_string(pid_file_path()).ok()?;
    content.trim().parse::<u32>().ok()
}

pub fn is_desktop_pet_running() -> bool {
    if let Some(pid) = read_pet_pid() {
        let proc_stat = PathBuf::from(format!("/proc/{}/stat", pid));
        if proc_stat.exists() {
            return true;
        }
        // Stale PID file; clean up
        remove_pet_pid();
        let _ = std::fs::remove_file(socket_path());
    }
    false
}

pub fn try_send_ipc_command(cmd: &str) -> bool {
    let path = socket_path();
    if !path.exists() {
        return false;
    }

    match UnixStream::connect(&path) {
        Ok(mut stream) => {
            let _ = stream.set_write_timeout(Some(Duration::from_millis(800)));
            let _ = stream.set_read_timeout(Some(Duration::from_millis(800)));
            if writeln!(stream, "{}", cmd).is_err() || stream.flush().is_err() {
                return false;
            }
            let mut reader = BufReader::new(&stream);
            let mut response = String::new();
            if reader.read_line(&mut response).is_ok() && !response.trim().is_empty() {
                return true;
            }
            false
        }
        Err(_) => false,
    }
}

pub fn send_pet_command(cmd: &str) -> bool {
    for _ in 0..3 {
        if try_send_ipc_command(cmd) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    false
}

pub fn start_ipc_server(tx: Sender<String>) {
    let path = socket_path();
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(Config::cache_dir().join("kittynotify.sock"));

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let listener = match UnixListener::bind(&path) {
        Ok(l) => l,
        Err(e) => {
            warn!("Failed to bind IPC socket {:?}: {}", path, e);
            return;
        }
    };

    let running = Arc::new(AtomicBool::new(true));
    let r_clone = running.clone();
    let path_clone = path.clone();

    std::thread::spawn(move || {
        info!("KittyNotify IPC daemon listening on {:?}", path_clone);
        for stream in listener.incoming() {
            if !r_clone.load(Ordering::Relaxed) {
                break;
            }
            if let Ok(mut s) = stream {
                let _ = s.set_read_timeout(Some(Duration::from_millis(1000)));
                let mut reader = BufReader::new(&s);
                let mut line = String::new();
                if reader.read_line(&mut line).is_ok() {
                    let cmd = line.trim().to_string();
                    let _ = writeln!(s, "ok");
                    let _ = s.flush();
                    let _ = tx.send_blocking(cmd);
                    glib::MainContext::default().wakeup();
                }
            }
        }
        let _ = std::fs::remove_file(&path_clone);
    });
}

pub fn stop_desktop_pet() -> bool {
    let mut stopped = false;

    // 1. Try sending IPC "quit" command to let it exit gracefully
    if try_send_ipc_command("quit") {
        stopped = true;
        std::thread::sleep(Duration::from_millis(100));
    }

    // 2. Fallback: check PID file and terminate process if still running
    if let Some(pid) = read_pet_pid() {
        if PathBuf::from(format!("/proc/{}", pid)).exists() {
            let _ = std::process::Command::new("kill")
                .arg(pid.to_string())
                .status();
            stopped = true;
        }
    }

    // 3. Clean up PID and socket files
    remove_pet_pid();
    let _ = std::fs::remove_file(socket_path());
    let _ = std::fs::remove_file(Config::cache_dir().join("kittynotify.sock"));

    stopped
}
