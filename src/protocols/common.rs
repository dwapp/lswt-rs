use crate::cli::Mode;
use crate::toplevel::ToplevelState;

/// Parse state array from Wayland protocol into ToplevelState
pub fn parse_state_array(state_array: &[u8]) -> ToplevelState {
    let mut new_state = ToplevelState::default();

    for chunk in state_array.chunks(4) {
        if chunk.len() == 4 {
            let state_val = u32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            match state_val {
                0 => new_state.maximized = true,  // MAXIMIZED
                1 => new_state.minimized = true,  // MINIMIZED
                2 => new_state.activated = true,  // ACTIVATED
                3 => new_state.fullscreen = true, // FULLSCREEN
                _ => {}
            }
        }
    }

    new_state
}

/// Print state change in verbose watch mode
pub fn print_state_change(toplevel_id: usize, new_state: &ToplevelState, mode: Mode) {
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
}
