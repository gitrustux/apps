# Process Model & Security Architecture Decision

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Component**: Rustica Shell - Security & Process Isolation

## Decision

**CHOSEN: Hybrid Security Model with Wayland Compositor as Security Boundary + Optional Sandboxing**

We will implement a **layered security model** that leverages the RUSTUX kernel's capability system while maintaining compatibility with standard Linux security mechanisms.

### Security Layers

```
┌────────────────────────────────────────────────────────────────┐
│                     User Application                           │
│                  (Untrusted or Trusted)                        │
└────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌────────────────────────────────────────────────────────────────┐
│               Rustica Compositor (Security Boundary)           │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    Wayland Protocol                       │  │
│  │   - All input/output mediated                            │  │
│  │   - No direct hardware access                            │  │
│  │   - No access to other app data                          │  │
│  └──────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │              RUSTUX Capability Bridge                     │  │
│  │   - Request GPU capability from kernel                   │  │
│  │   - Enforce per-app capability limits                     │  │
│  │   - Sandbox enforcement                                  │  │
│  └──────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌────────────────────────────────────────────────────────────────┐
│                    RUSTUX Microkernel                         │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │              Capability-Based Security                    │  │
│  │   - Object permissions                                   │  │
│  │   - IPC mediation                                        │  │
│  │   - Resource limits                                      │  │
│  └──────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────┘
```

## Process Isolation Strategy

