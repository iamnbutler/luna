//! Luna CLI - Command-line interface for interacting with Luna instances.
//!
//! Connect to a running Luna process and send commands/queries via JSON.

use anyhow::{Context, Result};
use api::{Command, Query};
use clap::{Parser, Subcommand};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

/// Luna CLI - interact with running Luna instances
#[derive(Parser)]
#[command(name = "luna")]
#[command(about = "Command-line interface for Luna design tool")]
struct Cli {
    /// Socket path to connect to (default: auto-detect)
    #[arg(short, long)]
    socket: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List running Luna instances
    List,

    /// Connect to a Luna instance and start interactive mode
    Connect {
        /// Process ID of the Luna instance
        #[arg(short, long)]
        pid: Option<u32>,
    },

    /// Send a single command to Luna
    Command {
        /// JSON command to execute
        json: String,
    },

    /// Send a query to Luna
    Query {
        /// JSON query to execute
        json: String,
    },

    /// Get all shapes from canvas
    Shapes,

    /// Get current selection
    Selection,

    /// Get shape count
    Count,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List => list_instances(),
        Commands::Connect { pid } => connect_interactive(cli.socket, pid),
        Commands::Command { json } => send_command(cli.socket, &json),
        Commands::Query { json } => send_query(cli.socket, &json),
        Commands::Shapes => query_shapes(cli.socket),
        Commands::Selection => query_selection(cli.socket),
        Commands::Count => query_count(cli.socket),
    }
}

/// List all running Luna instances by checking socket files.
fn list_instances() -> Result<()> {
    let sockets = find_luna_sockets()?;
    if sockets.is_empty() {
        println!("No running Luna instances found.");
        println!("\nNote: Luna must be running with debug server enabled.");
        println!("Start Luna with: LUNA_DEBUG=1 Luna2");
    } else {
        println!("Running Luna instances:");
        for socket in sockets {
            let pid = extract_pid_from_socket(&socket);
            if let Some(pid) = pid {
                println!("  PID {}: {}", pid, socket.display());
            } else {
                println!("  {}", socket.display());
            }
        }
    }
    Ok(())
}

/// Find all Luna socket files in /tmp.
fn find_luna_sockets() -> Result<Vec<PathBuf>> {
    let mut sockets = Vec::new();
    let tmp = PathBuf::from("/tmp");

    if let Ok(entries) = std::fs::read_dir(&tmp) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("luna-") && name.ends_with(".sock") {
                    // Verify socket is connectable
                    if UnixStream::connect(&path).is_ok() {
                        sockets.push(path);
                    }
                }
            }
        }
    }

    Ok(sockets)
}

/// Extract PID from socket filename like "luna-12345.sock".
fn extract_pid_from_socket(path: &PathBuf) -> Option<u32> {
    path.file_name()
        .and_then(|n| n.to_str())
        .and_then(|name| {
            let name = name.strip_prefix("luna-")?;
            let name = name.strip_suffix(".sock")?;
            name.parse().ok()
        })
}

