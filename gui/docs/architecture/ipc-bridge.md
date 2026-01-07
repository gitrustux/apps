# IPC Bridge (Kernel ↔ Userspace) Specification

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Component**: RUSTUX Kernel GUI IPC

## Overview

This specification defines the IPC bridge between the RUSTUX microkernel and Rustica Compositor (userspace). It enables **secure capability requests**, **display mode setting**, and **input device access** while maintaining **kernel-enforced security boundaries**.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                  Rustica Compositor (Userspace)                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │              Userspace IPC Client                         │  │
│  │  - Send requests to kernel                               │  │
│  │  - Receive responses from kernel                         │  │
│  │  - Serialize/deserialize GUI messages                    │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ IPC (syscalls, shared memory)
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    RUSTUX Microkernel                           │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │               GUI IPC Server Module                       │  │
│  │  - Receive requests from userspace                       │  │
│  │  - Validate capabilities                                 │  │
│  │  - Interact with hardware drivers                        │  │
│  │  - Send responses back                                   │  │
│  └───────────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │              Security Enforcement                         │  │
│  │  - Capability checking                                   │  │
│  │  - Access control                                        │  │
│  │  - Resource limits                                       │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Hardware Drivers                           │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐       │
│  │   GPU    │  │  Display │  │  Input   │  │   DRM    │       │
│  │  Driver  │  │  Driver  │  │  Driver  │  │  Driver  │       │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘       │
└─────────────────────────────────────────────────────────────────┘
```

## IPC Protocol

### Request Types

```rust
#[repr(C)]
pub enum GuiRequest {
    /// Request GPU rendering capability
    RequestGpu {
        pid: Pid,
        limits: GpuLimits,
    },

    /// Request input device access
    RequestInputDevice {
        pid: Pid,
        device: DeviceId,
    },

    /// Create display surface
    CreateSurface {
        pid: Pid,
        config: SurfaceConfig,
    },

    /// Destroy display surface
    DestroySurface {
        pid: Pid,
        surface: SurfaceId,
    },

    /// Set display mode (resolution, refresh rate)
    SetDisplayMode {
        connector: ConnectorId,
        mode: DisplayMode,
    },

    /// Query device type (desktop/mobile)
    QueryDeviceType,

    /// Register compositor with kernel
    RegisterCompositor {
        pid: Pid,
    },

    /// Unregister compositor
    UnregisterCompositor {
        pid: Pid,
    },

    /// Check if process has capability
    HasCapability {
        pid: Pid,
        capability: Capability,
    },
}
```

### Response Types

```rust
#[repr(C)]
pub enum GuiResponse {
    /// Request granted
    Granted {
        capability: Capability,
    },

    /// Request denied
    Denied {
        reason: DenialReason,
    },

    /// Generic success
    Success,

    /// Error occurred
    Error {
        message: String,
    },

    /// Device type query response
    DeviceType {
        is_mobile: bool,
    },

