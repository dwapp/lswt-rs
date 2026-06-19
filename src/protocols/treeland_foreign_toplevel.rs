use crate::cli::Mode;
use crate::protocols::AppState;
use crate::protocols::common::{parse_state_array, print_state_change};
use crate::toplevel::Toplevel;
use std::sync::atomic::{AtomicUsize, Ordering};
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_protocols_treeland::foreign_toplevel_manager::v1::client::{
    treeland_foreign_toplevel_handle_v1::{self, TreelandForeignToplevelHandleV1},
    treeland_foreign_toplevel_manager_v1::{self, TreelandForeignToplevelManagerV1},
};

// User data for toplevel handles
#[derive(Debug)]
pub struct TreelandToplevelHandleData {
    pub id: usize,
}

// Global counter for generating toplevel IDs (shared with manager event)
static NEXT_TREELAND_TOPLEVEL_ID: AtomicUsize = AtomicUsize::new(1000); // Start from 1000 to distinguish from wlr

// Dispatch for the foreign toplevel manager
impl Dispatch<TreelandForeignToplevelManagerV1, ()> for AppState {
    fn event(
        _state: &mut Self,
        _manager: &TreelandForeignToplevelManagerV1,
        event: treeland_foreign_toplevel_manager_v1::Event,
        _: &(),
        _: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            treeland_foreign_toplevel_manager_v1::Event::Toplevel { .. } => {
                // The toplevel handle is created automatically by event_created_child!
                // The ID is generated in event_created_child macro, so we don't create toplevel here
                // It will be created when we receive the first event from the handle
            }
            treeland_foreign_toplevel_manager_v1::Event::Finished => {
                // Manager is being destroyed
            }
            _ => {}
        }
    }

    wayland_client::event_created_child!(AppState, TreelandForeignToplevelManagerV1, [
        treeland_foreign_toplevel_manager_v1::EVT_TOPLEVEL_OPCODE => (TreelandForeignToplevelHandleV1, TreelandToplevelHandleData {
            id: NEXT_TREELAND_TOPLEVEL_ID.fetch_add(1, Ordering::SeqCst)
        })
    ]);
}

// Dispatch for individual toplevel handles
impl Dispatch<TreelandForeignToplevelHandleV1, TreelandToplevelHandleData> for AppState {
    fn event(
        state: &mut Self,
        _handle: &TreelandForeignToplevelHandleV1,
        event: treeland_foreign_toplevel_handle_v1::Event,
        data: &TreelandToplevelHandleData,
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
            treeland_foreign_toplevel_handle_v1::Event::Title { title } => {
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
            treeland_foreign_toplevel_handle_v1::Event::AppId { app_id } => {
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
            treeland_foreign_toplevel_handle_v1::Event::Identifier { identifier } => {
                let mode = state.mode;
                if let Some(toplevel) = state.find_toplevel_mut(toplevel_id) {
                    if mode == Mode::Watch || mode == Mode::VerboseWatch {
                        println!("toplevel {}: set identifier: {}", toplevel_id, identifier);
                    }
                    toplevel.set_identifier(identifier.to_string());
                }
            }
            treeland_foreign_toplevel_handle_v1::Event::Pid { pid } => {
                let mode = state.mode;
                if mode == Mode::VerboseWatch {
                    println!("toplevel {}: set pid: {}", toplevel_id, pid);
                }
            }
            treeland_foreign_toplevel_handle_v1::Event::State { state: state_array } => {
                let mode = state.mode;
                let new_state = parse_state_array(&state_array);

                // Handle ATTENTION state (value 4) specific to Treeland
                for chunk in state_array.chunks(4) {
                    if chunk.len() == 4 {
                        let state_val =
                            u32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                        if state_val == 4 && mode == Mode::VerboseWatch {
                            println!("toplevel {}: attention: true", toplevel_id);
                        }
                    }
                }

                print_state_change(toplevel_id, &new_state, mode);

                if let Some(toplevel) = state.find_toplevel_mut(toplevel_id) {
                    toplevel.set_state(new_state);
                }
            }
            treeland_foreign_toplevel_handle_v1::Event::Done => {
                if let Some(toplevel) = state.find_toplevel_mut(toplevel_id) {
                    toplevel.mark_listed();
                }
            }
            treeland_foreign_toplevel_handle_v1::Event::Closed => {
                state.remove_toplevel(toplevel_id);
            }
            treeland_foreign_toplevel_handle_v1::Event::OutputEnter { .. } => {
                // We don't track which outputs a toplevel is on
            }
            treeland_foreign_toplevel_handle_v1::Event::OutputLeave { .. } => {
                // We don't track which outputs a toplevel is on
            }
            treeland_foreign_toplevel_handle_v1::Event::Parent { .. } => {
                // We don't track parent relationships
            }
            _ => {}
        }
    }
}
