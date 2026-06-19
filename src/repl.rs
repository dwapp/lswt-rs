use crate::cli::{Mode, OutputFormat};
use crate::output::OutputWriter;
use crate::protocols::{AppState, UsedProtocol};
use crate::toplevel::Toplevel;
use anyhow::Result;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::sync::{Arc, Mutex};

pub fn run_repl(app: &mut AppState) -> Result<()> {
    // Clone the connection for the event queue
    let conn = app.conn.clone();
    let mut event_queue = conn.new_event_queue();

    // Wrap app in Arc<Mutex<>> for sharing
    let app = Arc::new(Mutex::new(std::mem::replace(
        app,
        AppState {
            toplevels: Vec::new(),
            used_protocol: UsedProtocol::None,
            force_protocol: None,
            mode: Mode::Repl,
            next_id: 0,
            conn: conn.clone(),
            output_names: std::collections::HashMap::new(),
        },
    )));

    // Store the app reference for the event loop
    let app_for_events = app.clone();

    // Spawn a thread to process Wayland events
    let _event_thread = std::thread::spawn(move || loop {
        {
            let mut state = app_for_events.lock().unwrap();
            if let Err(e) = event_queue.dispatch_pending(&mut *state) {
                eprintln!("Event dispatch error: {}", e);
                break;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    });

    // Run REPL in main thread
    let mut rl = DefaultEditor::new()?;
    let history_file = dirs_next::home_dir().map(|h| h.join(".lswt_history"));

    // Load history
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

                // Add to history
                let _ = rl.add_history_entry(line);

                match line {
                    "list" | "ls" => {
                        let state = app.lock().unwrap();
                        let writer = OutputWriter::new(&OutputFormat::Normal, &None);
                        writer.write_toplevels(&state.toplevels, state.used_protocol)?;
                    }
                    "list-json" | "lj" => {
                        let state = app.lock().unwrap();
                        let writer = OutputWriter::new(&OutputFormat::Json, &None);
                        writer.write_toplevels(&state.toplevels, state.used_protocol)?;
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
                                    let state = app.lock().unwrap();
                                    if let Ok(id) = parts[1].parse::<usize>() {
                                        show_toplevel_info(&state.toplevels, id);
                                    } else {
                                        let app_id = parts[1];
                                        show_toplevel_by_app_id(&state.toplevels, app_id);
                                    }
                                } else {
                                    eprintln!("Usage: info <id|app-id>");
                                }
                            }
                            Some("count") => {
                                let state = app.lock().unwrap();
                                println!("Toplevels: {}", state.toplevels.len());
                            }
                            Some("protocol") => {
                                let state = app.lock().unwrap();
                                println!("Protocol: {:?}", state.used_protocol);
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

    // Save history
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
