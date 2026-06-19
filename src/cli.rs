use clap::Parser;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Mode {
    #[default]
    List,
    Watch,
    VerboseWatch,
    Repl,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum OutputFormat {
    #[default]
    Normal,
    Json,
    Custom,
}

#[derive(Parser, Debug)]
#[command(name = "lswt")]
#[command(version = "2.0.0")]
#[command(about = "List Wayland toplevels", long_about = None)]
pub struct Args {
    /// Output data in JSON format
    #[arg(short, long)]
    pub json: bool,

    /// Run continuously and log title, identifier and app-id events
    #[arg(short, long)]
    pub watch: bool,

    /// Like --watch, but also log activated, fullscreen, minimized and maximized state
    #[arg(short = 'W', long)]
    pub verbose_watch: bool,

    /// Enter interactive REPL mode
    #[arg(long)]
    pub repl: bool,

    /// Define a custom line-based output format
    #[arg(short, long, value_name = "fmt")]
    pub custom: Option<String>,

    /// Use specified protocol, do not fall back onto others
    /// Supported: wlr, treeland, ext (or full protocol names)
    #[arg(long, value_name = "name")]
    pub force_protocol: Option<String>,

    #[clap(skip)]
    pub mode: Mode,

    #[clap(skip)]
    pub output_format: OutputFormat,

    #[clap(skip)]
    pub custom_format: Option<String>,
}

impl Args {
    pub fn parse_args() -> Self {
        let mut args = Args::parse();

        // Determine mode
        args.mode = if args.repl {
            Mode::Repl
        } else if args.verbose_watch {
            Mode::VerboseWatch
        } else if args.watch {
            Mode::Watch
        } else {
            Mode::List
        };

        // Determine output format
        if let Some(ref fmt) = args.custom {
            if !validate_custom_format(fmt) {
                eprintln!("ERROR: Invalid custom format");
                std::process::exit(1);
            }
            args.output_format = OutputFormat::Custom;
            args.custom_format = Some(fmt.clone());
        } else if args.json {
            args.output_format = OutputFormat::Json;
        } else {
            args.output_format = OutputFormat::Normal;
        }

        args
    }
}

fn validate_custom_format(fmt: &str) -> bool {
    if fmt.is_empty() {
        eprintln!("ERROR: Invalid custom format: Requires at least one field.");
        return false;
    }

    for c in fmt.chars() {
        match c {
            't' | 'a' | 'i' | 'A' | 'f' | 'm' | 'M' => continue,
            _ => {
                eprintln!("ERROR: Invalid custom format: Unknown field name: '{}'.", c);
                eprintln!("INFO:  Supported field names:");
                eprintln!("\tt: title");
                eprintln!("\ta: app-id");
                eprintln!("\ti: identifier");
                eprintln!("\tA: activated?");
                eprintln!("\tf: fullscreen?");
                eprintln!("\tm: minimized?");
                eprintln!("\tM: maximized?");
                return false;
            }
        }
    }

    true
}
