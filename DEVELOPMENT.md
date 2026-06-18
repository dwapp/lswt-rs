# Development Guide

## Project Structure

```
lswt-rs/
├── src/
│   ├── main.rs              # Application entry point
│   ├── cli.rs               # Command-line argument parsing
│   ├── toplevel.rs          # Toplevel window data structures
│   ├── output.rs            # Output formatting (normal, JSON, custom)
│   └── protocols/
│       ├── mod.rs           # Application state & registry handling
│       ├── wlr_foreign_toplevel.rs   # wlroots protocol implementation
│       └── ext_foreign_toplevel.rs   # (future) ext protocol
├── Cargo.toml               # Dependencies and metadata
├── README.md                # User documentation
├── COMPARISON.md            # Comparison with original lswt
└── DEVELOPMENT.md           # This file

```

## Building

### Development Build
```bash
cargo build
./target/debug/lswt --help
```

### Release Build
```bash
cargo build --release
./target/release/lswt --help
```

### Installation
```bash
# Using make
make install

# Or using cargo
cargo install --path .
```

## Dependencies

Key dependencies:
- **clap** (4.5): Command-line argument parsing with derive macros
- **wayland-client** (0.31): Wayland client protocol implementation
- **wayland-protocols-wlr** (0.3): wlroots protocol extensions
- **serde** & **serde_json**: JSON serialization
- **anyhow**: Error handling

## Code Organization

### main.rs
- Entry point
- Creates AppState
- Runs event loop
- Outputs results

### cli.rs
- Defines command-line arguments using clap
- Validates custom format strings
- Determines operating mode (list/watch/verbose-watch)

### toplevel.rs
- `Toplevel` struct: Represents a window
- `ToplevelState` struct: Window state (maximized, minimized, etc.)
- Methods for setting properties

### output.rs
- `OutputWriter`: Handles all output formatting
- Three formats: Normal (human-readable), JSON, Custom
- Handles quoting, escaping, and padding

### protocols/mod.rs
- `AppState`: Main application state
- Implements `Dispatch` for `wl_registry` events
- Binds to zwlr_foreign_toplevel_manager_v1
- Manages toplevel list

### protocols/wlr_foreign_toplevel.rs
- Implements `Dispatch` for wlroots protocol
- Handles manager events (toplevel creation)
- Handles toplevel events (title, app-id, state changes)

## Wayland Protocol Flow

1. **Connect** to Wayland display
2. **Get registry** and bind to protocols
3. **First roundtrip**: Receive global objects
4. **Bind manager**: zwlr_foreign_toplevel_manager_v1
5. **Second roundtrip**: Receive toplevel handles and their events
6. **Output** or **watch** for changes

## Adding New Features

### Adding a new output format

1. Add variant to `OutputFormat` enum in `cli.rs`
2. Add command-line flag in `Args` struct
3. Implement formatting in `output.rs` `write_toplevels()` match statement

### Adding ext-foreign-toplevel-list-v1 support

When the protocol becomes available in wayland-protocols:

1. Add dependency to `Cargo.toml`
2. Update `protocols/mod.rs` registry handler to bind ext protocol
3. Implement `Dispatch` traits in `protocols/ext_foreign_toplevel.rs`
4. Update `UsedProtocol` logic to prefer ext over wlr

## Testing

### Manual Testing
```bash
# In a Wayland session (Sway, Hyprland, etc.)
cargo run

# Test JSON output
cargo run -- --json | jq

# Test custom format
cargo run -- --custom "ta"

# Test watch mode (Ctrl+C to exit)
cargo run -- --watch
```

### With Debug Output
```bash
RUST_LOG=debug cargo run
```

## Debugging

### Check Protocol Support
```bash
# See what protocols your compositor supports
wayland-info | grep -i foreign
```

### Verify Binary Works
```bash
# Build and test
cargo build --release
./target/release/lswt

# Compare with original
/path/to/original/lswt
./target/release/lswt
```

## Performance

### Binary Size
```bash
# Debug build (with symbols)
ls -lh target/debug/lswt

# Release build (optimized, stripped)
ls -lh target/release/lswt
strip target/release/lswt
ls -lh target/release/lswt
```

### Benchmarking
```bash
# Time the execution
time ./target/release/lswt > /dev/null

# Compare with C version
time /path/to/original/lswt > /dev/null
```

## Common Issues

### "Wayland server supports none of the protocol extensions"

Your compositor doesn't support zwlr-foreign-toplevel-management. Try:
- Sway (has support)
- Hyprland (has support)
- Other wlroots-based compositors

### Compilation errors with wayland-protocols

Make sure you're using compatible versions:
```bash
cargo update
cargo clean
cargo build
```

## Contributing

Areas that need work:
- [ ] ext-foreign-toplevel-list-v1 support
- [ ] Man page generation
- [ ] Shell completion generation (bash, zsh, fish)
- [ ] Integration tests
- [ ] CI/CD pipeline
- [ ] Sandboxing support (landlock/seccomp)
- [ ] Better error messages

## Resources

- [Wayland Protocol Documentation](https://wayland.freedesktop.org/docs/html/)
- [wlroots Protocols](https://gitlab.freedesktop.org/wlroots/wlr-protocols)
- [wayland-rs Book](https://smithay.github.io/wayland-rs/)
- [Original lswt](https://sr.ht/~leon_plickat/lswt/)
