# lswt-rs

A Rust implementation of [lswt](https://sr.ht/~leon_plickat/lswt/) - list Wayland toplevels.

## Features

- List all Wayland toplevel windows with their titles and app-ids
- Support for multiple Wayland protocols:
  - `zwlr-foreign-toplevel-management-unstable-v1` (wlroots)
  - `ext-foreign-toplevel-list-v1` (standardized protocol, when available)
- Multiple output formats:
  - Normal (human-readable table)
  - JSON (machine-readable)
  - Custom format (user-defined fields)
- Watch mode for monitoring window changes in real-time
- Display window states (maximized, minimized, activated/focused, fullscreen)

## Requirements

Your Wayland compositor must support at least one of the following protocols:
- `zwlr_foreign_toplevel_management_unstable_v1` version 3 or higher
- `ext_foreign_toplevel_list_v1` version 1 or higher

Most wlroots-based compositors (Sway, river, Hyprland, etc.) support these protocols.

## Installation

### From source

```bash
cargo build --release
sudo cp target/release/lswt /usr/local/bin/
```

## Usage

```bash
# List all toplevels
lswt

# Output in JSON format
lswt --json

# Watch for changes
lswt --watch

# Watch with verbose state information
lswt --verbose-watch

# Custom output format (title,app-id,activated)
lswt --custom taA

# Force specific protocol
lswt --force-protocol zwlr-foreign-toplevel-management-unstable-v1
```

### Options

- `-h, --help` - Print help information
- `-v, --version` - Print version
- `-j, --json` - Output data in JSON format
- `-w, --watch` - Run continuously and log title, identifier and app-id events
- `-W, --verbose-watch` - Like --watch, but also log activated, fullscreen, minimized and maximized state
- `-c <fmt>, --custom <fmt>` - Define a custom line-based output format
- `--force-protocol <name>` - Use specified protocol, do not fall back onto others

### Custom Format Fields

- `t` - title
- `a` - app-id
- `i` - identifier (if supported)
- `A` - activated/focused state
- `f` - fullscreen state
- `m` - minimized state
- `M` - maximized state

## Example Output

### Normal format
```
state:   app-id:             title:
--a-     firefox             Mozilla Firefox
m-a-     kitty               Terminal
----     code                Visual Studio Code
```

### JSON format
```json
{
  "json-output-version": 2,
  "supported-data": {
    "title": true,
    "app-id": true,
    "identifier": false,
    "fullscreen": true,
    "activated": true,
    "minimized": true,
    "maximized": true
  },
  "toplevels": [
    {
      "title": "Mozilla Firefox",
      "app-id": "firefox",
      "activated": true,
      "fullscreen": false,
      "minimized": false,
      "maximized": false
    }
  ]
}
```

## License

GPL-3.0-or-later

This project is a Rust reimplementation inspired by the original C implementation by Leon Henrik Plickat.

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues.
