use crate::cli::OutputFormat;
use crate::protocols::UsedProtocol;
use crate::toplevel::Toplevel;
use anyhow::Result;
use serde_json::json;

pub struct OutputWriter {
    format: OutputFormat,
    custom_format: Option<String>,
}

impl OutputWriter {
    pub fn new(format: &OutputFormat, custom_format: &Option<String>) -> Self {
        Self {
            format: format.clone(),
            custom_format: custom_format.clone(),
        }
    }

    pub fn write_toplevels(&self, toplevels: &[Toplevel], protocol: UsedProtocol) -> Result<()> {
        match self.format {
            OutputFormat::Normal => self.write_normal(toplevels, protocol),
            OutputFormat::Json => self.write_json(toplevels, protocol),
            OutputFormat::Custom => self.write_custom(toplevels, protocol),
        }
    }

    fn write_normal(&self, toplevels: &[Toplevel], protocol: UsedProtocol) -> Result<()> {
        let supports_state = protocol == UsedProtocol::WlrForeignToplevel
            || protocol == UsedProtocol::TreelandForeignToplevel;
        let max_app_id_len = toplevels
            .iter()
            .map(|t| self.display_len(t.app_id_str()))
            .max()
            .unwrap_or(7)
            .clamp(7, 40); // At least "app-id:" length, max 40

        // Header
        if is_terminal() {
            print!("\x1b[0;1m"); // Bold
        }
        if supports_state {
            print!("state:   ");
        }
        print!("{:width$}   title:", "app-id:", width = max_app_id_len);
        if is_terminal() {
            print!("\x1b[0m"); // Reset
        }
        println!();

        // Toplevels
        for toplevel in toplevels {
            if supports_state {
                self.write_state(toplevel);
                print!("     ");
            }
            let app_id = self.format_string(toplevel.app_id_str());
            print!("{:width$}   ", app_id, width = max_app_id_len);
            print!("{}", self.format_string(toplevel.title_str()));
            if !toplevel.outputs.is_empty() {
                print!("  [{}]", toplevel.outputs.join(", "));
            }
            println!();
        }

        Ok(())
    }

    fn write_json(&self, toplevels: &[Toplevel], protocol: UsedProtocol) -> Result<()> {
        let supports_identifier = protocol == UsedProtocol::ExtForeignToplevel
            || protocol == UsedProtocol::TreelandForeignToplevel;
        let supports_state = protocol == UsedProtocol::WlrForeignToplevel
            || protocol == UsedProtocol::TreelandForeignToplevel;

        let mut output = json!({
            "json-output-version": 2,
            "supported-data": {
                "title": true,
                "app-id": true,
                "identifier": supports_identifier,
                "fullscreen": supports_state,
                "activated": supports_state,
                "minimized": supports_state,
                "maximized": supports_state,
            },
            "toplevels": []
        });

        let toplevels_array = output["toplevels"].as_array_mut().unwrap();

        for toplevel in toplevels {
            let mut t = json!({
                "title": toplevel.title.as_ref(),
                "app-id": toplevel.app_id.as_ref(),
            });

            if supports_state {
                t["activated"] = json!(toplevel.state.activated);
                t["fullscreen"] = json!(toplevel.state.fullscreen);
                t["minimized"] = json!(toplevel.state.minimized);
                t["maximized"] = json!(toplevel.state.maximized);
            }

            if supports_identifier {
                t["identifier"] = json!(toplevel.identifier.as_ref());
            }

            if !toplevel.outputs.is_empty() {
                t["outputs"] = json!(toplevel.outputs);
            }

            toplevels_array.push(t);
        }

        println!("{}", serde_json::to_string_pretty(&output)?);
        Ok(())
    }

    fn write_custom(&self, toplevels: &[Toplevel], protocol: UsedProtocol) -> Result<()> {
        let format = self.custom_format.as_ref().unwrap();
        let supports_identifier = protocol == UsedProtocol::ExtForeignToplevel
            || protocol == UsedProtocol::TreelandForeignToplevel;
        let supports_state = protocol == UsedProtocol::WlrForeignToplevel
            || protocol == UsedProtocol::TreelandForeignToplevel;

        for toplevel in toplevels {
            let mut first = true;
            for c in format.chars() {
                if !first {
                    print!(",");
                }
                first = false;

                match c {
                    't' => print!("{}", self.escape_custom(toplevel.title_str())),
                    'a' => print!("{}", self.escape_custom(toplevel.app_id_str())),
                    'i' => {
                        if supports_identifier {
                            print!("{}", self.escape_custom(toplevel.identifier_str()));
                        } else {
                            print!("unsupported");
                        }
                    }
                    'A' => {
                        if supports_state {
                            print!("{}", toplevel.state.activated);
                        } else {
                            print!("unsupported");
                        }
                    }
                    'f' => {
                        if supports_state {
                            print!("{}", toplevel.state.fullscreen);
                        } else {
                            print!("unsupported");
                        }
                    }
                    'm' => {
                        if supports_state {
                            print!("{}", toplevel.state.minimized);
                        } else {
                            print!("unsupported");
                        }
                    }
                    'M' => {
                        if supports_state {
                            print!("{}", toplevel.state.maximized);
                        } else {
                            print!("unsupported");
                        }
                    }
                    _ => {}
                }
            }
            println!();
        }

        Ok(())
    }

    fn write_state(&self, toplevel: &Toplevel) {
        print!("{}", if toplevel.state.maximized { 'm' } else { '-' });
        print!("{}", if toplevel.state.minimized { 'm' } else { '-' });
        print!("{}", if toplevel.state.activated { 'a' } else { '-' });
        print!("{}", if toplevel.state.fullscreen { 'f' } else { '-' });
    }

    fn format_string(&self, s: &str) -> String {
        if self.needs_quotes(s) {
            self.quote_string(s)
        } else {
            s.to_string()
        }
    }

    fn needs_quotes(&self, s: &str) -> bool {
        s.chars()
            .any(|c| c.is_whitespace() || c == '"' || c == '\'' || !c.is_ascii())
    }

    fn quote_string(&self, s: &str) -> String {
        let mut result = String::from("\"");
        for c in s.chars() {
            match c {
                '"' => result.push_str("\\\""),
                '\n' => result.push_str("\\n"),
                '\t' => result.push_str("\\t"),
                '\\' => result.push_str("\\\\"),
                _ => result.push(c),
            }
        }
        result.push('"');
        result
    }

    fn escape_custom(&self, s: &str) -> String {
        s.replace(',', "\\,")
    }

    fn display_len(&self, s: &str) -> usize {
        if self.needs_quotes(s) {
            self.quote_string(s).len()
        } else {
            s.len()
        }
    }
}

fn is_terminal() -> bool {
    use std::io::IsTerminal;
    std::io::stdout().is_terminal()
}
