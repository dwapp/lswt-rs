use crate::cli::Mode;
use crate::protocols::AppState;
use crate::toplevel::{Toplevel, ToplevelState};
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_protocols_wlr::foreign_toplevel::v1::client::{
    zwlr_foreign_toplevel_handle_v1::{self, ZwlrForeignToplevelHandleV1},
    zwlr_foreign_toplevel_manager_v1::{self, ZwlrForeignToplevelManagerV1},
};

// User data for toplevel handles
pub struct ToplevelHandleData {
    pub id: usize,
}

// Dispatch for the foreign toplevel manager
impl Dispatch<ZwlrForeignToplevelManagerV1, ()> for AppState {
    fn event(
        state: &mut Self,
        _manager: &ZwlrForeignToplevelManagerV1,
        event: zwlr_foreign_toplevel_manager_v1::Event,
        _: &(),
        _: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_foreign_toplevel_manager_v1::Event::Toplevel { toplevel: _ } => {
                let id = state.next_toplevel_id();
                let new_toplevel = Toplevel::new(id);
                state.add_toplevel(new_toplevel);

                // Note: The toplevel handle will dispatch its own events
                // We store the ID in user data for the handle
            }
            zwlr_foreign_toplevel_manager_v1::Event::Finished => {}
            _ => {}
        }
    }
}

// Dispatch for individual toplevel handles
impl Dispatch<ZwlrForeignToplevelHandleV1, ToplevelHandleData> for AppState {
    fn event(
        state: &mut Self,
        _handle: &ZwlrForeignToplevelHandleV1,
        event: zwlr_foreign_toplevel_handle_v1::Event,
        data: &ToplevelHandleData,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let toplevel_id = data.id;

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
                let mut new_state = ToplevelState::default();

                // Parse state array
                for chunk in state_array.chunks(4) {
                    if chunk.len() == 4 {
                        let state_val =
                            u32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                        match state_val {
                            0 => new_state.maximized = true,  // MAXIMIZED
                            1 => new_state.minimized = true,  // MINIMIZED
                            2 => new_state.activated = true,  // ACTIVATED
                            3 => new_state.fullscreen = true, // FULLSCREEN
                            _ => {}
                        }
                    }
                }

                if mode == Mode::VerboseWatch {
                    println!(
                        "toplevel {}: fullscreen: {}",
                        toplevel_id, new_state.fullscreen
                    );
                    println!(
                        "toplevel {}: activated (focused): {}",
                        toplevel_id, new_state.activated
                    );
                    println!(
                        "toplevel {}: maximized: {}",
                        toplevel_id, new_state.maximized
                    );
                    println!(
                        "toplevel {}: minimized: {}",
                        toplevel_id, new_state.minimized
                    );
                }

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
