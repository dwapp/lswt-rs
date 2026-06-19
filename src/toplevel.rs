use serde::Serialize;
use std::sync::{Arc, Mutex};
use wayland_client::Connection;

#[derive(Debug, Clone, Default, Serialize)]
pub struct ToplevelState {
    pub fullscreen: bool,
    pub activated: bool,
    pub maximized: bool,
    pub minimized: bool,
}

/// Actions that can be performed on a toplevel
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToplevelAction {
    Maximize,
    UnMaximize,
    Minimize,
    UnMinimize,
    Activate,
    Fullscreen,
    UnFullscreen,
    Close,
}

/// Handle to a toplevel for performing actions
#[derive(Clone)]
pub enum ToplevelHandle {
    Wlr(wayland_protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1),
    Treeland(wayland_protocols_treeland::foreign_toplevel_manager::v1::client::treeland_foreign_toplevel_handle_v1::TreelandForeignToplevelHandleV1),
}

#[derive(Debug, Clone, Serialize)]
pub struct Toplevel {
    pub id: usize,
    pub title: Option<String>,
    pub app_id: Option<String>,
    pub identifier: Option<String>,
    pub state: ToplevelState,
    pub outputs: Vec<String>,
    #[serde(skip)]
    pub listed: bool,
    #[serde(skip)]
    #[allow(dead_code)]
    pub handle_id: Option<usize>, // Index into handles vec
}

impl Toplevel {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            title: None,
            app_id: None,
            identifier: None,
            state: ToplevelState::default(),
            outputs: Vec::new(),
            listed: false,
            handle_id: None,
        }
    }

    pub fn set_title(&mut self, title: String) {
        self.title = Some(title);
    }

    pub fn set_app_id(&mut self, app_id: String) {
        self.app_id = Some(app_id);
    }

    pub fn set_identifier(&mut self, identifier: String) {
        self.identifier = Some(identifier);
    }

    pub fn set_state(&mut self, state: ToplevelState) {
        self.state = state;
    }

    pub fn mark_listed(&mut self) {
        self.listed = true;
    }

    pub fn add_output(&mut self, output: String) {
        if !self.outputs.contains(&output) {
            self.outputs.push(output);
        }
    }

    pub fn remove_output(&mut self, output: &str) {
        self.outputs.retain(|o| o != output);
    }

    pub fn title_str(&self) -> &str {
        self.title.as_deref().unwrap_or("<NULL>")
    }

    pub fn app_id_str(&self) -> &str {
        self.app_id.as_deref().unwrap_or("<NULL>")
    }

    pub fn identifier_str(&self) -> &str {
        self.identifier.as_deref().unwrap_or("<NULL>")
    }
}

/// Store for toplevel handles, shared between threads
pub type SharedHandles = Arc<Mutex<Vec<Option<ToplevelHandle>>>>;

/// Perform an action on a toplevel
pub fn perform_action(
    handles: &SharedHandles,
    conn: &Connection,
    toplevel_id: usize,
    action: ToplevelAction,
) -> Result<(), String> {
    let handle = {
        let handles = handles.lock().unwrap();
        handles
            .get(toplevel_id)
            .and_then(|h| h.as_ref())
            .cloned()
            .ok_or_else(|| format!("No handle found for toplevel {}", toplevel_id))?
    };

    match &handle {
        ToplevelHandle::Wlr(h) => {
            match action {
                ToplevelAction::Maximize => h.set_maximized(),
                ToplevelAction::UnMaximize => h.unset_maximized(),
                ToplevelAction::Minimize => h.set_minimized(),
                ToplevelAction::UnMinimize => h.unset_minimized(),
                ToplevelAction::Activate => {
                    // wlr activate requires a seat, we'll use a workaround
                    // For now, just log that it's not fully supported
                    return Err(
                        "Activate requires a seat object (not yet implemented for wlr)".to_string(),
                    );
                }
                ToplevelAction::Fullscreen => h.set_fullscreen(None),
                ToplevelAction::UnFullscreen => h.unset_fullscreen(),
                ToplevelAction::Close => h.close(),
            }
            Ok(())
        }
        ToplevelHandle::Treeland(h) => {
            match action {
                ToplevelAction::Maximize => h.set_maximized(),
                ToplevelAction::UnMaximize => h.unset_maximized(),
                ToplevelAction::Minimize => h.set_minimized(),
                ToplevelAction::UnMinimize => h.unset_minimized(),
                ToplevelAction::Activate => {
                    // treeland activate requires a seat
                    return Err(
                        "Activate requires a seat object (not yet implemented for treeland)"
                            .to_string(),
                    );
                }
                ToplevelAction::Fullscreen => h.set_fullscreen(None),
                ToplevelAction::UnFullscreen => h.unset_fullscreen(),
                ToplevelAction::Close => h.close(),
            }
            Ok(())
        }
    }
    .and_then(|_| {
        conn.flush()
            .map_err(|e| format!("Failed to flush Wayland request: {}", e))
    })
}
