use crate::cli::Mode;
use crate::protocols::AppState;
use crate::toplevel::Toplevel;
use std::sync::atomic::{AtomicUsize, Ordering};
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_protocols::ext::foreign_toplevel_list::v1::client::{
    ext_foreign_toplevel_handle_v1::{self, ExtForeignToplevelHandleV1},
    ext_foreign_toplevel_list_v1::{self, ExtForeignToplevelListV1},
};

// User data for toplevel handles
#[derive(Debug)]
pub struct ExtToplevelHandleData {
    pub id: usize,
}

// Global counter for generating toplevel IDs
static NEXT_EXT_TOPLEVEL_ID: AtomicUsize = AtomicUsize::new(2000); // Start from 2000 to distinguish from wlr and treeland

// Dispatch for the foreign toplevel list
impl Dispatch<ExtForeignToplevelListV1, ()> for AppState {
    fn event(
        _state: &mut Self,
        _manager: &ExtForeignToplevelListV1,
        event: ext_foreign_toplevel_list_v1::Event,
        _: &(),
        _: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            ext_foreign_toplevel_list_v1::Event::Toplevel { .. } => {
                // The toplevel handle is created automatically by event_created_child!
                // The ID is generated in event_created_child macro, so we don't create toplevel here
                // It will be created when we receive the first event from the handle
            }
            ext_foreign_toplevel_list_v1::Event::Finished => {
                // List is being destroyed
            }
            _ => {}
        }
    }

    wayland_client::event_created_child!(AppState, ExtForeignToplevelListV1, [
        ext_foreign_toplevel_list_v1::EVT_TOPLEVEL_OPCODE => (ExtForeignToplevelHandleV1, ExtToplevelHandleData {
            id: NEXT_EXT_TOPLEVEL_ID.fetch_add(1, Ordering::SeqCst)
        })
    ]);
}

// Dispatch for individual toplevel handles
impl Dispatch<ExtForeignToplevelHandleV1, ExtToplevelHandleData> for AppState {
    fn event(
        state: &mut Self,
        _handle: &ExtForeignToplevelHandleV1,
        event: ext_foreign_toplevel_handle_v1::Event,
        data: &ExtToplevelHandleData,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let toplevel_id = data.id;

        // Ensure the toplevel exists in our list
        if state.find_toplevel_mut(toplevel_id).is_none() {
            let new_toplevel = Toplevel::new(toplevel_id);
            state.add_toplevel(new_toplevel);
        }

        match event {
            ext_foreign_toplevel_handle_v1::Event::Title { title } => {
                let mode = state.mode;
                if let Some(toplevel) = state.find_toplevel_mut(toplevel_id) {
                    if mode == Mode::Watch || mode == Mode::VerboseWatch {
                        if let Some(ref old_title) = toplevel.title {
                            println!(
                                "toplevel {}: change title: '{}' -> '{}'",
                                toplevel_id, old_title, title
                            );
                        } else {
                            println!("toplevel {}: set title: '{}'", toplevel_id, title);
                        }
                    }
                    toplevel.set_title(title);
                }
            }
            ext_foreign_toplevel_handle_v1::Event::AppId { app_id } => {
                let mode = state.mode;
                if let Some(toplevel) = state.find_toplevel_mut(toplevel_id) {
                    if mode == Mode::Watch || mode == Mode::VerboseWatch {
                        if let Some(ref old_app_id) = toplevel.app_id {
                            println!(
                                "toplevel {}: change app-id: '{}' -> '{}'",
                                toplevel_id, old_app_id, app_id
                            );
                        } else {
                            println!("toplevel {}: set app-id: '{}'", toplevel_id, app_id);
                        }
                    }
                    toplevel.set_app_id(app_id);
                }
            }
            ext_foreign_toplevel_handle_v1::Event::Identifier { identifier } => {
                let mode = state.mode;
                if let Some(toplevel) = state.find_toplevel_mut(toplevel_id) {
                    if mode == Mode::Watch || mode == Mode::VerboseWatch {
                        println!("toplevel {}: set identifier: {}", toplevel_id, identifier);
                    }
                    toplevel.set_identifier(identifier);
                }
            }
            ext_foreign_toplevel_handle_v1::Event::Done => {
                if let Some(toplevel) = state.find_toplevel_mut(toplevel_id) {
                    toplevel.mark_listed();
                }
            }
            ext_foreign_toplevel_handle_v1::Event::Closed => {
                state.remove_toplevel(toplevel_id);
            }
            _ => {}
        }
    }
}
