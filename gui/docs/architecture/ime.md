# IME & Multilingual Text Input Specification

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Component**: Rustica Shell - Internationalization

## Overview

This specification defines how Rustica Shell handles international text input, including **IME support for CJK languages**, **composition**, **candidate selection**, and **text input method extensibility**. It ensures **native input experience for all languages**.

## Design Philosophy

1. **International First** - All languages supported from day one
2. **IME Protocol** - Standard Wayland text-input protocol
3. **Extensible** - Third-party input methods can be added
4. **Transparent** - Apps don't need special handling

## IME Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                   Application (Text Field)                       │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  Text Input Widget                                         │  │
│  │  - Focus handling                                         │  │
│  │  - Cursor position                                        │  │
│  │  - Surrounding text                                       │  │
│  └───────────────────┬───────────────────────────────────────┘  │
└──────────────────────┼──────────────────────────────────────────┘
                       │ Wayland text-input protocol
                       ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Rustica IME Manager                          │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  IME Protocol Handler                                      │  │
│  │  - Text input v3 protocol                                 │  │
│  │  - Input method v2 protocol                               │  │
│  └───────────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  Composition Context                                      │  │
│  │  - Pre-edit text                                         │  │
│  │  - Commit text                                           │  │
│  │  - Cursor position                                        │  │
│  └───────────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  Candidate Window                                         │  │
│  │  - Candidate list                                         │  │
│  │  - Selection state                                        │  │
│  │  - Position tracking                                      │  │
│  └───────────────────────────────────────────────────────────┘  │
└───────────────────────┬──────────────────────────────────────────┘
                        │ IME protocol (DBus)
                        ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Input Method Engines                           │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐       │
│  │   Fcitx  │  │   IBus   │  │  uim     │  │ Custom   │       │
│  │   (CJK)  │  │ (Generic)│  │ (Japanese)│  │  IME     │       │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘       │
│                                                                   │
│  Supported languages:                                            │
│  - Chinese (Simplified/Traditional)                              │
│  - Japanese (Hiragana, Katakana, Kanji)                          │
│  - Korean (Hangul)                                               │
│  - Arabic, Hebrew (RTL)                                          │
│  - Indic scripts (Devanagari, Bengali, etc.)                     │
│  - Thai, Vietnamese                                              │
│  - All Latin scripts                                             │
└─────────────────────────────────────────────────────────────────┘
```

## Wayland Text Input Protocol

### Text Input Manager

```rust
use wayland_server::protocol::{
    wl_text_input_manager_v3::WlTextInputManagerV3,
    wl_text_input_v3::WlTextInputV3,
    wl_surface::WlSurface,
};

pub struct TextInputManager {
    // Global text input manager
    global: Global<WlTextInputManagerV3>,

    // Active text inputs per surface
    inputs: HashMap<SurfaceId, TextInput>,

    // Current input method
    input_method: Option<Box<dyn InputMethod>>,
}

impl TextInputManager {
    pub fn new(display: &Display) -> Self {
        let global = display.create_global(
            3,  // version
            move |display, _client, _version| {
                display.create_text_input_manager()
            },
        );

        Self {
            global,
            inputs: HashMap::new(),
            input_method: None,
        }
    }

    /// Get or create text input for surface
    pub fn get_text_input(&mut self, surface: WlSurface) -> TextInput {
        let id = surface.id();

        self.inputs.entry(id).or_insert_with(|| {
            TextInput::new(surface.clone())
        }).clone()
    }

    /// Set input method
    pub fn set_input_method(&mut self, method: Box<dyn InputMethod>) {
        self.input_method = Some(method);
    }
}
```

### Text Input Interface

```rust
pub struct TextInput {
    surface: WlSurface,

    // Current state
    focused: bool,
    content_type: ContentType,
    content_hint: ContentHint,

    // Cursor
    cursor_rect: Rectangle,

    // Text state
    surrounding_text: String,
    surrounding_cursor: usize,
    surrounding_anchor: usize,

