use crate::cli::Mode;
use crate::protocols::common::{parse_state_array, print_state_change};
use crate::protocols::AppState;
use crate::toplevel::{Toplevel, ToplevelHandle};
use std::sync::atomic::{AtomicUsize, Ordering};
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_protocols_wlr::foreign_toplevel::v1::client::{
    zwlr_foreign_toplevel_handle_v1::{self, ZwlrForeignToplevelHandleV1},
    zwlr_foreign_toplevel_manager_v1::{self, ZwlrForeignToplevelManagerV1},
};

// User data for toplevel handles
#[derive(Debug)]
pub struct ToplevelHandleData {
    pub id: usize,
}

// Global counter for generating toplevel IDs
static NEXT_TOPLEVEL_ID: AtomicUsize = AtomicUsize::new(0);

// Dispatch for the foreign toplevel manager
impl Dispatch<ZwlrForeignToplevelManagerV1, ()> for AppState {
    fn event(
        _state: &mut Self,
        _manager: &ZwlrForeignToplevelManagerV1,
        event: zwlr_foreign_toplevel_manager_v1::Event,
        _: &(),
        _: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_foreign_toplevel_manager_v1::Event::Toplevel { .. } => {
                // The toplevel handle is created automatically by event_created_child!
                // It will be created when we receive the first event from the handle.
            }
            zwlr_foreign_toplevel_manager_v1::Event::Finished => {}
            _ => {}
        }
    }

    wayland_client::event_created_child!(AppState, ZwlrForeignToplevelManagerV1, [
        zwlr_foreign_toplevel_manager_v1::EVT_TOPLEVEL_OPCODE => (ZwlrForeignToplevelHandleV1, ToplevelHandleData {
            id: NEXT_TOPLEVEL_ID.fetch_add(1, Ordering::SeqCst)
        })
    ]);
}

// Dispatch for individual toplevel handles
impl Dispatch<ZwlrForeignToplevelHandleV1, ToplevelHandleData> for AppState {
    fn event(
        state: &mut Self,
        handle: &ZwlrForeignToplevelHandleV1,
        event: zwlr_foreign_toplevel_handle_v1::Event,
        data: &ToplevelHandleData,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let toplevel_id = data.id;
        let handle_idx = toplevel_id; // Use toplevel_id as index

        // Ensure the toplevel exists in our list
        if state.find_toplevel_mut(toplevel_id).is_none() {
            let new_toplevel = Toplevel::new(toplevel_id);
            state.add_toplevel(new_toplevel);
        }

        // Save handle if not already saved
        while state.handles.len() <= handle_idx {
            state.handles.push(None);
        }
        if state.handles[handle_idx].is_none() {
            state.handles[handle_idx] = Some(ToplevelHandle::Wlr(handle.clone()));
            // Update toplevel with handle_id
            if let Some(toplevel) = state.find_toplevel_mut(toplevel_id) {
                toplevel.handle_id = Some(handle_idx);
            }
        }

        match event {
            zwlr_foreign_toplevel_handle_v1::Event::Title { title } => {
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
            zwlr_foreign_toplevel_handle_v1::Event::AppId { app_id } => {
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
            zwlr_foreign_toplevel_handle_v1::Event::State { state: state_array } => {
                let mode = state.mode;
                let new_state = parse_state_array(&state_array);
                print_state_change(toplevel_id, &new_state, mode);

                if let Some(toplevel) = state.find_toplevel_mut(toplevel_id) {
                    toplevel.set_state(new_state);
                }
            }
            zwlr_foreign_toplevel_handle_v1::Event::Done => {
                if let Some(toplevel) = state.find_toplevel_mut(toplevel_id) {
                    toplevel.mark_listed();
                }
            }
            zwlr_foreign_toplevel_handle_v1::Event::Closed => {
                state.remove_toplevel(toplevel_id);
            }
            _ => {}
        }
    }
}
