use crate::cli::{Args, OutputFormat};
use crate::output::OutputWriter;
use crate::protocols::{AppState, UsedProtocol};
use crate::toplevel::Toplevel;
use anyhow::Result;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::sync::{Arc, Mutex};

pub fn run_repl(args: &Args) -> Result<()> {
    // Create connection and event queue in main thread
    let conn = wayland_client::Connection::connect_to_env()?;

    // Shared state for toplevels
    let shared_toplevels: Arc<Mutex<Vec<Toplevel>>> = Arc::new(Mutex::new(Vec::new()));
    let shared_protocol: Arc<Mutex<UsedProtocol>> = Arc::new(Mutex::new(UsedProtocol::None));
    let toplevels_for_thread = shared_toplevels.clone();
    let protocol_for_thread = shared_protocol.clone();

    // Clone args for the background thread
    let force_protocol = args.force_protocol.clone();
    let mode = args.mode;

    // Spawn background thread for event processing
    let _event_thread = std::thread::spawn(move || {
        // Create AppState in the background thread
        let mut app = AppState {
            toplevels: Vec::new(),
            used_protocol: UsedProtocol::None,
            force_protocol,
            mode,
            next_id: 0,
            conn: conn.clone(),
            output_names: std::collections::HashMap::new(),
        };

        // Initialize: bind protocols and get initial state
        let mut event_queue = conn.new_event_queue();
        let qh = event_queue.handle();

        // Get registry
        let display = conn.display();
        display.get_registry(&qh, ());

        // First roundtrip to get globals
        if let Err(e) = event_queue.roundtrip(&mut app) {
            eprintln!("Error during initialization: {}", e);
            return;
        }

        if !app.has_protocol() {
            eprintln!("No supported protocol found");
            return;
        }

        // Second roundtrip to get initial toplevel data
        if let Err(e) = event_queue.roundtrip(&mut app) {
            eprintln!("Error getting initial data: {}", e);
            return;
        }

        // Update shared state with initial data
        {
            let mut toplevels = toplevels_for_thread.lock().unwrap();
            *toplevels = app.toplevels.clone();
            let mut protocol = protocol_for_thread.lock().unwrap();
            *protocol = app.used_protocol;
        }

        // Event loop - process events continuously
        loop {
            if let Err(e) = event_queue.blocking_dispatch(&mut app) {
                eprintln!("Event dispatch error: {}", e);
                break;
            }

            // Update shared state
            let mut toplevels = toplevels_for_thread.lock().unwrap();
            *toplevels = app.toplevels.clone();
        }
    });

    // Wait a bit for initialization
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Run REPL in main thread
    let mut rl = DefaultEditor::new()?;
    let history_file = dirs_next::home_dir().map(|h| h.join(".lswt_history"));

    if let Some(ref path) = history_file {
        let _ = rl.load_history(path);
    }

    println!("lswt REPL mode - type 'help' for commands, 'exit' or 'quit' to leave");

    loop {
        match rl.readline("lswt> ") {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                let _ = rl.add_history_entry(line);

                match line {
                    "list" | "ls" => {
                        let toplevels = shared_toplevels.lock().unwrap();
                        let protocol = shared_protocol.lock().unwrap();
                        let writer = OutputWriter::new(&OutputFormat::Normal, &None);
                        writer.write_toplevels(&toplevels, *protocol)?;
                    }
                    "list-json" | "lj" => {
                        let toplevels = shared_toplevels.lock().unwrap();
                        let protocol = shared_protocol.lock().unwrap();
                        let writer = OutputWriter::new(&OutputFormat::Json, &None);
                        writer.write_toplevels(&toplevels, *protocol)?;
                    }
                    "help" | "h" => {
                        print_help();
                    }
                    "exit" | "quit" | "q" => {
                        println!("Bye!");
                        break;
                    }
                    _ => {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        match parts.first().copied() {
                            Some("info") => {
                                if parts.len() > 1 {
                                    let toplevels = shared_toplevels.lock().unwrap();
                                    if let Ok(id) = parts[1].parse::<usize>() {
                                        show_toplevel_info(&toplevels, id);
                                    } else {
                                        show_toplevel_by_app_id(&toplevels, parts[1]);
                                    }
                                } else {
                                    eprintln!("Usage: info <id|app-id>");
                                }
                            }
                            Some("count") => {
                                let toplevels = shared_toplevels.lock().unwrap();
                                println!("Toplevels: {}", toplevels.len());
                            }
                            Some("protocol") => {
                                let protocol = shared_protocol.lock().unwrap();
                                println!("Protocol: {:?}", *protocol);
                            }
                            _ => {
                                eprintln!(
                                    "Unknown command: '{}'. Type 'help' for available commands.",
                                    line
                                );
                            }
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Use 'exit' or 'quit' to leave.");
            }
            Err(ReadlineError::Eof) => {
                println!("Bye!");
                break;
            }
            Err(err) => {
                eprintln!("Error: {}", err);
                break;
            }
        }
    }

    if let Some(ref path) = history_file {
        let _ = rl.save_history(path);
    }

    Ok(())
}

fn print_help() {
    println!("Available commands:");
    println!("  list, ls          - List all toplevels");
    println!("  list-json, lj     - List all toplevels in JSON format");
    println!("  info <id>         - Show detailed info for a toplevel by ID");
    println!("  info <app-id>     - Show detailed info for a toplevel by app-id");
    println!("  count             - Show number of toplevels");
    println!("  protocol          - Show current protocol");
    println!("  help, h           - Show this help");
    println!("  exit, quit, q     - Exit REPL");
}

fn show_toplevel_info(toplevels: &[Toplevel], id: usize) {
    if let Some(t) = toplevels.iter().find(|t| t.id == id) {
        println!("Toplevel #{}", t.id);
        println!("  Title:      {}", t.title_str());
        println!("  App ID:     {}", t.app_id_str());
        println!("  Identifier: {}", t.identifier_str());
        println!("  State:");
        println!("    Maximized:  {}", t.state.maximized);
        println!("    Minimized:  {}", t.state.minimized);
        println!("    Activated:  {}", t.state.activated);
        println!("    Fullscreen: {}", t.state.fullscreen);
        if !t.outputs.is_empty() {
            println!("  Outputs:    {}", t.outputs.join(", "));
        }
    } else {
        eprintln!("Toplevel #{} not found", id);
    }
}

fn show_toplevel_by_app_id(toplevels: &[Toplevel], app_id: &str) {
    let matches: Vec<&Toplevel> = toplevels
        .iter()
        .filter(|t| t.app_id.as_deref() == Some(app_id))
        .collect();

    if matches.is_empty() {
        eprintln!("No toplevel found with app-id '{}'", app_id);
    } else {
        for t in matches {
            show_toplevel_info(toplevels, t.id);
            println!();
        }
    }
}
