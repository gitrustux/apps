# Display Stack Architecture Decision

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Component**: Rustica Shell - Display Server

## Decision

**CHOSEN: Wayland Protocol with Custom Extensions**

We will use the **Wayland display protocol** as the foundation for Rustica Shell, with the following implementation strategy:

### Core Protocol
- **Wayland 1.22+** protocol for display server communication
- **XDG Shell** protocol for window management
- **Layer Shell** protocol for panels, popovers, and lock screens
- **Presentation Time** protocol for smooth animations
- **Input Method** protocol (v2) for IME support

### Custom Extensions
We will implement Rustica-specific Wayland protocols for:

1. **rustica_security_layer** - Capability-based security integration
   - Pass RUSTUX kernel capabilities to compositor
   - Request GPU/rendering capabilities
   - Enforce sandbox boundaries

2. **rustica_mobile** - Mobile-specific extensions
   - Touch gesture protocols
   - On-screen keyboard integration
   - Screen rotation events
   - Mobile status bar integration

3. **rustica_workspace** - Enhanced workspace management
   - Workspace lifecycle events
   - Window assignment notifications
   - Workspace overview/expose mode triggers

### Protocol Wire Format
- Binary protocol following Wayland marshaling rules
- XML protocol description files (like standard Wayland protocols)
- Protocol versioning for backward compatibility

### Compatibility Requirements
- **No X11 compatibility** - Wayland-only for modern, clean design
- **Standard XDG shell** - Enable third-party Wayland apps to run
- **libwayland** client library compatibility

## Rationale

### Why Wayland?

1. **Modern Architecture**
   - Built-in security through compositor mediation
   - No X11 legacy baggage
   - Clear separation of concerns

2. **Ecosystem Support**
   - Widely adopted by modern desktops (GNOME, KDE, COSMIC, sway, etc.)
   - Extensive third-party application support
   - Well-documented protocol specifications

3. **Performance**
   - Direct rendering (no compositing manager needed)
   - Lower latency than X11
   - Efficient for mobile/embedded use cases

4. **Security**
   - All input/output mediated by compositor
   - No random clients can read input or screen contents
   - Fits our capability-based security model

### Why Not Alternatives?

| Alternative | Rejection Reason |
|-------------|------------------|
| **X11** | Legacy architecture, security issues, being phased out |
| **Direct Framebuffer** | No multi-window support, no hardware acceleration, limited |
| **Custom Protocol** | No third-party app support, maintenance burden, reinventing the wheel |

## Implementation Strategy

### Phase 1: Core Compositor (Weeks 1-6)
- Use **Smithay 0.18+** as the Wayland compositor framework
- Implement XDG shell protocol support
- Basic window management (tiling, stacking, fullscreen)
- Input handling (keyboard, mouse, basic touch)

### Phase 2: Custom Protocols (Weeks 7-10)
- Implement rustica_security_layer protocol
- Implement rustica_mobile protocol
- Implement rustica_workspace protocol
- Integrate with RUSTUX kernel IPC

### Phase 3: Integration (Weeks 11+)
- Desktop shell components
- Application toolkit integration
- Testing and optimization

## Reference Implementations

### COSMIC Desktop
- Uses Smithay framework
- Implements custom protocols for COSMIC-specific features
- Wayland-only (no X11)
- https://github.com/pop-os/cosmic-comp

### Smithay
- Pure Rust Wayland compositor framework
- Active development
- Supports all Wayland protocols
- https://github.com/Smithay/smithay

### wlroots
- C-based Wayland compositor library
- Used by sway, Wayfire, etc.
- We use Smithay instead to stay in Rust

## Dependencies

### System Libraries
```
libwayland-dev      # Wayland client library
libwayland-protocols  # Standard protocol definitions
libxkbcommon-dev    # Keyboard handling
libinput-dev        # Input device handling
libseat-dev         # Session management
libdrm-dev          # Direct rendering manager
mesa/libegl1-mesa-dev # OpenGL/EGL
libgbm-dev          # Buffer management
```

### Rust Crates
```toml
[dependencies]
smithay = { version = "0.18", features = ["use_system_lib"] }
smithay-egl = "0.18"
wayland-server = "0.31"
wayland-sys = "0.31"
```

## File Locations

### Protocol Definitions
```
/var/www/rustux.com/prod/apps/gui/protocols/
├── rustica-security.xml      # Security layer protocol
├── rustica-mobile.xml          # Mobile extensions
├── rustica-workspace.xml       # Workspace management
└── README.md                   # Protocol documentation
```

### Compositor Implementation
```
/var/www/rustux.com/prod/apps/gui/rustica-comp/
├── protocols/                   # Generated protocol code
│   ├── rustica_security.rs
│   ├── rustica_mobile.rs
│   └── rustica_workspace.rs
├── src/
│   ├── main.rs                  # Compositor entry point
│   ├── compositor.rs            # Main compositor struct
│   ├── shell.rs                 # XDG shell implementation
│   ├── input.rs                 # Input handling
│   ├── rendering.rs             # Rendering backend
│   └── security.rs              # RUSTUX kernel security integration
```

## Success Criteria

- [ ] Compositor can launch and display a simple test window
- [ ] Standard Wayland apps (e.g., weston-terminal) run successfully
- [ ] Touch input works alongside mouse/keyboard
- [ ] Custom protocols (security, mobile, workspace) are functional
- [ ] Performance target: <16ms per frame (60 FPS)

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Smithay learning curve | COSMIC implementation as reference |
| Protocol complexity | Start with standard protocols, add custom later |
| Hardware compatibility | Test on multiple GPUs (Intel, AMD, NVIDIA) |
| Mobile-specific features | Design mobile extensions from the start |

## Sign-Off

**Architect**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅

---

## References

- [Wayland Protocol](https://wayland.freedesktop.org/)
- [Smithay Documentation](https://docs.rs/smithay/)
- [COSMIC Compositor](https://github.com/pop-os/cosmic-comp)
- [Wayland Protocol Documentation](https://wayland.freedesktop.org/docs/html/)