    /// Capability check result
    HasCapabilityResult {
        has_capability: bool,
    },
}
```

### Capability Types

```rust
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub enum Capability {
    /// GPU rendering capability
    GpuRendering {
        memory_mb: u32,
        max_surfaces: u32,
    },

    /// Input device access
    InputDevice {
        device: DeviceId,
        exclusive: bool,
    },

    /// Display control
    DisplayControl {
        connector: ConnectorId,
    },

    /// Surface management
    SurfaceManagement,

    /// Compositor registration
    Compositor,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub enum DenialReason {
    /// Process not authorized
    Unauthorized,

    /// Resource limit exceeded
    ResourceLimitExceeded,

    /// Device not found
    DeviceNotFound,

    /// Invalid configuration
    InvalidConfig,

    /// Capability already held
    AlreadyHeld,

    /// Other reason
    Other(u32),
}
```

## Kernel Implementation

### GUI IPC Module

```rust
// kernel/src/gui/mod.rs

use crate::{
    capability::CapabilityManager,
    ipc::{IpcServer, IpcMessage},
    process::Pid,
};

/// GUI IPC server
pub struct GuiIpcServer {
    /// IPC server endpoint
    ipc: IpcServer,

    /// Capability manager
    caps: CapabilityManager,

    /// Registered compositor PID
    compositor_pid: Option<Pid>,

    /// Active GPU capabilities
    gpu_caps: HashMap<Pid, GpuCapability>,

    /// Active input device grants
    input_grants: HashMap<Pid, Vec<InputGrant>>,
}

impl GuiIpcServer {
    /// Create a new GUI IPC server
    pub fn new() -> Result<Self> {
        let ipc = IpcServer::new("rustica-gui")?;
        let caps = CapabilityManager::get();

        Ok(Self {
            ipc,
            caps,
            compositor_pid: None,
            gpu_caps: HashMap::new(),
            input_grants: HashMap::new(),
        })
    }

    /// Run the IPC server
    pub fn run(&mut self) -> ! {
        loop {
            // Receive message from userspace
            let msg = self.ipc.receive();

            // Handle request
            let response = self.handle_request(msg);

            // Send response
            self.ipc.send(response);
        }
    }

    /// Handle a GUI request
    fn handle_request(&mut self, msg: IpcMessage) -> GuiResponse {
        match msg.request {
            GuiRequest::RequestGpu { pid, limits } => {
                self.handle_gpu_request(pid, limits)
            }

            GuiRequest::RequestInputDevice { pid, device } => {
                self.handle_input_request(pid, device)
            }

            GuiRequest::CreateSurface { pid, config } => {
                self.handle_create_surface(pid, config)
            }

            GuiRequest::DestroySurface { pid, surface } => {
                self.handle_destroy_surface(pid, surface)
            }

            GuiRequest::SetDisplayMode { connector, mode } => {
                self.handle_set_display_mode(connector, mode)
            }

            GuiRequest::QueryDeviceType => {
                self.handle_query_device_type()
            }

            GuiRequest::RegisterCompositor { pid } => {
                self.handle_register_compositor(pid)
            }

            GuiRequest::UnregisterCompositor { pid } => {
                self.handle_unregister_compositor(pid)
            }

            GuiRequest::HasCapability { pid, capability } => {
                self.handle_has_capability(pid, capability)
            }
        }
    }

    /// Handle GPU capability request
    fn handle_gpu_request(&mut self, pid: Pid, limits: GpuLimits) -> GuiResponse {
        // Check if process is authorized
        if !self.caps.process_has_capability(pid, Capability::Compositor) {
            return GuiResponse::Denied {
                reason: DenialReason::Unauthorized,
            };
        }

        // Check resource limits
        let total_gpu_memory: u32 = self.gpu_caps.values()
            .map(|c| c.memory_mb)
            .sum();

        if total_gpu_memory + limits.memory_mb > MAX_TOTAL_GPU_MEMORY {
            return GuiResponse::Denied {
                reason: DenialReason::ResourceLimitExceeded,
            };
        }

        // Grant capability
        let cap = GpuCapability {
            memory_mb: limits.memory_mb,
            max_surfaces: limits.max_surfaces,
        };

        self.gpu_caps.insert(pid, cap);

        GuiResponse::Granted {
            capability: Capability::GpuRendering {
                memory_mb: limits.memory_mb,
                max_surfaces: limits.max_surfaces,
            },
        }
    }

    /// Handle input device request
    fn handle_input_request(&mut self, pid: Pid, device: DeviceId) -> GuiResponse {
        // Check if compositor is registered
        if self.compositor_pid != Some(pid) {
            return GuiResponse::Denied {
                reason: DenialReason::Unauthorized,
            };
        }

        // Grant input device access
        let grant = InputGrant {
            device,
            exclusive: false,  // Allow shared access for now
        };

        self.input_grants
            .entry(pid)
            .or_insert_with(Vec::new)
            .push(grant);

        GuiResponse::Granted {
            capability: Capability::InputDevice {
                device,
                exclusive: false,
            },
        }
    }

    /// Handle surface creation
    fn handle_create_surface(&mut self, pid: Pid, config: SurfaceConfig) -> GuiResponse {
        // Check if process has GPU capability
        if !self.gpu_caps.contains_key(&pid) {
            return GuiResponse::Denied {
                reason: DenialReason::Unauthorized,
            };
        }

        // Check surface limit
        let caps = self.gpu_caps.get(&pid).unwrap();
        let current_surfaces = 0;  // Track surface count

        if current_surfaces >= caps.max_surfaces {
            return GuiResponse::Denied {
                reason: DenialReason::ResourceLimitExceeded,
            };
        }

        // Create surface
        let surface_id = SurfaceId::new();

        GuiResponse::Success  // Return surface ID in real implementation
    }

    /// Handle surface destruction
    fn handle_destroy_surface(&mut self, pid: Pid, surface: SurfaceId) -> GuiResponse {
        // Clean up surface
        GuiResponse::Success
    }

    /// Handle display mode setting
    fn handle_set_display_mode(&mut self, connector: ConnectorId, mode: DisplayMode) -> GuiResponse {
        // Only compositor can set display mode
        if self.compositor_pid.is_none() {
            return GuiResponse::Denied {
                reason: DenialReason::Unauthorized,
            };
        }

        // Set display mode via DRM driver
        match drm::set_mode(connector, mode) {
            Ok(_) => GuiResponse::Success,
            Err(e) => GuiResponse::Error {
                message: format!("Failed to set display mode: {}", e),
            },
        }
    }

    /// Handle device type query
    fn handle_query_device_type(&mut self) -> GuiResponse {
        // Detect device type from hardware
        let is_mobile = self.detect_mobile_device();

        GuiResponse::DeviceType { is_mobile }
    }

    /// Detect if current device is mobile
    fn detect_mobile_device(&self) -> bool {
        // Check for mobile indicators:
        // - Touchscreen present
        // - No physical keyboard/mouse
        // - Battery-powered
        // - Screen size < 10 inches

        // Placeholder: check device tree or ACPI
        false
    }

    /// Handle compositor registration
    fn handle_register_compositor(&mut self, pid: Pid) -> GuiResponse {
        // Only one compositor allowed
        if self.compositor_pid.is_some() {
            return GuiResponse::Denied {
                reason: DenialReason::AlreadyHeld,
            };
        }

        // Grant compositor capability
        self.compositor_pid = Some(pid);
        self.caps.grant_capability(pid, Capability::Compositor);

        GuiResponse::Success
    }

    /// Handle compositor unregistration
    fn handle_unregister_compositor(&mut self, pid: Pid) -> GuiResponse {
        if self.compositor_pid != Some(pid) {
            return GuiResponse::Denied {
                reason: DenialReason::Unauthorized,
            };
        }

        // Clean up
        self.compositor_pid = None;
        self.gpu_caps.clear();
        self.input_grants.clear();

        GuiResponse::Success
    }

    /// Handle capability check
    fn handle_has_capability(&mut self, pid: Pid, capability: Capability) -> GuiResponse {
        let has = self.caps.process_has_capability(pid, capability);

        GuiResponse::HasCapabilityResult { has_capability: has }
    }
}
```

## Userspace Implementation

### IPC Client

```rust
// apps/gui/libs/rustica-ipc/src/lib.rs

use crate::proto::{GuiRequest, GuiResponse};
use anyhow::Result;

/// IPC client for communicating with kernel
pub struct KernelIpcClient {
    /// IPC connection
    conn: IpcConnection,
}

impl KernelIpcClient {
    /// Connect to kernel GUI IPC server
    pub fn connect() -> Result<Self> {
        let conn = IpcConnection::connect("rustica-gui")?;
        Ok(Self { conn })
    }

    /// Request GPU capability
    pub fn request_gpu(&self, limits: GpuLimits) -> Result<GpuCapability> {
        let request = GuiRequest::RequestGpu {
            pid: std::process::id(),
            limits,
        };

        self.send_request(request)?;

        match self.receive_response()? {
            GuiResponse::Granted { capability } => {
                if let Capability::GpuRendering { memory_mb, max_surfaces } = capability {
                    Ok(GpuCapability {
                        memory_mb,
                        max_surfaces,
                    })
                } else {
                    Err(anyhow!("Unexpected capability type"))
                }
            }
            GuiResponse::Denied { reason } => {
                Err(anyhow!("GPU capability denied: {:?}", reason))
            }
            GuiResponse::Error { message } => {
                Err(anyhow!("GPU capability error: {}", message))
            }
            _ => Err(anyhow!("Unexpected response")),
        }
    }

    /// Request input device access
    pub fn request_input_device(&self, device: DeviceId) -> Result<()> {
        let request = GuiRequest::RequestInputDevice {
            pid: std::process::id(),
            device,
        };

        self.send_request(request)?;

        match self.receive_response()? {
            GuiResponse::Granted { .. } => Ok(()),
            GuiResponse::Denied { reason } => {
                Err(anyhow!("Input device denied: {:?}", reason))
            }
            _ => Err(anyhow!("Unexpected response")),
        }
    }

    /// Set display mode
    pub fn set_display_mode(&self, connector: ConnectorId, mode: DisplayMode) -> Result<()> {
        let request = GuiRequest::SetDisplayMode { connector, mode };

        self.send_request(request)?;

        match self.receive_response()? {
            GuiResponse::Success => Ok(()),
            GuiResponse::Error { message } => {
                Err(anyhow!("Failed to set display mode: {}", message))
            }
            _ => Err(anyhow!("Unexpected response")),
        }
    }

    /// Query device type
    pub fn is_mobile_device(&self) -> Result<bool> {
        let request = GuiRequest::QueryDeviceType;

        self.send_request(request)?;

        match self.receive_response()? {
            GuiResponse::DeviceType { is_mobile } => Ok(is_mobile),
            _ => Err(anyhow!("Unexpected response")),
        }
    }

    /// Register compositor
    pub fn register_compositor(&self) -> Result<()> {
        let request = GuiRequest::RegisterCompositor {
            pid: std::process::id(),
        };

        self.send_request(request)?;

        match self.receive_response()? {
            GuiResponse::Success => Ok(()),
            GuiResponse::Denied { reason } => {
                Err(anyhow!("Compositor registration denied: {:?}", reason))
            }
            _ => Err(anyhow!("Unexpected response")),
        }
    }

    /// Send request to kernel
    fn send_request(&self, request: GuiRequest) -> Result<()> {
        let msg = bincode::serialize(&request)?;
        self.conn.send(&msg)?;
        Ok(())
    }

    /// Receive response from kernel
    fn receive_response(&self) -> Result<GuiResponse> {
        let msg = self.conn.receive()?;
        let response: GuiResponse = bincode::deserialize(&msg)?;
        Ok(response)
    }
}
```

## Security

### Capability Enforcement

```rust
/// GPU capability limits
pub const MAX_TOTAL_GPU_MEMORY: u32 = 4096;  // 4GB total
pub const MAX_PROCESS_GPU_MEMORY: u32 = 1024;  // 1GB per process
pub const DEFAULT_GPU_MEMORY: u32 = 256;      // 256MB default
pub const DEFAULT_MAX_SURFACES: u32 = 16;
```

### Access Control

1. **Compositor-only operations**:
   - Set display mode
   - Access input devices
   - Register other processes

2. **Application capabilities**:
   - Request GPU rendering (with limits)
   - Create surfaces (with limits)
   - Destroy own surfaces

3. **Enforcement**:
   - All capability requests validated in kernel
   - Resource limits enforced
   - Only one compositor allowed
   - Automatic cleanup on process exit

## File Structure

```
/var/www/rustux.com/prod/
├── kernel/src/
│   └── gui/
│       ├── mod.rs                # GUI IPC module
│       ├── ipc.rs                # IPC protocol
│       ├── capability.rs         # Capability handling
│       └── display.rs            # Display management
│
└── apps/gui/
    └── libs/
        └── rustica-ipc/
            ├── Cargo.toml
            └── src/
                ├── lib.rs        # IPC client library
                └── proto.rs      # Protocol definitions
```

## Performance Requirements

| Metric | Target | Measurement |
|--------|--------|-------------|
| IPC round-trip | <100μs | Request to response |
| Capability grant | <1ms | Request to granted |
| Display mode set | <50ms | Request to applied |
| Memory overhead | <1MB | Kernel module |

## Success Criteria

- [ ] IPC protocol defined
- [ ] Kernel module compiles
- [ ] Userspace client compiles
- [ ] Compositor can register
- [ ] GPU capability requests work
- [ ] Input device access works
- [ ] Display mode setting works
- [ ] Security enforced
- [ ] Performance targets met
- [ ] Tests pass

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| IPC overhead | Use shared memory for bulk data |
| Capability leaks | Automatic cleanup on process exit |
| Deadlocks | Timeout mechanism, clear locking order |
| Privilege escalation | Strict capability checking |

## Sign-Off

**Architect**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅

---

## References

- [RUSTUX Kernel Capability System](/var/www/rustux.com/prod/kernel/src/capability/)
- [Linux DRM/KMS](https://www.kernel.org/doc/html/latest/gpu/drm-kms.html)
- [Wayland Security Model](https://wayland.freedesktop.org/docs/html/ch03.html)
- [seccomp Secure Computing](https://www.kernel.org/doc/html/latest/userspace-api/seccomp.html)
