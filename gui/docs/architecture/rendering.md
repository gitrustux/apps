# Rendering Backend Architecture Decision

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Component**: Rustica Shell - Rendering Pipeline

## Decision

**CHOSEN: Hybrid Rendering with EGL/OpenGL Primary + Pixman Software Fallback**

We will use a **tiered rendering approach** with hardware acceleration as primary and software rendering as fallback:

### Primary: EGL/OpenGL ES 3.1+
- **EGL** for display and surface management
- **OpenGL ES 3.1+** for rendering
- **DMABUF** for zero-copy buffer sharing
- **GLSL shaders** for effects and compositing

### Fallback: Pixman Software Rendering
- **Pixman** for software rasterization
- Automatic fallback when GPU unavailable
- Maintains full functionality without acceleration

### Rendering Pipeline

```
┌─────────────────────────────────────────────────────────────┐
│                     Rustica Compositor                       │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   EGL API    │  │   OpenGL ES  │  │   Pixman     │      │
│  │              │  │              │  │              │      │
│  │  Display/    │  │  Shaders/    │  │   Software   │      │
│  │  Surface     │  │  Rendering   │  │   Rasterize  │      │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘      │
│         │                 │                   │              │
│         └─────────────────┴───────────────────┘              │
│                           │                                  │
│                    ┌──────▼───────┐                           │
│                    │   Render     │                           │
│                    │   Backend    │                           │
│                    │  Abstraction │                           │
│                    └──────┬───────┘                           │
└───────────────────────────┼───────────────────────────────────┘
                            │
                    ┌───────▼────────┐
                    │  DRM/KMS       │
                    │  (Direct HW)   │
                    └────────────────┘
```

## Rationale

### Why EGL/OpenGL ES?

1. **Broad Hardware Support**
   - Supported by Intel, AMD, NVIDIA (via Mesa/NVIDIA drivers)
   - Mobile GPUs (Mali, Adreno, etc.)
   - Software implementation (Mesa llvmpipe) as last resort

2. **Performance**
   - Hardware acceleration for 60 FPS+
   - Efficient buffer management via DMABUF
   - GPU-accelerated shaders for effects

3. **Ecosystem**
   - Standard Wayland rendering stack
   - Well-tested in production (GNOME, KDE, COSMIC)
   - Extensive documentation

4. **Modern Features**
   - Multi-plane overlays (hardware planes)
   - Async flip for reduced latency
   - Variable refresh rate (VRR) support

### Why Not Alternatives?

| Alternative | Rejection Reason |
|-------------|------------------|
| **Vulkan** | Higher complexity, less mature for compositors, overkill for 2D |
| **wgpu** | Additional abstraction layer, less direct control, not battle-tested |
| **pure Pixman** | No hardware acceleration, poor performance on high-res |
| **Direct GL without EGL** | No portable surface management, no multi-vendor support |

## Implementation Details

### Rendering Backend Abstraction

```rust
// Rendering backend trait
pub trait RenderBackend {
    // Frame lifecycle
    fn start_frame(&mut self) -> Result<Frame>;
    fn finish_frame(&mut self, frame: Frame) -> Result<()>;

    // Surface management
    fn create_surface(&mut self, config: SurfaceConfig) -> Result<Surface>;
    fn destroy_surface(&mut self, surface: Surface);

    // Rendering operations
    fn render_surface(&mut self, frame: &mut Frame, surface: &Surface,
                      position: (i32, i32), alpha: f32);

    // Effects
    fn set_brightness(&mut self, value: f32);
    fn set_saturation(&mut self, value: f32);

    // Capability queries
    fn supports_dmabuf(&self) -> bool;
    fn supports_hw_planes(&self) -> bool;
    fn is_hardware_accelerated(&self) -> bool;
}

// EGL/OpenGL implementation
pub struct EGLBackend {
    display: EGLDisplay,
    context: EGLContext,
    // ...
}

impl RenderBackend for EGLBackend {
    // ...
}

// Pixman software fallback
pub struct PixmanBackend {
    // ...
}

impl RenderBackend for PixmanBackend {
    // ...
}
```

### Surface Formats

Supported pixel formats (in priority order):

1. **XRGB8888** / **ARGB8888** - 32-bit, most common
2. **RGB565** - 16-bit, mobile-friendly
3. **Double-buffered** - Always use double buffering

### Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| Frame time | <16ms (60 FPS) | Frame-to-frame latency |
| Input latency | <10ms | Input-to-display |
| Window create | <100ms | Time to first frame |
| Memory | <50MB compositor | RSS measurement |

### Hardware Acceleration Detection

