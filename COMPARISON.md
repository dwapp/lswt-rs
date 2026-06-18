# Comparison: lswt vs lswt-rs

This document compares the original C implementation (lswt) with the Rust implementation (lswt-rs).

## Implementation Overview

| Aspect | lswt (C) | lswt-rs (Rust) |
|--------|----------|----------------|
| **Language** | C | Rust |
| **Lines of Code** | ~1500 lines | ~800 lines (modular structure) |
| **Memory Safety** | Manual management | Automatic (borrow checker) |
| **Dependencies** | libwayland-client | wayland-client, wayland-protocols, wayland-protocols-wlr |
| **Build System** | Make + wayland-scanner | Cargo |

## Features Comparison

### Supported Features

| Feature | lswt | lswt-rs | Notes |
|---------|------|---------|-------|
| List toplevels | ✅ | ✅ | Both support basic listing |
| JSON output | ✅ | ✅ | Compatible format |
| Custom format | ✅ | ✅ | Same format string syntax |
| Watch mode | ✅ | ✅ | Real-time monitoring |
| Verbose watch | ✅ | ✅ | Detailed state changes |
| zwlr protocol | ✅ | ✅ | wlroots foreign-toplevel v1 |
| ext protocol | ✅ | 🚧 | Planned (waiting for library support) |
| Force protocol | ✅ | ✅ | Select specific protocol |
| Bash completion | ✅ | ⏳ | TODO |
| Man page | ✅ | ⏳ | TODO |
| Landlock sandbox | ✅ | ⏳ | TODO |

### Protocol Support

Both implementations support:
- `zwlr_foreign_toplevel_management_unstable_v1` (version 3+)

The C version also supports:
- `ext_foreign_toplevel_list_v1` (standardized protocol)

The Rust version will add ext protocol support when it becomes available in upstream wayland-protocols crates.

### Output Compatibility

The JSON output format (v2) is compatible between both implementations:

```json
{
  "json-output-version": 2,
  "supported-data": {
    "title": true,
    "app-id": true,
    "identifier": bool,
    "fullscreen": bool,
    "activated": bool,
    "minimized": bool,
    "maximized": bool
  },
  "toplevels": [...]
}
```

Custom format strings use the same syntax:
- `t` - title
- `a` - app-id  
- `i` - identifier
- `A` - activated
- `f` - fullscreen
- `m` - minimized
- `M` - maximized

## Architecture Differences

### C Implementation (lswt)

```
lswt.c                     # Monolithic ~1500 lines
├── Protocol handlers      # Inline callbacks
├── Data structures        # Manual memory management
├── Output formatters      # Integrated
└── Main loop             # Signal handling with longjmp
```

### Rust Implementation (lswt-rs)

```
src/
├── main.rs               # Entry point
├── cli.rs                # Argument parsing (clap)
├── toplevel.rs           # Data structures
├── output.rs             # Output formatting
└── protocols/
    ├── mod.rs            # State management
    ├── wlr_foreign_toplevel.rs    # Protocol implementation
    └── ext_foreign_toplevel.rs    # (placeholder)
```

## Safety & Error Handling

### Memory Safety

| Aspect | lswt (C) | lswt-rs (Rust) |
|--------|----------|----------------|
| Buffer overflows | Possible | Prevented at compile-time |
| Use-after-free | Possible | Prevented by borrow checker |
| NULL derefs | Runtime checks | Option<T> type system |
| Memory leaks | Manual tracking | Automatic Drop trait |

### Error Handling

- **lswt**: Uses errno, fprintf, exit codes
- **lswt-rs**: Uses Result<T, E> with anyhow for error propagation

## Performance

Both implementations should have similar performance characteristics:
- Wayland protocol communication is the bottleneck
- Minimal CPU usage when idle in watch mode
- Memory usage is comparable (few KB per toplevel)

## Building & Installation

### lswt (C)
```bash
make
sudo make install
```

### lswt-rs (Rust)
```bash
cargo build --release
sudo make install
# or
cargo install --path .
```

## Future Plans

### Short-term
- ✅ Core zwlr protocol support
- ✅ All output formats
- ✅ Watch modes

### Medium-term
- 🔄 ext-foreign-toplevel-list-v1 support
- 📝 Man page generation
- 🐚 Shell completion generation

### Long-term
- 🔒 Optional sandboxing (seccomp/landlock)
- 📦 Package distribution (AUR, crates.io)
- 🧪 Integration tests

## Compatibility Notes

lswt-rs aims to be a drop-in replacement for lswt in most use cases:

✅ **Compatible:**
- Command-line flags (except those using protocols not yet supported)
- JSON output format
- Custom format strings
- Normal output format
- Exit codes

⚠️ **Differences:**
- Different binary name by default (`lswt` in both cases after installation)
- Error messages may be worded differently
- Some advanced C-specific features (landlock) not yet implemented

## Migration Guide

If you're using lswt in scripts, lswt-rs should work as a drop-in replacement:

```bash
# All of these work the same:
lswt
lswt --json
lswt --custom "ta"
lswt --watch
```

The only scenarios where you might see differences:
1. Using `ext-foreign-toplevel-list-v1` protocol specifically (not yet supported in Rust version)
2. Relying on specific error message text (wording may differ)
3. Using landlock sandboxing features (not yet implemented)

## Contributing

Both projects welcome contributions:
- **lswt**: https://sr.ht/~leon_plickat/lswt/
- **lswt-rs**: Contributions welcome via pull requests

## License

Both implementations are licensed under GPL-3.0-or-later.
