use crate::cli::OutputFormat;
use crate::output::OutputWriter;
use crate::protocols::AppState;
use crate::toplevel::Toplevel;
use anyhow::Result;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

pub fn run_repl(app: &mut AppState) -> Result<()> {
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
                        let writer = OutputWriter::new(&OutputFormat::Normal, &None);
                        writer.write_toplevels(&app.toplevels, app.used_protocol)?;
                    }
                    "list-json" | "lj" => {
                        let writer = OutputWriter::new(&OutputFormat::Json, &None);
                        writer.write_toplevels(&app.toplevels, app.used_protocol)?;
                    }
                    "help" | "h" => {
                        print_help();
                    }
                    "exit" | "quit" | "q" => {
                        println!("Bye!");
                        break;
                    }
                    _ => {
                        // Try to parse as a command
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        match parts.first().copied() {
                            Some("list") | Some("ls") => {
                                let writer = OutputWriter::new(&OutputFormat::Normal, &None);
                                writer.write_toplevels(&app.toplevels, app.used_protocol)?;
                            }
                            Some("list-json") | Some("lj") => {
                                let writer = OutputWriter::new(&OutputFormat::Json, &None);
                                writer.write_toplevels(&app.toplevels, app.used_protocol)?;
                            }
                            Some("info") => {
                                if parts.len() > 1 {
                                    if let Ok(id) = parts[1].parse::<usize>() {
                                        show_toplevel_info(&app.toplevels, id);
                                    } else {
                                        // Search by app-id
                                        let app_id = parts[1];
                                        show_toplevel_by_app_id(&app.toplevels, app_id);
                                    }
                                } else {
                                    eprintln!("Usage: info <id|app-id>");
                                }
                            }
                            Some("count") => {
                                println!("Toplevels: {}", app.toplevels.len());
                            }
                            Some("protocol") => {
                                println!("Protocol: {:?}", app.used_protocol);
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