    // IME callbacks
    commit_handler: Option<Box<dyn Fn(String)>>,
    preedit_handler: Option<Box<dyn Fn(Preedit)>>,
    delete_surrounding_handler: Option<Box<dyn Fn(usize, usize)>>,

    // Input method
    ime: Option<Box<dyn ImeContext>>,
}

#[derive(Clone, Copy)]
pub struct ContentType {
    pub hint: ContentHint,
    pub purpose: ContentPurpose,
}

#[derive(Clone, Copy, Flags)]
pub enum ContentHint {
    None = 0,
    Default = 1 << 0,
    Password = 1 << 1,
    AutoCompletion = 1 << 2,
    AutoCorrection = 1 << 3,
    AutoCapitalization = 1 << 4,
    Lowercase = 1 << 5,
    Uppercase = 1 << 6,
    Titlecase = 1 << 7,
    HiddenText = 1 << 8,
    SensitiveData = 1 << 9,
    Latin = 1 << 10,
    Multiline = 1 << 11,
}

pub enum ContentPurpose {
    Normal,
    Password,
    Email,
    Url,
    Number,
    Phone,
    Date,
    Time,
    DateTime,
    Terminal,
    Pin,
}

#[derive(Clone)]
pub struct Preedit {
    pub text: String,
    pub cursor_begin: Option<usize>,
    pub cursor_end: Option<usize>,
}

impl TextInput {
    /// Handle focus gain
    pub fn focus(&mut self) {
        self.focused = true;

        // Activate IME
        if let Some(ref mut ime) = self.ime {
            ime.activate();
            ime.set_content_type(self.content_type);
            ime.set_cursor_rect(self.cursor_rect);
        }
    }

    /// Handle focus loss
    pub fn unfocus(&mut self) {
        self.focused = false;

        // Deactivate IME
        if let Some(ref mut ime) = self.ime {
            ime.deactivate();
        }
    }

    /// Set cursor rectangle (for candidate window positioning)
    pub fn set_cursor_rectangle(&mut self, rect: Rectangle) {
        self.cursor_rect = rect;

        if let Some(ref mut ime) = self.ime {
            ime.set_cursor_rect(rect);
        }
    }

    /// Set content type (password, email, etc.)
    pub fn set_content_type(&mut self, content_type: ContentType) {
        self.content_type = content_type;

        if let Some(ref mut ime) = self.ime {
            ime.set_content_type(content_type);
        }
    }

    /// Set surrounding text (for context-aware input)
    pub fn set_surrounding_text(
        &mut self,
        text: String,
        cursor: usize,
        anchor: usize,
    ) {
        self.surrounding_text = text;
        self.surrounding_cursor = cursor;
        self.surrounding_anchor = anchor;

        if let Some(ref mut ime) = self.ime {
            ime.set_surrounding_text(&text, cursor, anchor);
        }
    }

    /// Handle key event
    pub fn handle_key(&mut self, key: KeyEvent) -> InputResult {
        if let Some(ref mut ime) = self.ime && self.focused {
            ime.process_key(key)
        } else {
            InputResult::NotHandled
        }
    }

    /// Commit text from IME
    pub fn commit(&self, text: String) {
        if let Some(ref handler) = self.commit_handler {
            handler(text);
        }
    }

    /// Show pre-edit text from IME
    pub fn show_preedit(&self, preedit: Preedit) {
        if let Some(ref handler) = self.preedit_handler {
            handler(preedit);
        }
    }

    /// Delete surrounding text
    pub fn delete_surrounding(&self, before: usize, after: usize) {
        if let Some(ref handler) = self.delete_surrounding_handler {
            handler(before, after);
        }
    }
}

pub enum InputResult {
    /// IME handled the key
    Handled,

    /// IME didn't handle, app should handle
    NotHandled,

    /// IME committed text
    Commit(String),

    /// IME updated pre-edit
    Preedit(Preedit),
}
```

## IME Context Interface

```rust
pub trait ImeContext: Send + Sync {
    /// Activate IME for this context
    fn activate(&mut self);

    /// Deactivate IME for this context
    fn deactivate(&mut self);

