use crate::cli::OutputFormat;
use crate::output::OutputWriter;
use crate::protocols::AppState;
use crate::toplevel::Toplevel;
use anyhow::Result;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::sync::{Arc, Mutex};

pub fn run_repl(app: &mut AppState) -> Result<()> {
    // Get the connection and create event queue
    let conn = app.conn.clone();
    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();

    // Get registry and bind protocols
    let display = conn.display();
    display.get_registry(&qh, ());

    // First roundtrip to get globals
    event_queue.roundtrip(app)?;

    if !app.has_protocol() {
        anyhow::bail!(
            "Wayland server supports none of the protocol extensions required for getting toplevel information"
        );
    }

    // Second roundtrip to get toplevel data
    event_queue.roundtrip(app)?;

    // Get initial state
    let used_protocol = app.used_protocol;
    let shared_toplevels = Arc::new(Mutex::new(app.toplevels.clone()));
    let toplevels_for_thread = shared_toplevels.clone();

    // Clone everything we need for the background thread
    let force_protocol = app.force_protocol.clone();
    let mode = app.mode;
    let next_id = app.next_id;
    let output_names = app.output_names.clone();
    let initial_toplevels = app.toplevels.clone();

    // Move the event queue and app state into background thread
    let _event_thread = std::thread::spawn(move || {
        let mut app_state = AppState {
            toplevels: initial_toplevels,
            used_protocol,
            force_protocol,
            mode,
            next_id,
            conn,
            output_names,
        };

        loop {
            // Blocking dispatch - waits for events
            if let Err(e) = event_queue.blocking_dispatch(&mut app_state) {
                eprintln!("Event dispatch error: {}", e);
                break;
            }

            // Update shared state
            let mut toplevels = toplevels_for_thread.lock().unwrap();
            *toplevels = app_state.toplevels.clone();
        }
    });

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
                        let writer = OutputWriter::new(&OutputFormat::Normal, &None);
                        writer.write_toplevels(&toplevels, used_protocol)?;
                    }
                    "list-json" | "lj" => {
                        let toplevels = shared_toplevels.lock().unwrap();
                        let writer = OutputWriter::new(&OutputFormat::Json, &None);
                        writer.write_toplevels(&toplevels, used_protocol)?;
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
                                println!("Protocol: {:?}", used_protocol);
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
