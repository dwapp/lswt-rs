pub mod common;
pub mod ext_foreign_toplevel;
pub mod treeland_foreign_toplevel;
pub mod wlr_foreign_toplevel;

use crate::cli::{Args, Mode};
use crate::toplevel::Toplevel;
use anyhow::Result;
use wayland_client::{protocol::wl_registry, Connection, Dispatch, QueueHandle};

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
    pub next_id: usize,
    pub conn: Connection,
}

impl AppState {
    pub fn new(args: &Args) -> Result<Self> {
        let conn = Connection::connect_to_env()?;

        Ok(Self {
            toplevels: Vec::new(),
            used_protocol: UsedProtocol::None,
            force_protocol: args.force_protocol.clone(),
            mode: args.mode,
            next_id: 0,
            conn,
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

    pub fn next_toplevel_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
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
            if self.mode == Mode::Watch || self.mode == Mode::VerboseWatch {
                println!("toplevel {}: destroyed", id);
            }
            self.toplevels.remove(pos);
        }
    }

    pub fn has_protocol(&self) -> bool {
        self.used_protocol != UsedProtocol::None
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
                        && (state.force_protocol.is_none()
                            || state.force_protocol.as_deref()
                                == Some("zwlr-foreign-toplevel-management-unstable-v1"))
                        && state.used_protocol == UsedProtocol::None =>
                {
                    use wayland_protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1;
                    let _manager: ZwlrForeignToplevelManagerV1 = registry.bind(name, 3, qh, ());
                    state.used_protocol = UsedProtocol::WlrForeignToplevel;
                }
                "treeland_foreign_toplevel_manager_v1"
                    if (state.force_protocol.is_none()
                        || state.force_protocol.as_deref()
                            == Some("treeland-foreign-toplevel-manager-v1"))
                        && state.used_protocol == UsedProtocol::None =>
                {
                    use wayland_protocols_treeland::foreign_toplevel_manager::v1::client::treeland_foreign_toplevel_manager_v1::TreelandForeignToplevelManagerV1;
                    let _manager: TreelandForeignToplevelManagerV1 =
                        registry.bind(name, version.min(2), qh, ());
                    state.used_protocol = UsedProtocol::TreelandForeignToplevel;
                }
                "ext_foreign_toplevel_list_v1"
                    if (state.force_protocol.is_none()
                        || state.force_protocol.as_deref() == Some("ext-foreign-toplevel-list-v1"))
                        && state.used_protocol == UsedProtocol::None =>
                {
                    // TODO: Bind ext-foreign-toplevel-list-v1 when available
                }
                _ => {}
            }
        }
    }
}