    /// Set cursor rectangle (for candidate window)
    fn set_cursor_rect(&mut self, rect: Rectangle);

    /// Set content type
    fn set_content_type(&mut self, content_type: ContentType);

    /// Set surrounding text
    fn set_surrounding_text(&mut self, text: &str, cursor: usize, anchor: usize);

    /// Process key event
    fn process_key(&mut self, key: KeyEvent) -> InputResult;

    /// Reset IME state
    fn reset(&mut self);

    /// Get IME name
    fn name(&self) -> &str;

    /// Get supported languages
    fn languages(&self) -> &[LanguageCode];
}

#[derive(Clone, Debug)]
pub struct LanguageCode(String);

impl LanguageCode {
    pub fn from_str(code: &str) -> Self {
        Self(code.to_lowercase())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Common language codes
impl LanguageCode {
    pub const ENGLISH: Self = Self("en".into());
    pub const CHINESE_SIMPLIFIED: Self = Self("zh_cn".into());
    pub const CHINESE_TRADITIONAL: Self = Self("zh_tw".into());
    pub const JAPANESE: Self = Self("ja".into());
    pub const KOREAN: Self = Self("ko".into());
    pub const ARABIC: Self = Self("ar".into());
    pub const HEBREW: Self = Self("he".into());
    pub const HINDI: Self = Self("hi".into());
    pub const THAI: Self = Self("th".into());
    pub const VIETNAMESE: Self = Self("vi".into());
}
```

## Fcitx Integration

```rust
use dbus::blocking::Connection;

pub struct FcitxIme {
    connection: Connection,
    dbus_name: String,
    ic_path: String,
    capabilities: ImeCapabilities,
}

#[derive(Clone, Copy)]
pub struct ImeCapabilities {
    pub preedit: bool,
    pub client_window: bool,
    pub surrounding_text: bool,
    pub password: bool,
}

impl FcitxIme {
    pub fn new() -> Result<Self> {
        // Connect to Fcitx D-Bus service
        let connection = Connection::new_session()?;

        // Check if Fcitx is running
        let proxy = connection.with_proxy(
            "org.fcitx.Fcitx",
            "/inputmethod",
            Duration::from_secs(5),
        );

        // Create input context
        let (ic_path, capabilities): (String, ImeCapabilities) = proxy
            .method_call("org.fcitx.Fcitx.InputMethod", "CreateIC", ("rustica",))
            .await?;

        Ok(Self {
            connection,
            dbus_name: "org.fcitx.Fcitx".into(),
            ic_path,
            capabilities,
        })
    }
}

impl ImeContext for FcitxIme {
    fn activate(&mut self) {
        let proxy = self.connection.with_proxy(
            &self.dbus_name,
            &self.ic_path,
            Duration::from_secs(1),
        );

        let _: () = proxy
            .method_call("org.fcitx.Fcitx.InputContext", "FocusIn", ())
            .unwrap_or(());
    }

    fn deactivate(&mut self) {
        let proxy = self.connection.with_proxy(
            &self.dbus_name,
            &self.ic_path,
            Duration::from_secs(1),
        );

        let _: () = proxy
            .method_call("org.fcitx.Fcitx.InputContext", "FocusOut", ())
            .unwrap_or(());
    }

    fn set_cursor_rect(&mut self, rect: Rectangle) {
        let proxy = self.connection.with_proxy(
            &self.dbus_name,
            &self.ic_path,
            Duration::from_secs(1),
        );

        let _: () = proxy
            .method_call(
                "org.fcitx.Fcitx.InputContext",
                "SetCursorRect",
                (rect.x, rect.y, rect.width, rect.height),
            )
            .unwrap_or(());
    }

    fn set_content_type(&mut self, content_type: ContentType) {
        let proxy = self.connection.with_proxy(
            &self.dbus_name,
            &self.ic_path,
            Duration::from_secs(1),
        );

        let purpose = match content_type.purpose {
            ContentPurpose::Password => 1,
            ContentPurpose::Email => 2,
            ContentPurpose::Url => 3,
            ContentPurpose::Number => 4,
            ContentPurpose::Phone => 5,
            ContentPurpose::Date => 6,
            ContentPurpose::Time => 7,
            ContentPurpose::DateTime => 8,
            ContentPurpose::Terminal => 9,
            ContentPurpose::Pin => 10,
            ContentPurpose::Normal => 0,
        };

        let _: () = proxy
            .method_call("org.fcitx.Fcitx.InputContext", "SetContentType", (purpose,))
            .unwrap_or(());
    }

    fn set_surrounding_text(&mut self, text: &str, cursor: usize, anchor: usize) {
        let proxy = self.connection.with_proxy(
            &self.dbus_name,
            &self.ic_path,
            Duration::from_secs(1),
        );

        let _: () = proxy
            .method_call(
                "org.fcitx.Fcitx.InputContext",
                "SetSurroundingText",
                (text, cursor as i32, anchor as i32),
            )
            .unwrap_or(());
    }

    fn process_key(&mut self, key: KeyEvent) -> InputResult {
        let proxy = self.connection.with_proxy(
            &self.dbus_name,
            &self.ic_path,
            Duration::from_secs(1),
        );

        let keycode = key.key as u32;
        let modifiers = modifiers_to_fcitx(key.modifiers);

        let (handled,): (bool,) = proxy
            .method_call(
                "org.fcitx.Fcitx.InputContext",
                "ProcessKeyEvent",
                (keycode, 0, modifiers, 0, 0),
            )
            .unwrap_or((false,));

        if handled {
            // Wait for commit signal
            InputResult::Handled
        } else {
            InputResult::NotHandled
        }
    }

    fn reset(&mut self) {
        let proxy = self.connection.with_proxy(
            &self.dbus_name,
            &self.ic_path,
            Duration::from_secs(1),
        );

        let _: () = proxy
            .method_call("org.fcitx.Fcitx.InputContext", "Reset", ())
            .unwrap_or(());
    }

    fn name(&self) -> &str {
        "Fcitx"
    }

    fn languages(&self) -> &[LanguageCode] {
        &[
            LanguageCode::CHINESE_SIMPLIFIED,
            LanguageCode::CHINESE_TRADITIONAL,
            LanguageCode::JAPANESE,
            LanguageCode::KOREAN,
            LanguageCode::ENGLISH,
        ]
    }
}
```

## Candidate Window

```rust
pub struct CandidateWindow {
    // Candidates
    candidates: Vec<Candidate>,
    selected_index: usize,

    // Position
    position: Point,

    // Visibility
    visible: bool,

    // Page size
    page_size: usize,
    current_page: usize,
}

#[derive(Clone)]
pub struct Candidate {
    pub text: String,
    pub annotation: Option<String>,
    pub index: usize,
}

impl CandidateWindow {
    pub fn new() -> Self {
        Self {
            candidates: Vec::new(),
            selected_index: 0,
            position: Point::new(0, 0),
            visible: false,
            page_size: 10,
            current_page: 0,
        }
    }

    /// Update candidates
    pub fn update_candidates(&mut self, candidates: Vec<Candidate>) {
        self.candidates = candidates;
        self.selected_index = 0;
        self.current_page = 0;
    }

    /// Show window at position
    pub fn show(&mut self, position: Point) {
        self.position = position;
        self.visible = true;
    }

    /// Hide window
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Select next candidate
    pub fn select_next(&mut self) {
        if self.selected_index + 1 < self.candidates.len() {
            self.selected_index += 1;
        }
    }

    /// Select previous candidate
    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Select candidate by index
    pub fn select(&mut self, index: usize) {
        if index < self.candidates.len() {
            self.selected_index = index;
        }
    }

    /// Get selected candidate
    pub fn selected(&self) -> Option<&Candidate> {
        self.candidates.get(self.selected_index)
    }

    /// Commit selected candidate
    pub fn commit_selected(&self) -> Option<String> {
        self.selected().map(|c| c.text.clone())
    }

    /// Get current page
    pub fn current_page(&self) -> &[Candidate] {
        let start = self.current_page * self.page_size;
        let end = (start + self.page_size).min(self.candidates.len());
        &self.candidates[start..end]
    }

    /// Next page
    pub fn next_page(&mut self) {
        let total_pages = (self.candidates.len() + self.page_size - 1) / self.page_size;
        self.current_page = (self.current_page + 1) % total_pages;
    }

    /// Previous page
    pub fn previous_page(&mut self) {
        let total_pages = (self.candidates.len() + self.page_size - 1) / self.page_size;
        self.current_page = self.current_page.checked_sub(1).unwrap_or(total_pages - 1);
    }

    /// Handle key event
    pub fn handle_key(&mut self, key: KeyEvent) -> CandidateResult {
        match key {
            KeyEvent { key: KeyCode::Up, .. } => {
                self.select_previous();
                CandidateResult::Update
            }
            KeyEvent { key: KeyCode::Down, .. } => {
                self.select_next();
                CandidateResult::Update
            }
            KeyEvent { key: KeyCode::PageUp, .. } => {
                self.previous_page();
                CandidateResult::Update
            }
            KeyEvent { key: KeyCode::PageDown, .. } => {
                self.next_page();
                CandidateResult::Update
            }
            KeyEvent { key: KeyCode::Enter, .. } => {
                if let Some(text) = self.commit_selected() {
                    CandidateResult::Commit(text)
                } else {
                    CandidateResult::None
                }
            }
            KeyEvent { key: KeyCode::Escape, .. } => {
                CandidateResult::Cancel
            }
            // Number keys (1-9) for quick selection
            KeyEvent { key: k @ KeyCode::Key1 ..= KeyCode::Key9, .. } => {
                let index = (k as u32 - KeyCode::Key1 as u32) as usize;
                if index < self.current_page().len() {
                    let actual_index = self.current_page * self.page_size + index;
                    self.select(actual_index);
                    CandidateResult::Commit(self.candidates[actual_index].text.clone())
                } else {
                    CandidateResult::None
                }
            }
            _ => CandidateResult::None,
        }
    }
}

pub enum CandidateResult {
    /// Candidate selection changed
    Update,

    /// Candidate committed
    Commit(String),

    /// Candidate selection cancelled
    Cancel,

    /// Key not handled
    None,
}
```

## Text Layout

### Complex Text Shaping

```rust
use rustybuzz::Face;

pub struct TextShaper {
    face: Face,
}

impl TextShaper {
    pub fn new(font_data: &[u8]) -> Self {
        let face = Face::from_slice(font_data, 0).unwrap();
        Self { face }
    }

    /// Shape text with proper ligatures and positioning
    pub fn shape(&self, text: &str, script: Script) -> Vec<Glyph> {
        let mut buffer = rustybuzz::UnicodeBuffer::new();
        buffer.push_str(text);
        buffer.set_script(script);

        let glyphs = rustybuzz::shape(&self.face, &[], buffer);

        glyphs.glyph_positions().iter()
            .zip(glyphs.glyph_infos().iter())
            .map(|(pos, info)| Glyph {
                id: info.glyph_id,
                x: pos.x_offset,
                y: pos.y_offset,
                cluster: info.cluster,
            })
            .collect()
    }

    /// Get text metrics
    pub fn metrics(&self, text: &str) -> TextMetrics {
        let glyphs = self.shape(text, Script::Latin);

        let width = glyphs.iter()
            .map(|g| g.x_advance)
            .sum();

        let height = self.face.metrics().ascender - self.face.metrics().descender;

        TextMetrics { width, height }
    }
}

pub struct Glyph {
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub x_advance: i32,
    pub cluster: u32,
}

pub struct TextMetrics {
    pub width: i32,
    pub height: i32,
}

// Script support
pub enum Script {
    Latin,
    Arabic,
    Hebrew,
    Chinese,
    Japanese,
    Korean,
    Hindi,
    Thai,
    Vietnamese,
}

impl Script {
    pub fn from_str(code: &str) -> Self {
        match code.to_lowercase().as_str() {
            "ar" => Script::Arabic,
            "he" => Script::Hebrew,
            "zh" => Script::Chinese,
            "ja" => Script::Japanese,
            "ko" => Script::Korean,
            "hi" => Script::Hindi,
            "th" => Script::Thai,
            "vi" => Script::Vietnamese,
            _ => Script::Latin,
        }
    }
}
```

### Bidirectional Text

```rust
use unicode_bidi as bidi;

pub struct BidiProcessor;

impl BidiProcessor {
    /// Reorder text for proper display
    pub fn reorder(text: &str, direction: Direction) -> String {
        let bidi_info = bidi::BidiInfo::new(text, None);

        // Get visual runs
        let runs = bidi_info.visual_runs(
            bidi_info.paragraphs,
            direction,
        );

        // Reorder based on runs
        let mut result = String::new();
        for run in runs {
            let segment = &text[run.start..run.end];
            if run.is_rtl {
                // Reverse for RTL
                result.extend(segment.chars().rev());
            } else {
                result.push_str(segment);
            }
        }

        result
    }

    /// Detect text direction
    pub fn detect_direction(text: &str) -> Direction {
        let has_rtl = text.chars()
            .any(|c| bidi::bidi_class::is_rtl(c));

        if has_rtl {
            Direction::RTL
        } else {
            Direction::LTR
        }
    }
}

pub enum Direction {
    LTR,
    RTL,
}
```

## File Structure

```
/var/www/rustux.com/prod/apps/gui/
├── libs/librustica-ime/
│   ├── src/
│   │   ├── mod.rs
│   │   ├── context.rs          # IME context interface
│   │   ├── text_input.rs       # Text input implementation
│   │   ├── preedit.rs          # Pre-edit handling
│   │   ├── candidate.rs        # Candidate window
│   │   ├── engines/
│   │   │   ├── mod.rs
│   │   │   ├── fcitx.rs        # Fcitx integration
│   │   │   ├── ibus.rs         # IBus integration
│   │   │   └── dummy.rs        # Fallback (Latin only)
│   │   └── text/
│   │       ├── mod.rs
│   │       ├── shaping.rs      # Text shaping
│   │       ├── bidi.rs         # Bidirectional text
│   │       └── metrics.rs      # Text metrics
│   └── Cargo.toml
│
└── rustica-comp/src/
    └── ime/
        ├── mod.rs
        ├── protocol.rs         # Wayland text-input protocol
        └── manager.rs          # IME manager
```

## Performance Requirements

| Metric | Target | Measurement |
|--------|--------|-------------|
| IME commit latency | <50ms | Key to commit |
| Candidate update | <16ms | Change to visible |
| Text shaping | <10ms | Text to glyphs |
| IME activation | <20ms | Focus to ready |
| Memory overhead | <20MB | With IME loaded |

## Success Criteria

- [ ] Wayland text-input protocol works
- [ ] Fcitx integration functional
- [ ] IBus integration functional
- [ ] Candidate window displays correctly
- [ ] All CJK languages supported
- [ ] RTL text works (Arabic, Hebrew)
- [ ] Text shaping correct
- [ ] Performance targets met
- [ ] Tests pass

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| IME not available | Provide fallback Latin-only input |
| Performance issues | Cache shaping results, lazy loading |
| Font support missing | Bundle CJK fonts, system font fallback |
| Candidate positioning | Track cursor position accurately |

## Sign-Off

**Architect**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅

---

## References

- [Wayland Text Input Protocol v3](https://wayland.freedesktop.org/docs/html/ch04.html#sect-Protocol)
- [Fcitx Documentation](https://fcitx-im.org/wiki/Documentation)
- [IBus Documentation](https://github.com/ibus/ibus/wiki)
- [Unicode Bidirectional Algorithm](https://unicode.org/reports/tr9/)
- [Rustybuzz Text Shaping](https://github.com/RazrFalcon/rustybuzz)
- [HarfBuzz Shaping Engine](https://harfbuzz.github.io/)
