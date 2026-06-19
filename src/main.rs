mod cli;
mod output;
mod protocols;
mod repl;
mod toplevel;

use anyhow::Result;
use cli::{Args, Mode};
use output::OutputWriter;
use protocols::AppState;

fn main() -> Result<()> {
    let args = Args::parse_args();

    // Create and run the application
    let mut app = AppState::new(&args)?;
    app.run()?;

    // Output results if in list mode
    match args.mode {
        Mode::List => {
            let writer = OutputWriter::new(&args.output_format, &args.custom_format);
            writer.write_toplevels(&app.toplevels, app.used_protocol)?;
        }
        Mode::Repl => {
            repl::run_repl(&args)?;
        }
        _ => {}
    }

    Ok(())
}