### Tier 1: Wayland Security (Base Level)
All applications get:
- **No direct hardware access** (no DRM, no input devices)
- **Mediated input/output** (through compositor)
- **No screen scraping** (can't read other windows)
- **No input interception** (can't spy on other apps)

This is standard Wayland security and protects against basic attacks.

### Tier 2: RUSTUX Capability Enforcement
Applications request capabilities from kernel:
- **GPU rendering capability** (with memory limits)
- **Network capability** (if needed)
- **File system access** (with path restrictions)
- **D-Bus/service access** (with filtering)

Compositor acts as capability broker:
```rust
// App requests GPU access via compositor
fn request_gpu_access(pid: Pid) -> Result<GpuCapability> {
    // Ask kernel if this process can have GPU access
    let cap = kernel::request_capability(
        pid,
        Capability {
            type: CapabilityType::GpuRendering,
            limits: Limits {
                memory_mb: 512,
                max_surfaces: 16,
            },
        }
    )?;

    Ok(cap)
}
```

### Tier 3: Optional Sandboxing (For Untrusted Apps)
For apps from unknown sources (e.g., Flatpak, third-party):
- **seccomp filters** - restrict syscalls
- **namespace isolation** - separate filesystem, network, PID namespaces
- **resource limits** - CPU, memory, I/O throttling
- **filesystem sandbox** - read-only except explicit paths

## IPC Architecture

### Application ↔ Compositor IPC
```rust
// Wayland protocol (secure by design)
// - All messages mediated by compositor
// - No direct peer-to-peer communication

// For cross-app communication, use:
// 1. D-Bus (session bus) - mediated by bus daemon
// 2. Pipe/socket through compositor - with capability checks
// 3. Shared memory (DMABUF) - with compositor approval
```

### Compositor ↔ Kernel IPC
```rust
// Custom kernel IPC protocol for GUI
pub enum GuiRequest {
    // GPU access
    RequestGpu { pid: Pid, limits: GpuLimits },

    // Input device access
    RequestInputDevice { pid: Pid, device: DeviceId },

    // Display control
    CreateSurface { pid: Pid, config: SurfaceConfig },
    DestroySurface { pid: Pid, surface: SurfaceId },

    // Capability queries
    HasCapability { pid: Pid, capability: Capability },
}

pub enum GuiResponse {
    Granted { capability: Capability },
    Denied { reason: DenialReason },
    Error { message: String },
}
```

## Security Model

### Threat Model

| Attacker | Attack Vector | Mitigation |
|----------|--------------|------------|
| **Malicious App** | Keylogging, screen scraping | Wayland mediation, no raw input |
| **Complicated App** | Privilege escalation | Capability system, kernel enforcement |
| **Network App** | Data exfiltration | Explicit network capability |
| **Sandbox Escape** | Break out of sandbox | seccomp, namespace isolation |

### Capability Examples

```rust
// Trusted system app (e.g., file manager)
let file_manager_caps = vec![
    Capability::FileSystem {
        paths: vec!["/home/*".into()],
        write: true,
    },
    Capability::Network,
    Capability::GpuRendering {
        memory_mb: 1024,
    },
];

// Untrusted app from store
let untrusted_caps = vec![
    Capability::GpuRendering {
        memory_mb: 256,  // Limited
    },
    Capability::Sandboxed,  // Full sandbox
    // No network, no filesystem
];
```

## Sandbox Specification

### seccomp Filter
```rust
// Allowed syscalls for sandboxed app
const ALLOWED_SYSCALLS: &[i64] = &[
    // Basic I/O
    libc::SYS_read, libc::SYS_write, libc::SYS_poll,

    // Memory
    libc::SYS_mmap, libc::SYS_munmap, libc::SYS_brk,

    // Process (limited)
    libc::SYS_exit, libc::SYS_exit_group,

    // Wayland (only)
    libc::SYS_sendmsg, libc::SYS_recvmsg,
    libc::SYS_connect, libc::SYS_socket,

    // Nothing else!
];

// Blocked: execve, fork, clone, openat, etc.
```

### Namespace Isolation
```rust
pub struct SandboxConfig {
    // Filesystem namespace
    pub mount_ns: bool,        // Separate mount namespace
    pub readonly_paths: Vec<PathBuf>,  // Read-only mounts
    pub readwrite_paths: Vec<PathBuf>,  // Writable dirs

    // Network namespace
    pub network_ns: bool,       // Separate network namespace

    // IPC namespace
    pub ipc_ns: bool,           // Separate IPC namespace

    // PID namespace
    pub pid_ns: bool,           // Separate PID namespace
}
```

## Resource Limits

```rust
pub struct ResourceLimits {
    // Memory limits
    pub max_memory_mb: usize,
    pub max_gpus_mb: usize,

    // CPU limits
    pub max_cpu_percent: u8,
    pub max_threads: usize,

    // I/O limits
    pub max_io_mb_per_sec: usize,

    // Time limits (optional)
    pub max_cpu_time_secs: Option<usize>,
}
```

## Implementation Phases

### Phase 1: Base Security (Weeks 1-4)
- [ ] Wayland mediation (automatic via Smithay)
- [ ] GPU capability requests
- [ ] Basic capability checking

### Phase 2: Capability Integration (Weeks 5-8)
- [ ] RUSTUX kernel IPC bridge
- [ ] Capability granting/revoking
- [ ] Per-app capability profiles

### Phase 3: Sandboxing (Weeks 9+)
- [ ] seccomp filter generation
- [ ] Namespace setup
- [ ] Resource limit enforcement
- [ ] Sandbox escape testing

## File Structure

```
/var/www/rustux.com/prod/
├── kernel/src/gui/
│   ├── mod.rs                # GUI IPC module
│   ├── capability.rs         # Capability requests
│   └── security.rs           # Security enforcement
│
├── apps/gui/rustica-comp/src/
│   ├── security/
│   │   ├── mod.rs
│   │   ├── capability.rs     # Capability broker
│   │   ├── sandbox.rs        # Sandbox setup
│   │   └── ipc.rs            # Kernel IPC
│   └── ...
│
└── libs/librustica/src/
    └── sandbox/
        ├── mod.rs
        ├── seccomp.rs        # seccomp filters
        ├── namespace.rs      # Namespace helpers
        └── profile.rs        # Sandbox profiles
```

## Security Best Practices

### For App Developers
```rust
// 1. Request minimal capabilities
let app = Application::new()
    .capability(Capability::Gpu { memory_mb: 256 })
    // Don't request network if not needed!

// 2. Run in sandbox during development
app.sandbox(SandboxProfile::strict());

// 3. Validate all input
app.on_input(|input| {
    validate(input)?;
    // ...
});

// 4. Use secure IPC
app.use_dbus(SessionBus, |bus| {
    // Only talk to trusted services
});
```

### For Compositor
```rust
// 1. Validate all requests
fn handle_request(request: Request) -> Result<()> {
    validate_request(&request)?;
    // ...
}

// 2. Enforce timeouts
compositor.set_app_timeout(Duration::from_secs(5));

// 3. Monitor resource usage
compositor.watch_app_limits();

// 4. Kill misbehaving apps
if app.exceeds_limits() {
    compositor.terminate_app(app.id());
}
```

## Success Criteria

- [ ] Wayland mediation prevents screen scraping
- [ ] Capability system enforces GPU/memory limits
- [ ] Sandbox isolates untrusted apps
- [ ] No privilege escalation possible
- [ ] Performance impact <5% overhead

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Sandbox too restrictive | Provide capability requests, profile system |
| Compatibility issues | Use sandbox for new apps only, opt-in for existing |
| Performance overhead | Lazy sandboxing, only for untrusted apps |
| Complexity | Start without sandboxing, add incrementally |

## Sign-Off

**Architect**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅

---

## References

- [Wayland Security](https://wayland.freedesktop.org/docs/html/ch03.html)
- [Linux Capabilities](https://man7.org/linux/man-pages/man7/capabilities.7.html)
- [seccomp](https://man7.org/linux/man-pages/man2/seccomp.2.html)
- [Linux Namespaces](https://man7.org/linux/man-pages/man7/namespaces.7.html)
- [Flatpak Sandboxing](https://docs.flatpak.org/en/latest/sandbox-permissions.html)
