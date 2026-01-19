//! Debug server for Luna CLI connections.
//!
//! Listens on a Unix socket and processes commands/queries from CLI clients.
//! Commands are queued and processed on the main GPUI thread.

use crate::{execute_command_in_context, execute_query_in_context, Command, Query};
use canvas_2::Canvas;
use gpui::{Context, Entity};
use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

/// A pending request from a CLI client.
struct PendingRequest {
    json: String,
    response: Arc<(Mutex<Option<String>>, Condvar)>,
}

/// Debug server state shared between threads.
struct ServerState {
    pending: Mutex<VecDeque<PendingRequest>>,
    running: AtomicBool,
}

/// Debug server that accepts CLI connections.
pub struct DebugServer {
    socket_path: PathBuf,
    state: Arc<ServerState>,
}

impl DebugServer {
    /// Create a new debug server for the given process.
    pub fn new() -> Self {
        let pid = std::process::id();
        let socket_path = PathBuf::from(format!("/tmp/luna-{}.sock", pid));
        Self {
            socket_path,
            state: Arc::new(ServerState {
                pending: Mutex::new(VecDeque::new()),
                running: AtomicBool::new(false),
            }),
        }
    }

    /// Check if the debug server should be started based on environment.
    pub fn should_start() -> bool {
        std::env::var("LUNA_DEBUG").map(|v| v == "1" || v == "true").unwrap_or(false)
    }

    /// Get the socket path.
    pub fn socket_path(&self) -> &PathBuf {
        &self.socket_path
    }

    /// Start the debug server in a background thread.
    pub fn start(&self) {
        // Clean up any stale socket
        let _ = std::fs::remove_file(&self.socket_path);

        let listener = match UnixListener::bind(&self.socket_path) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Failed to start debug server: {}", e);
                return;
            }
        };

        self.state.running.store(true, Ordering::SeqCst);
        let state = self.state.clone();
        let socket_path = self.socket_path.clone();

        eprintln!("Debug server listening on {}", socket_path.display());

        thread::spawn(move || {
            // Set non-blocking so we can check the running flag
            listener.set_nonblocking(true).ok();

            while state.running.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let state = state.clone();
                        thread::spawn(move || {
                            if let Err(e) = handle_client(stream, state) {
                                eprintln!("Client error: {}", e);
                            }
                        });
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(std::time::Duration::from_millis(100));
                    }
                    Err(e) => {
                        eprintln!("Accept error: {}", e);
                    }
                }
            }

            let _ = std::fs::remove_file(&socket_path);
        });
    }

    /// Process any pending requests. Call this from the main thread.
    pub fn process_pending<T: 'static>(&self, canvas: &Entity<Canvas>, cx: &mut Context<T>) {
        loop {
            let request = {
                let mut pending = self.state.pending.lock().unwrap();
                pending.pop_front()
            };

            match request {
                Some(req) => {
                    let response = process_message(canvas, &req.json, cx);
                    let (lock, cvar) = &*req.response;
                    let mut result = lock.lock().unwrap();
                    *result = Some(response);
                    cvar.notify_one();
                }
                None => break,
            }
        }
    }

    /// Check if there are pending requests.
    pub fn has_pending(&self) -> bool {
        self.state.pending.lock().unwrap().is_empty() == false
    }

    /// Stop the debug server.
    pub fn stop(&self) {
        self.state.running.store(false, Ordering::SeqCst);
    }
}

impl Default for DebugServer {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for DebugServer {
    fn drop(&mut self) {
        self.stop();
        let _ = std::fs::remove_file(&self.socket_path);
    }
}

fn handle_client(stream: UnixStream, state: Arc<ServerState>) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut writer = stream;

    loop {
        let mut line = String::new();
        let bytes = reader.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Create response channel
        let response: Arc<(Mutex<Option<String>>, Condvar)> =
            Arc::new((Mutex::new(None), Condvar::new()));

        // Queue the request
        {
            let mut pending = state.pending.lock().unwrap();
            pending.push_back(PendingRequest {
                json: line.to_string(),
                response: response.clone(),
            });
        }

        // Wait for response (with timeout)
        let (lock, cvar) = &*response;
        let mut result = lock.lock().unwrap();
        let timeout = std::time::Duration::from_secs(10);
        while result.is_none() {
            let (new_result, timeout_result) = cvar.wait_timeout(result, timeout).unwrap();
            result = new_result;
            if timeout_result.timed_out() {
                writeln!(writer, "{{\"status\": \"error\", \"message\": \"Request timed out\"}}")?;
                writer.flush()?;
                continue;
            }
        }

        if let Some(ref resp) = *result {
            writeln!(writer, "{}", resp)?;
            writer.flush()?;
        }
    }

    Ok(())
}

/// Process a JSON message (command or query).
pub fn process_message<T: 'static>(canvas: &Entity<Canvas>, json: &str, cx: &mut Context<T>) -> String {
    // Try to parse as command first
    if let Ok(cmd) = serde_json::from_str::<Command>(json) {
        let result = execute_command_in_context(canvas, cmd, cx);
        return serde_json::to_string(&result).unwrap_or_else(|e| {
            format!(
                "{{\"status\": \"error\", \"message\": \"Serialization failed: {}\"}}",
                e
            )
        });
    }

    // Try to parse as query
    if let Ok(query) = serde_json::from_str::<Query>(json) {
        let result = execute_query_in_context(canvas, query, cx);
        return serde_json::to_string(&result).unwrap_or_else(|e| {
            format!(
                "{{\"status\": \"error\", \"message\": \"Serialization failed: {}\"}}",
                e
            )
        });
    }

    // Neither - return error
    format!(
        "{{\"status\": \"error\", \"message\": \"Invalid JSON: not a valid command or query\"}}"
    )
}