```rust
pub fn detect_backend() -> Box<dyn RenderBackend> {
    // Try EGL/OpenGL first
    if let Ok(egl) = EGLBackend::new() {
        if egl.is_hardware_accelerated() {
            log::info!("Using EGL/OpenGL hardware acceleration");
            return Box::new(egl);
        }
    }

    // Fall back to Pixman
    log::warn!("Hardware acceleration unavailable, using Pixman software rendering");
    Box::new(PixmanBackend::new())
}
```

## Rendering Features

### Core Features (Phase 1)
- [ ] Basic surface composition
- [ ] Alpha blending
- [ ] Positioning and scaling
- [ ] Double-buffering

### Advanced Features (Phase 2)
- [ ] Multi-plane overlays (hardware planes)
- [ ] DMABUF zero-copy
- [ ] Async flip
- [ ] Variable refresh rate

### Effects (Phase 3)
- [ ] Blur/backdrop-filter
- [ ] Shadows
- [ ] Transitions and animations
- [ ] Color correction

## Dependencies

### System Libraries
```
libegl1-mesa-dev     # EGL platform interface
libgles2-mesa-dev    # OpenGL ES 3.1+
libgbm-dev           # Buffer management
libdrm-dev           # Direct rendering
mesa-vulkan-drivers  # Vulkan (optional, for future)
libpixman-1-dev      # Software rendering fallback
```

### Rust Crates
```toml
[dependencies]
smithay = { version = "0.18", features = [
    "use_system_lib",  # Use system libraries
    "renderer_gl",     # EGL/OpenGL backend
    "renderer_pixman", # Software fallback
] }
egl = "0.2"
opengles = "0.1"
```

## File Structure

```
/var/www/rustux.com/prod/apps/gui/
├── rustica-comp/src/
│   ├── rendering/
│   │   ├── mod.rs              # Rendering module
│   │   ├── backend.rs          # RenderBackend trait
│   │   ├── egl.rs              # EGL/OpenGL implementation
│   │   ├── pixman.rs           # Pixman fallback
│   │   ├── surface.rs          # Surface management
│   │   ├── shader.rs           # GLSL shaders
│   │   └── dmabuf.rs           # DMABUF handling
│   └── shaders/                # GLSL shader sources
│       ├── vertex.glsl
│       ├── fragment.glsl
│       └── effects.glsl
```

## Shader Examples

### Basic Vertex Shader
```glsl
#version 310 es

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 texcoord;

layout(location = 0) out vec2 v_texcoord;
layout(location = 1) out vec2 v_position;

layout(std140, binding = 0) uniform Globals {
    mat4 projection;
    mat4 transform;
};

void main() {
    v_texcoord = texcoord;
    v_position = position;
    gl_Position = projection * transform * vec4(position, 0.0, 1.0);
}
```

### Basic Fragment Shader
```glsl
#version 310 es

precision mediump float;

layout(location = 0) in vec2 v_texcoord;
layout(location = 1) in vec2 v_position;

layout(binding = 1) uniform sampler2D tex;

layout(location = 0) out vec4 frag_color;

uniform float u_alpha = 1.0;
uniform float u_brightness = 1.0;
uniform float u_saturation = 1.0;

void main() {
    vec4 color = texture(tex, v_texcoord);

    // Apply brightness
    color.rgb *= u_brightness;

    // Apply saturation
    float gray = dot(color.rgb, vec3(0.299, 0.587, 0.114));
    color.rgb = mix(vec3(gray), color.rgb, u_saturation);

    // Apply alpha
    frag_color = color * u_alpha;
}
```

## Success Criteria

- [ ] Compositor renders at 60 FPS with hardware acceleration
- [ ] Automatic fallback to software rendering works
- [ ] All surface formats render correctly
- [ ] DMABUF zero-copy buffer sharing works
- [ ] Multi-monitor rendering works
- [ ] Performance targets met (see table above)

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| GPU driver bugs | Test on multiple drivers, have software fallback |
| DMABUF complexity | Start without, add incrementally |
| Mobile GPU differences | Test on Mali/Adreno hardware |
| Performance regressions | Continuous benchmarking |

## Sign-Off

**Architect**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅

---

## References

- [EGL Specification](https://www.khronos.org/egl/)
- [OpenGL ES 3.1 Specification](https://www.khronos.org/opengles/)
- [Smithay Rendering](https://docs.rs/smithay/*/smithay/backend/renderer/index.html)
- [Mesa 3D Graphics Library](https://www.mesa3d.org/)
- [Pixman Image Compositing](https://pixman.org/)
