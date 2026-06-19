pub mod common;
pub mod ext_foreign_toplevel;
pub mod treeland_foreign_toplevel;
pub mod wlr_foreign_toplevel;

use crate::cli::{Args, Mode};
use crate::toplevel::{Toplevel, ToplevelHandle};
use anyhow::Result;
use std::collections::HashMap;

use wayland_client::{
    protocol::{wl_output, wl_registry},
    Connection, Dispatch, QueueHandle,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UsedProtocol {
    None,
    WlrForeignToplevel,
    ExtForeignToplevel,
    TreelandForeignToplevel,
}

pub struct AppState {
    pub toplevels: Vec<Toplevel>,
    pub used_protocol: UsedProtocol,
    pub force_protocol: Option<String>,
    pub mode: Mode,
    pub conn: Connection,
    pub output_names: HashMap<u32, String>, // wl_output name -> output name
    pub handles: Vec<Option<ToplevelHandle>>,
}

impl AppState {
    pub fn new(args: &Args) -> Result<Self> {
        let conn = Connection::connect_to_env()?;

        Ok(Self {
            toplevels: Vec::new(),
            used_protocol: UsedProtocol::None,
            force_protocol: args.force_protocol.clone(),
            mode: args.mode,
            conn,
            output_names: HashMap::new(),
            handles: Vec::new(),
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let mut event_queue = self.conn.new_event_queue();
        let qh = event_queue.handle();

        // Get registry
        let display = self.conn.display();
        display.get_registry(&qh, ());

        // First roundtrip to get globals
        event_queue.roundtrip(self)?;

        // Check if we found a supported protocol
        if !self.has_protocol() {
            anyhow::bail!(
                "Wayland server supports none of the protocol extensions required for getting toplevel information:\n");
        }

        // Second roundtrip to get toplevel data
        event_queue.roundtrip(self)?;

        // Continue running if in watch mode
        if self.mode == Mode::Watch || self.mode == Mode::VerboseWatch {
            eprintln!("Watching for toplevel changes... (Press Ctrl+C to exit)");
            loop {
                event_queue.blocking_dispatch(self)?;
            }
        }

        Ok(())
    }

    pub fn add_toplevel(&mut self, toplevel: Toplevel) {
        if self.mode == Mode::Watch || self.mode == Mode::VerboseWatch {
            println!("toplevel {}: created", toplevel.id);
        }
        self.toplevels.push(toplevel);
    }

    pub fn find_toplevel_mut(&mut self, id: usize) -> Option<&mut Toplevel> {
        self.toplevels.iter_mut().find(|t| t.id == id)
    }

    pub fn remove_toplevel(&mut self, id: usize) {
        if let Some(pos) = self.toplevels.iter().position(|t| t.id == id) {
            if let Some(handle_id) = self.toplevels[pos].handle_id {
                if let Some(handle) = self.handles.get_mut(handle_id) {
                    *handle = None;
                }
            }

            if self.mode == Mode::Watch || self.mode == Mode::VerboseWatch {
                println!("toplevel {}: destroyed", id);
            }
            self.toplevels.remove(pos);
        }
    }

    pub fn has_protocol(&self) -> bool {
        self.used_protocol != UsedProtocol::None
    }

    /// Check if force_protocol matches a specific protocol (supports short names)
    pub fn force_protocol_matches(&self, protocol: &str) -> bool {
        match &self.force_protocol {
            Some(fp) => {
                let fp_lower = fp.to_lowercase();
                match fp_lower.as_str() {
                    // Short names
                    "wlr" => protocol == "zwlr-foreign-toplevel-management-unstable-v1",
                    "treeland" => protocol == "treeland-foreign-toplevel-manager-v1",
                    "ext" => protocol == "ext-foreign-toplevel-list-v1",
                    // Full names (case-insensitive)
                    _ => fp_lower == protocol.to_lowercase(),
                }
            }
            None => true, // No force_protocol means accept any
        }
    }

    #[allow(dead_code)]
    pub fn supports_identifier(&self) -> bool {
        self.used_protocol == UsedProtocol::ExtForeignToplevel
            || self.used_protocol == UsedProtocol::TreelandForeignToplevel
    }

    #[allow(dead_code)]
    pub fn supports_state(&self) -> bool {
        self.used_protocol == UsedProtocol::WlrForeignToplevel
            || self.used_protocol == UsedProtocol::TreelandForeignToplevel
    }
}

// Implement Dispatch for registry
impl Dispatch<wl_registry::WlRegistry, ()> for AppState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match interface.as_str() {
                "zwlr_foreign_toplevel_manager_v1"
                    if version >= 3
                        && state.force_protocol_matches(
                            "zwlr-foreign-toplevel-management-unstable-v1",
                        )
                        && state.used_protocol == UsedProtocol::None =>
                {
                    use wayland_protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1;
                    let _manager: ZwlrForeignToplevelManagerV1 = registry.bind(name, 3, qh, ());
                    state.used_protocol = UsedProtocol::WlrForeignToplevel;
                }
                "treeland_foreign_toplevel_manager_v1"
                    if state.force_protocol_matches("treeland-foreign-toplevel-manager-v1")
                        && state.used_protocol == UsedProtocol::None =>
                {
                    use wayland_protocols_treeland::foreign_toplevel_manager::v1::client::treeland_foreign_toplevel_manager_v1::TreelandForeignToplevelManagerV1;
                    let _manager: TreelandForeignToplevelManagerV1 =
                        registry.bind(name, version.min(2), qh, ());
                    state.used_protocol = UsedProtocol::TreelandForeignToplevel;
                }
                "ext_foreign_toplevel_list_v1"
                    if state.force_protocol_matches("ext-foreign-toplevel-list-v1")
                        && state.used_protocol == UsedProtocol::None =>
                {
                    use wayland_protocols::ext::foreign_toplevel_list::v1::client::ext_foreign_toplevel_list_v1::ExtForeignToplevelListV1;
                    let _list: ExtForeignToplevelListV1 = registry.bind(name, 1, qh, ());
                    state.used_protocol = UsedProtocol::ExtForeignToplevel;
                }
                "wl_output" => {
                    let _output: wl_output::WlOutput = registry.bind(name, 1, qh, name);
                }
                _ => {}
            }
        }
    }
}

// Implement Dispatch for wl_output to track output names
impl Dispatch<wl_output::WlOutput, u32> for AppState {
    fn event(
        state: &mut Self,
        _output: &wl_output::WlOutput,
        event: wl_output::Event,
        data: &u32,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_output::Event::Name { name } = event {
            state.output_names.insert(*data, name);
        }
    }
}