/// Connect to Luna and start interactive REPL.
fn connect_interactive(socket: Option<PathBuf>, pid: Option<u32>) -> Result<()> {
    let socket_path = resolve_socket(socket, pid)?;
    println!("Connecting to Luna at {}...", socket_path.display());

    let stream = UnixStream::connect(&socket_path)
        .with_context(|| format!("Failed to connect to {}", socket_path.display()))?;

    println!("Connected! Enter commands (JSON) or 'help' for usage. Ctrl+C to exit.\n");

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut writer = stream;
    let stdin = std::io::stdin();

    loop {
        print!("luna> ");
        std::io::stdout().flush()?;

        let mut input = String::new();
        if stdin.lock().read_line(&mut input)? == 0 {
            break;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        match input {
            "help" | "?" => print_help(),
            "quit" | "exit" => break,
            "shapes" => {
                let query = Query::GetAllShapes;
                send_to_stream(&mut writer, &mut reader, &serde_json::to_string(&query)?)?;
            }
            "selection" => {
                let query = Query::GetSelection;
                send_to_stream(&mut writer, &mut reader, &serde_json::to_string(&query)?)?;
            }
            "count" => {
                let query = Query::GetShapeCount;
                send_to_stream(&mut writer, &mut reader, &serde_json::to_string(&query)?)?;
            }
            _ => {
                // Try to parse as command or query
                send_to_stream(&mut writer, &mut reader, input)?;
            }
        }
    }

    Ok(())
}

fn print_help() {
    println!("Luna CLI Interactive Mode");
    println!("========================");
    println!();
    println!("Built-in commands:");
    println!("  shapes      - Get all shapes on canvas");
    println!("  selection   - Get current selection");
    println!("  count       - Get shape count");
    println!("  help, ?     - Show this help");
    println!("  quit, exit  - Exit interactive mode");
    println!();
    println!("JSON Commands (examples):");
    println!("  {{\"type\": \"create_shape\", \"kind\": \"Rectangle\", \"position\": [100, 100], \"size\": [50, 50]}}");
    println!("  {{\"type\": \"delete\", \"target\": \"selection\"}}");
    println!("  {{\"type\": \"move\", \"target\": \"selection\", \"delta\": [10, 0]}}");
    println!("  {{\"type\": \"set_fill\", \"target\": \"selection\", \"fill\": {{\"h\": 0.5, \"s\": 1.0, \"l\": 0.5, \"a\": 1.0}}}}");
    println!();
    println!("JSON Queries:");
    println!("  {{\"type\": \"get_selection\"}}");
    println!("  {{\"type\": \"get_all_shapes\"}}");
    println!("  {{\"type\": \"get_viewport\"}}");
    println!("  {{\"type\": \"get_tool\"}}");
}

fn send_to_stream(writer: &mut UnixStream, reader: &mut BufReader<UnixStream>, json: &str) -> Result<()> {
    // Send the command
    writeln!(writer, "{}", json)?;
    writer.flush()?;

    // Read response
    let mut response = String::new();
    reader.read_line(&mut response)?;

    // Pretty print the response
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&response) {
        println!("{}", serde_json::to_string_pretty(&value)?);
    } else {
        println!("{}", response.trim());
    }

    Ok(())
}

/// Send a single command and exit.
fn send_command(socket: Option<PathBuf>, json: &str) -> Result<()> {
    let socket_path = resolve_socket(socket, None)?;
    let stream = UnixStream::connect(&socket_path)?;

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut writer = stream;

    // Parse to validate
    let _: Command = serde_json::from_str(json)
        .with_context(|| "Invalid command JSON")?;

    send_to_stream(&mut writer, &mut reader, json)?;
    Ok(())
}

/// Send a single query and exit.
fn send_query(socket: Option<PathBuf>, json: &str) -> Result<()> {
    let socket_path = resolve_socket(socket, None)?;
    let stream = UnixStream::connect(&socket_path)?;

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut writer = stream;

    // Parse to validate
    let _: Query = serde_json::from_str(json)
        .with_context(|| "Invalid query JSON")?;

    send_to_stream(&mut writer, &mut reader, json)?;
    Ok(())
}

/// Query all shapes.
fn query_shapes(socket: Option<PathBuf>) -> Result<()> {
    let query = Query::GetAllShapes;
    let json = serde_json::to_string(&query)?;
    send_query(socket, &json)
}

/// Query selection.
fn query_selection(socket: Option<PathBuf>) -> Result<()> {
    let query = Query::GetSelection;
    let json = serde_json::to_string(&query)?;
    send_query(socket, &json)
}

/// Query shape count.
fn query_count(socket: Option<PathBuf>) -> Result<()> {
    let query = Query::GetShapeCount;
    let json = serde_json::to_string(&query)?;
    send_query(socket, &json)
}

/// Resolve which socket to connect to.
fn resolve_socket(explicit: Option<PathBuf>, pid: Option<u32>) -> Result<PathBuf> {
    // If explicit socket provided, use it
    if let Some(socket) = explicit {
        return Ok(socket);
    }

    // If PID provided, construct socket path
    if let Some(pid) = pid {
        return Ok(PathBuf::from(format!("/tmp/luna-{}.sock", pid)));
    }

    // Auto-detect: find first available socket
    let sockets = find_luna_sockets()?;
    sockets.into_iter().next()
        .ok_or_else(|| anyhow::anyhow!(
            "No Luna instances found. Start Luna with LUNA_DEBUG=1 or specify --socket"
        ))
}
