# Rustica Apps Directory Structure

```
apps/
├── cli/                  # CLI utilities (minimal, essential)
│   ├── redit/            # text editor
│   │   ├── src/
│   │   ├── Cargo.toml
│   │   └── tests/
│   ├── net-tools/        # ping, ip, ifconfig, etc.
│   ├── sys-tools/        # uname, ps, kill, etc.
│   └── build.rs
│
├── gui/                  # GUI / desktop apps (Aurora + future apps)
│   ├── aurora-shell/
│   ├── aurora-panel/
│   └── aurora-launcher/
│
├── libs/                 # shared Rust libraries for apps
│   ├── rutils/           # Rustica utilities library
│   ├── rgui/             # GUI helpers for Aurora
│   └── netlib/           # networking utilities
│
├── examples/             # example applications / demos
│
├── tests/                # integration / CI tests
│
├── scripts/              # build scripts, deployment helpers
│   ├── build-all.sh
│   └── package.sh
│
├── Cargo.toml            # workspace manifest
└── README.md
```

## Component Overview

### CLI Applications (`cli/`)
- **redit**: Minimal, modal text editor for Rustux
- **net-tools**: Networking utilities (ping, ip, ifconfig, netstat, etc.)
- **sys-tools**: System utilities (uname, ps, kill, top, etc.)

### GUI Applications (`gui/`)
- **aurora-shell**: Main Aurora desktop shell/compositor
- **aurora-panel**: Desktop panel (taskbar, system tray)
- **aurora-launcher**: Application launcher menu

### Libraries (`libs/`)
- **rutils**: Shared utilities and common functions
- **rgui**: GUI framework and helpers for Aurora apps
- **netlib**: Networking abstraction layer

### Build System
- `scripts/build-all.sh`: Build all apps in dependency order
- `scripts/package.sh`: Package apps for distribution
- Root `Cargo.toml`: Workspace configuration
