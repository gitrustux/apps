# Phase 8.2: On-Screen Keyboard Integration (rustica-keyboard)

## Overview

**Component**: rustica-keyboard
**Purpose**: Virtual on-screen keyboard for touch devices with multilingual support
**Language**: Rust
**Dependencies**: smithay, crossterm (key layouts), xkbcommon, dbus (zbus)

## Goals

1. **Auto-Show/Hide**: Automatically appear when text input is focused
2. **Layout Variety**: QWERTY, AZERTY, QWERTZ, Dvorak, and phonetic layouts
3. **Multilingual**: Support 100+ languages via XKB layouts
4. **Gesture Typing**: Swipe-to-type functionality
5. **Predictive Text**: AI-powered word suggestions
6. **Accessibility**: High contrast mode, resizable keyboard, scanning mode

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Client Application                        │
│                    (text field focused)                       │
└────────────────────────┬────────────────────────────────────┘
                         │ text_input_v3 protocol
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                  rustica-comp (Compositor)                   │
│              (manages text-input state)                       │
└────────────────────────┬────────────────────────────────────┘
                         │ D-Bus activation request
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                  rustica-keyboard                            │
│                  (Virtual Keyboard)                          │
├─────────────────────────────────────────────────────────────┤
│  KeyboardView       │  InputEngine      │  PredictionEngine  │
│  - Layout rendering │  - Key events     │  - Language model  │
│  - Touch handling   │  - XKB state      │  - Word suggestions│
│  - Animations       │  - IME integration│  - Swipe typing    │
└────────────────────────┬────────────────────────────────────┘
                         │ key events
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                  rustica-comp                                 │
│           (forward to client via text-input)                  │
└─────────────────────────────────────────────────────────────┘
```

## Core Data Structures

```rust
/// Keyboard layout and appearance
pub struct KeyboardState {
    pub layout: KeyboardLayout,
    pub mode: KeyboardMode,
    pub shift_state: ShiftState,
    pub modifier_state: ModifierState,
    pub prediction_enabled: bool,
    pub height_percent: u32,  // Screen height percentage (25-50)
    pub theme: KeyboardTheme,
    pub autocapitalization: Autocapitalization,
}

/// Supported keyboard layouts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardLayout {
    Qwerty,
    Azerty,
    Qwertz,
    Dvorak,
    Colemak,
    Workman,
    Neo2,
    /// Language-specific layout (e.g., Russian, Arabic, Hebrew)
    Phonetic(Language),
}

/// Keyboard display modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardMode {
    /// Standard alphanumeric layout
    Text,

    /// Only numbers and symbols
    Numeric,

    /// Phone-style numeric pad (3×3 grid)
    PhonePad,

    /// Email entry mode (optimized layout with @ key)
    Email,

    /// URL entry mode (optimized with /, ., and quick .com)
    Url,

    /// Decimal number entry
    Decimal,
}

/// Shift key state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShiftState {
    Off,
    On,
    Lock,  // Caps lock
}

/// Modifier key states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ModifierState {
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
    pub super_: bool,
    pub altgr: bool,
}

/// Keyboard visual theme
#[derive(Debug, Clone)]
pub enum KeyboardTheme {
    Light,
    Dark,
    HighContrast,
}

/// Autocapitalization behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Autocapitalization {
    None,
    Sentences,  // Capitalize after sentence-ending punctuation
    Words,      // Capitalize every word
    Characters, // All caps
}

/// Individual keyboard key
#[derive(Debug, Clone)]
pub struct Key {
    pub label: String,
    pub key_code: KeyCode,
    pub width: KeyWidth,
    pub action: KeyAction,
    pub popup_label: Option<String>,  // Accented character popup
}

/// Key width specification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyWidth {
    Standard,      // 1 unit
    Half,          // 0.5 units
    Double,        // 2 units
    Triple,        // 3 units (spacebar)
    Quadruple,     // 4 units
    Flexible,      // Expands to fill available space
}

/// Key press action
#[derive(Debug, Clone)]
pub enum KeyAction {
    /// Send character to application
    Char(char),

    /// Send special keycode
    Code(KeyCode),

    /// Shift modifier toggle
    Shift,

    /// Backspace/delete
    Backspace,

    /// Enter/return key
    Enter,

    /// Switch to different keyboard mode
    ModeChange(KeyboardMode),

    /// Change keyboard layout
    LayoutChange(KeyboardLayout),

    /// Show emoji picker
    EmojiPicker,

    /// Show language switcher
    LanguageSwitcher,

    /// Hide keyboard
    Hide,

    /// No action (spacer)
    None,
}

/// Physical key codes (following evdev/XKB naming)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    KEY_A, KEY_B, KEY_C, /* ... */
    KEY_1, KEY_2, KEY_3, /* ... */
    KEY_SPACE, KEY_ENTER, KEY_BACKSPACE,
    KEY_LEFTSHIFT, KEY_RIGHTSHIFT,
    KEY_LEFTCTRL, KEY_RIGHTCTRL,
    KEY_LEFTALT, KEY_RIGHTALT,
    KEY_TAB, KEY_ESCAPE,
    /* ... */
}
```

## Keyboard View

```rust
pub struct KeyboardView {
    state: KeyboardState,
    keys: Vec<Vec<Key>>,
    current_suggestion: Option<String>,
    swipe_trail: Vec<Point<f64, Logical>>,
    gesture_engine: GestureEngine,
}

impl KeyboardView {
    /// Generate keyboard layout for current mode
    pub fn generate_layout(&mut self) {
        self.keys = match self.state.mode {
            KeyboardMode::Text => self.text_layout(),
            KeyboardMode::Numeric => self.numeric_layout(),
            KeyboardMode::PhonePad => self.phone_pad_layout(),
            KeyboardMode::Email => self.email_layout(),
            KeyboardMode::Url => self.url_layout(),
            KeyboardMode::Decimal => self.decimal_layout(),
        };
    }

    /// Standard QWERTY text layout
    fn text_layout(&self) -> Vec<Vec<Key>> {
        let mut rows = Vec::new();

        // Row 1: Numbers
        rows.push(vec![
            key!("1", KeyCode::KEY_1),
            key!("2", KeyCode::KEY_2),
            key!("3", KeyCode::KEY_3),
            key!("4", KeyCode::KEY_4),
            key!("5", KeyCode::KEY_5),
            key!("6", KeyCode::KEY_6),
            key!("7", KeyCode::KEY_7),
            key!("8", KeyCode::KEY_8),
            key!("9", KeyCode::KEY_9),
            key!("0", KeyCode::KEY_0),
        ]);

        // Row 2: QWERTYUIOP
        rows.push(vec![
            key!("q", KeyCode::KEY_Q),
            key!("w", KeyCode::KEY_W),
            key!("e", KeyCode::KEY_E),
            key!("r", KeyCode::KEY_R),
            key!("t", KeyCode::KEY_T),
            key!("y", KeyCode::KEY_Y),
            key!("u", KeyCode::KEY_U),
            key!("i", KeyCode::KEY_I),
            key!("o", KeyCode::KEY_O),
            key!("p", KeyCode::KEY_P),
        ]);

        // Row 3: ASDFGHJKL
        rows.push(vec![
            key!("a", KeyCode::KEY_A, width = Half),
            key!("s", KeyCode::KEY_S),
            key!("d", KeyCode::KEY_D),
            key!("f", KeyCode::KEY_F),
            key!("g", KeyCode::KEY_G),
            key!("h", KeyCode::KEY_H),
            key!("j", KeyCode::KEY_J),
            key!("k", KeyCode::KEY_K),
            key!("l", KeyCode::KEY_L),
            key_backspace(),
        ]);

        // Row 4: ZXCVBNM with shift
        rows.push(vec![
            key_shift(),
            key!("z", KeyCode::KEY_Z),
            key!("x", KeyCode::KEY_X),
            key!("c", KeyCode::KEY_C),
            key!("v", KeyCode::KEY_V),
            key!("b", KeyCode::KEY_B),
            key!("n", KeyCode::KEY_N),
            key!("m", KeyCode::KEY_M),
            key_enter(),
        ]);

        // Row 5: Comma, period, space, ?, emoji
        rows.push(vec![
            key!(",", KeyCode::KEY_COMMA),
            key!(".", KeyCode::KEY_DOT),
            key_space(),
            key!("?", KeyCode::KEY_SLASH),
            key_emoji(),
        ]);

        rows
    }

    /// Numeric keypad layout
    fn numeric_layout(&self) -> Vec<Vec<Key>> {
        vec![
            vec![
                key!("1", KeyCode::KEY_1),
                key!("2", KeyCode::KEY_2),
                key!("3", KeyCode::KEY_3),
            ],
            vec![
                key!("4", KeyCode::KEY_4),
                key!("5", KeyCode::KEY_5),
                key!("6", KeyCode::KEY_6),
            ],
            vec![
                key!("7", KeyCode::KEY_7),
                key!("8", KeyCode::KEY_8),
                key!("9", KeyCode::KEY_9),
            ],
            vec![
                key!(".", KeyCode::KEY_DOT),
                key!("0", KeyCode::KEY_0),
                key_backspace(),
            ],
            vec![key_hide()],
        ]
    }

    /// Phone-style numeric pad (3×3)
    fn phone_pad_layout(&self) -> Vec<Vec<Key>> {
        vec![
            vec![
                key!("1", KeyCode::KEY_1),
                key!("2", KeyCode::KEY_2, "ABC"),
                key!("3", KeyCode::KEY_3, "DEF"),
            ],
            vec![
                key!("4", KeyCode::KEY_4, "GHI"),
                key!("5", KeyCode::KEY_5, "JKL"),
                key!("6", KeyCode::KEY_6, "MNO"),
            ],
            vec![
                key!("7", KeyCode::KEY_7, "PQRS"),
                key!("8", KeyCode::KEY_8, "TUV"),
                key!("9", KeyCode::KEY_9, "WXYZ"),
            ],
            vec![
                key!("*", KeyCode::KEY_KPASTERISK),
                key!("0", KeyCode::KEY_0, "+"),
                key!("#", KeyCode::KEY_HASH),
            ],
        ]
    }

    /// Email-optimized layout
    fn email_layout(&self) -> Vec<Vec<Key>> {
        // Similar to text layout but with @ key in prominent position
        // and .com quick action
        vec![
            // ... (similar to text layout with email-specific keys)
        ]
    }

    /// URL-optimized layout
    fn url_layout(&self) -> Vec<Vec<Key>> {
        // With / and . prominently placed
        // and .com, .net, .org quick actions
        vec![
            // ...
        ]
    }
}

/// Helper macro for creating keys (simplified)
macro_rules! key {
    ($label:expr, $code:expr) => {
        Key {
            label: $label.to_string(),
            key_code: $code,
            width: KeyWidth::Standard,
            action: KeyAction::Code($code),
            popup_label: None,
        }
    };
    ($label:expr, $code:expr, popup = $popup:expr) => {
        Key {
            label: $label.to_string(),
            key_code: $code,
            width: KeyWidth::Standard,
            action: KeyAction::Code($code),
            popup_label: Some($popup.to_string()),
        }
    };
    ($label:expr, $code:expr, width = $width:ident) => {
        Key {
            label: $label.to_string(),
            key_code: $code,
            width: KeyWidth::$width,
            action: KeyAction::Code($code),
            popup_label: None,
        }
    };
}

fn key_backspace() -> Key {
    Key {
        label: "⌫".to_string(),
        key_code: KeyCode::KEY_BACKSPACE,
        width: KeyWidth::Double,
        action: KeyAction::Backspace,
        popup_label: None,
    }
}

fn key_shift() -> Key {
    Key {
        label: "⇧".to_string(),
        key_code: KeyCode::KEY_LEFTSHIFT,
        width: KeyWidth::Double,
        action: KeyAction::Shift,
        popup_label: None,
    }
}

fn key_enter() -> Key {
    Key {
        label: "↵".to_string(),
        key_code: KeyCode::KEY_ENTER,
        width: KeyWidth::Double,
        action: KeyAction::Enter,
        popup_label: None,
    }
}

fn key_space() -> Key {
    Key {
        label: " ".to_string(),
        key_code: KeyCode::KEY_SPACE,
        width: KeyWidth::Quadruple,
        action: KeyAction::Char(' '),
        popup_label: None,
    }
}

fn key_emoji() -> Key {
    Key {
        label: "☺".to_string(),
        key_code: KeyCode::KEY_UNKNOWN,
        width: KeyWidth::Standard,
        action: KeyAction::EmojiPicker,
        popup_label: None,
    }
}

fn key_hide() -> Key {
    Key {
        label: "▼".to_string(),
        key_code: KeyCode::KEY_UNKNOWN,
        width: KeyWidth::Flexible,
        action: KeyAction::Hide,
        popup_label: None,
    }
}
```

## Input Engine

```rust
pub struct InputEngine {
    xkb_context: xkbcommon::xkb::Context,
    xkb_keymap: xkbcommon::xkb::Keymap,
    xkb_state: xkbcommon::xkb::State,
    current_layout: KeyboardLayout,
    modifier_state: ModifierState,
    ime_engine: ImeEngine,
}

impl InputEngine {
    /// Create new input engine with specified layout
    pub fn new(layout: KeyboardLayout) -> Result<Self, Error> {
        let context = xkbcommon::xkb::Context::new(0);
        let layout_name = match layout {
            KeyboardLayout::Qwerty => "us",
            KeyboardLayout::Azerty => "fr",
            KeyboardLayout::Qwertz => "de",
            KeyboardLayout::Dvorak => "us(dvorak)",
            KeyboardLayout::Phonetic(lang) => lang.xkb_layout_name(),
        };

        let keymap = context.keymap_from_layout(layout_name)?;
        let state = keymap.state_new();

        Ok(Self {
            xkb_context: context,
            xkb_keymap: keymap,
            xkb_state: state,
            current_layout: layout,
            modifier_state: ModifierState::default(),
            ime_engine: ImeEngine::new(),
        })
    }

    /// Process a key press and return resulting text
    pub fn process_key(&mut self, key: &Key) -> KeyResult {
        match &key.action {
            KeyAction::Char(c) => {
                let char = if self.modifier_state.shift {
                    c.to_ascii_uppercase()
                } else {
                    *c
                };

                // Auto-disable shift after typing
                if matches!(self.shift_state, ShiftState::On) {
                    self.shift_state = ShiftState::Off;
                }

                KeyResult::Char(char)
            }

            KeyAction::Code(code) => {
                // Use XKB to translate keycode + modifiers to keysym
                let keysym = self.xkb_state.keymap_one_sym(
                    code.to_xkb_keycode(),
                    self.modifier_state.to_xkb_modmask(),
                );

                // Convert keysym to Unicode
                let text = self.keysym_to_unicode(keysym);

                KeyResult::Text(text)
            }

            KeyAction::Shift => {
                self.shift_state = match self.shift_state {
                    ShiftState::Off => ShiftState::On,
                    ShiftState::On => ShiftState::Lock,
                    ShiftState::Lock => ShiftState::Off,
                };

                KeyResult::ModifierChange
            }

            KeyAction::Backspace => KeyResult::Command(KeyCommand::Backspace),

            KeyAction::Enter => KeyResult::Command(KeyCommand::Enter),

            KeyAction::ModeChange(mode) => {
                KeyResult::ModeChange(*mode)
            }

            KeyAction::Hide => KeyResult::Command(KeyCommand::HideKeyboard),

            _ => KeyResult::None,
        }
    }

    /// Handle key release
    pub fn process_key_release(&mut self, key: &Key) -> KeyResult {
        match &key.action {
            KeyAction::Code(code) => {
                KeyResult::KeyUp(*code)
            }
            _ => KeyResult::None,
        }
    }

    fn keysym_to_unicode(&self, keysym: u32) -> String {
        // Use xkbcommon_keysym_to_utf32 or custom mapping
        xkbcommon::xkb::keysym_to_utf32(keysym)
            .map(|c| char::from_u32(c).unwrap_or('\u{FFFD}'))
            .unwrap_or('\u{FFFD}')
            .to_string()
    }
}

pub enum KeyResult {
    Char(char),
    Text(String),
    Command(KeyCommand),
    ModeChange(KeyboardMode),
    ModifierChange,
    KeyUp(KeyCode),
    None,
}

pub enum KeyCommand {
    Backspace,
    Enter,
    HideKeyboard,
    ShowEmojiPicker,
    ShowLanguageSwitcher,
}
```

## Prediction Engine

```rust
pub struct PredictionEngine {
    language_model: LanguageModel,
    swipe_engine: SwipeEngine,
    dictionary: Dictionary,
    user_history: UserHistory,
}

impl PredictionEngine {
    /// Get word suggestions based on current input
    pub fn get_suggestions(&self, current_word: &str) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        // 1. Exact matches from dictionary
        suggestions.extend(self.dictionary.lookup_prefix(current_word));

        // 2. User history suggestions
        suggestions.extend(self.user_history.lookup(current_word));

        // 3. AI language model predictions
        if self.language_model.is_loaded() {
            suggestions.extend(self.language_model.predict(current_word));
        }

        // 4. Deduplicate and sort by relevance
        suggestions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        suggestions.dedup_by_key(|s| s.text.clone());
        suggestions.truncate(5);  // Top 5 suggestions

        suggestions
    }

    /// Process swipe gesture trail and return predicted word
    pub fn process_swipe(&self, trail: &[Point<f64, Logical>], keyboard: &KeyboardView) -> Option<String> {
        self.swipe_engine.match_trail(trail, keyboard)
    }

    /// Record word usage for learning
    pub fn record_usage(&mut self, word: &str, context: &str) {
        self.user_history.record(word, context);
    }
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub text: String,
    pub score: f64,
    pub is_from_history: bool,
    pub is_correction: bool,  // Spelling correction
}

pub struct LanguageModel {
    // Loaded on-demand, can be large (hundreds of MB)
    model: Option<Box<dyn LmModel>>,
}

pub trait LmModel {
    fn predict(&self, prefix: &str) -> Vec<(String, f64)>;
}

pub struct SwipeEngine {
    // Pre-computed key positions for each layout
    key_positions: HashMap<KeyboardLayout, KeyPositionMap>,
}

impl SwipeEngine {
    /// Match swipe trail to dictionary words
    pub fn match_trail(
        &self,
        trail: &[Point<f64, Logical>],
        keyboard: &KeyboardView,
    ) -> Option<String> {
        // 1. Extract sequence of keys touched by swipe
        let touched_keys = self.extract_key_sequence(trail, keyboard)?;

        // 2. Find dictionary words matching key sequence
        let candidates = self.find_matching_words(&touched_keys);

        // 3. Score candidates based on trail proximity and key order
        let best = candidates.into_iter()
            .map(|word| {
                let score = self.score_word(&word, trail, keyboard);
                (word, score)
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        best.map(|(word, _)| word)
    }

    fn extract_key_sequence(
        &self,
        trail: &[Point<f64, Logical>],
        keyboard: &KeyboardView,
    ) -> Option<Vec<char>> {
        let mut keys = Vec::new();
        let mut last_key = None;

        for point in trail {
            // Find key at this point
            if let Some(key) = keyboard.key_at_point(*point) {
                if last_key.as_ref() != Some(&key.key_code) {
                    keys.push(key.label.chars().next()?);
                    last_key = Some(key.key_code);
                }
            }
        }

        if keys.is_empty() {
            None
        } else {
            Some(keys)
        }
    }

    fn find_matching_words(&self, key_sequence: &[char]) -> Vec<String> {
        // Query dictionary for words with matching key sequence
        // Fuzzy matching allowed (e.g., "helo" → "hello")
        self.dictionary.lookup_by_keys(key_sequence)
    }

    fn score_word(
        &self,
        word: &str,
        trail: &[Point<f64, Logical>],
        keyboard: &KeyboardView,
    ) -> f64 {
        // Score based on:
        // 1. How closely trail follows the word's keys
        // 2. Trajectory smoothness
        // 3. Word frequency
        // 4. User history

        let mut score = 0.0;

        for (i, ch) in word.chars().enumerate() {
            if let Some(expected_pos) = keyboard.key_position(ch) {
                // Find closest point in trail after time i
                let closest = trail.iter()
                    .skip(i)
                    .min_by(|a, b| {
                        a.distance(expected_pos)
                            .partial_cmp(&b.distance(expected_pos))
                            .unwrap()
                    });

                if let Some(point) = closest {
                    let distance = point.distance(expected_pos);
                    score += (-distance / 50.0).exp();  // 50px tolerance
                }
            }
        }

        score / word.len().max(1) as f64
    }
}
```

## Compositor Integration

```rust
// In rustica-comp
pub struct KeyboardManager {
    keyboard_bus: dbus::Connection,
    visible: bool,
    focused_surface: Option<WlSurface>,
    text_input: Option<ZwpTextInputV3>,
}

impl KeyboardManager {
    /// Show on-screen keyboard
    pub fn show(&mut self) {
        if self.visible {
            return;
        }

        // Request keyboard via D-Bus
        self.keyboard_bus.call_method(
            "org.rustica.Keyboard",
            "/org/rustica/Keyboard",
            "org.rustica.Keyboard",
            "Show",
            &(),
        ).unwrap();

        self.visible = true;
    }

    /// Hide on-screen keyboard
    pub fn hide(&mut self) {
        if !self.visible {
            return;
        }

        self.keyboard_bus.call_method(
            "org.rustica.Keyboard",
            "/org/rustica/Keyboard",
            "org.rustica.Keyboard",
            "Hide",
            &(),
        ).unwrap();

        self.visible = false;
    }

    /// Handle text input focus
    pub fn handle_text_input_focus(&mut self, surface: &WlSurface, text_input: &ZwpTextInputV3) {
        self.focused_surface = Some(surface.clone());
        self.text_input = Some(text_input.clone());

        // Auto-show keyboard
        self.show();
    }

    /// Handle text input blur
    pub fn handle_text_input_blur(&mut self) {
        self.focused_surface = None;
        self.text_input = None;

        // Auto-hide keyboard
        self.hide();
    }

    /// Receive key from virtual keyboard and forward to client
    pub fn handle_keyboard_key(&mut self, key: Key) {
        if let Some(text_input) = &self.text_input {
            match key.result {
                KeyResult::Char(c) => {
                    text_input.commit_string(c.to_string());
                }
                KeyResult::Text(s) => {
                    text_input.commit_string(s);
                }
                KeyResult::Command(KeyCommand::Backspace) => {
                    text_input.delete_surrounding_text(1, 0);
                }
                KeyResult::Command(KeyCommand::Enter) => {
                    // Send enter key via key event
                }
                _ => {}
            }

            text_input.commit();
        }
    }
}
```

## D-Bus Interface

```xml
<!DOCTYPE node PUBLIC "-//freedesktop//DTD D-BUS Object Introspection 1.0//EN"
"http://www.freedesktop.org/standards/dbus/1.0/introspect.dtd">
<node name="/org/rustica/Keyboard">
  <interface name="org.rustica.Keyboard">
    <!-- Show keyboard with specified mode -->
    <method name="Show">
      <arg name="mode" type="s" direction="in"/>
    </method>

    <!-- Hide keyboard -->
    <method name="Hide"/>

    <!-- Set keyboard height (percentage of screen) -->
    <method name="SetHeight">
      <arg name="percent" type="u" direction="in"/>
    </method>

    <!-- Set keyboard layout -->
    <method name="SetLayout">
      <arg name="layout" type="s" direction="in"/>
    </method>

    <!-- Signal emitted when key is pressed -->
    <signal name="KeyPressed">
      <arg name="key" type="s"/>
      <arg name="modifiers" type="u"/>
    </signal>

    <!-- Signal emitted when text is committed -->
    <signal name="TextCommitted">
      <arg name="text" type="s"/>
    </signal>
  </interface>
</node>
```

## Configuration

```toml
# /etc/rustica/keyboard.conf
[general]
# Default keyboard height (25-50% of screen)
default_height_percent = 35

# Enable haptic feedback
haptic_feedback = true

# Enable key sounds
key_sound = false

# Enable predictive text
predictions = true

# Enable swipe typing
swipe_typing = true

# Auto-capitalize sentences
autocapitalize = "sentences"

# Auto-punctuate (double-space for period)
auto_punctuate = true

[appearance]
# Keyboard theme
theme = "dark"

# Key opacity (0.0-1.0)
key_opacity = 0.9

# Show character popups on long press
show_popups = true

[prediction]
# Language model to use
language_model = "en"

# Number of suggestions to show
num_suggestions = 3

# Learn from user typing
learn_from_user = true

[accessibility]
# High contrast mode
high_contrast = false

# Key size multiplier (1.0 = normal, 1.5 = large)
key_size_multiplier = 1.0

# Scanning mode for switch access
scanning_mode = false

# Scanning interval (ms)
scanning_interval = 1000
```

## Accessibility Features

1. **Scanning Mode**: Single-switch access with row/column scanning
2. **Resizable Keys**: 0.5x to 2x key size
3. **High Contrast**: WCAG AAA compliant color scheme
4. **Touch Accommodations**: Ignore repeat touches, adjust hold duration
5. **Full Keyboard Access**: Keyboard navigation and selection
6. **Text-to-Speech**: Speak keys and suggestions aloud

## Performance Optimizations

1. **Lazy Loading**: Load language models on-demand
2. **Gesture Coalescing**: Batch swipe trail points
3. **Prediction Caching**: Cache common word predictions
4. **Incremental Rendering**: Render visible keys first, suggestions later
5. **Texture Atlas**: Use GPU-accelerated rendering for key graphics

## Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qwerty_layout_generation() {
        let state = KeyboardState {
            layout: KeyboardLayout::Qwerty,
            mode: KeyboardMode::Text,
            ..Default::default()
        };

        let view = KeyboardView::new(state);
        view.generate_layout();

        assert_eq!(view.keys.len(), 5);  // 5 rows

        // Check first row
        assert_eq!(view.keys[0].len(), 10);
        assert_eq!(view.keys[0][0].label, "1");
        assert_eq!(view.keys[0][9].label, "0");

        // Check spacebar
        let space_key = view.keys[4].iter()
            .find(|k| k.action == KeyAction::Char(' '))
            .unwrap();
        assert!(matches!(space_key.width, KeyWidth::Quadruple));
    }

    #[test]
    fn test_shift_toggle() {
        let mut engine = InputEngine::new(KeyboardLayout::Qwerty).unwrap();

        // Process shift key
        let shift_key = key_shift();
        engine.process_key(&shift_key);

        assert!(matches!(engine.shift_state, ShiftState::On));

        // Type 'a' should produce 'A'
        let a_key = key!("a", KeyCode::KEY_A);
        let result = engine.process_key(&a_key);
        assert!(matches!(result, KeyResult::Char('A')));

        // Shift should auto-disable
        assert!(matches!(engine.shift_state, ShiftState::Off));
    }

    #[test]
    fn test_backspace() {
        let mut engine = InputEngine::new(KeyboardLayout::Qwerty).unwrap();

        let bs_key = key_backspace();
        let result = engine.process_key(&bs_key);

        assert!(matches!(result, KeyResult::Command(KeyCommand::Backspace)));
    }

    #[test]
    fn test_mode_switch() {
        let state = KeyboardState {
            layout: KeyboardLayout::Qwerty,
            mode: KeyboardMode::Text,
            ..Default::default()
        };

        let mut view = KeyboardView::new(state);
        view.generate_layout();

        // Switch to numeric mode
        let mode_key = Key {
            label: "123".to_string(),
            key_code: KeyCode::KEY_UNKNOWN,
            width: KeyWidth::Standard,
            action: KeyAction::ModeChange(KeyboardMode::Numeric),
            popup_label: None,
        };

        let result = view.state.input_engine.process_key(&mode_key);
        assert!(matches!(result, KeyResult::ModeChange(KeyboardMode::Numeric)));

        view.set_mode(KeyboardMode::Numeric);
        view.generate_layout();

        // Should have 4 rows (3x3 grid + hide row)
        assert_eq!(view.keys.len(), 4);
    }

    #[test]
    fn test_prediction() {
        let engine = PredictionEngine::new();

        let suggestions = engine.get_suggestions("hel");
        assert!(suggestions.iter().any(|s| s.text == "hello"));
        assert!(suggestions.iter().any(|s| s.text == "help"));
    }
}
```

## Internationalization

Support for 100+ languages via XKB layouts:

- **Latin scripts**: English, Spanish, French, German, Italian, Portuguese, Dutch, Polish, Czech, Slovak, Romanian, Turkish, Hungarian, Finnish, Swedish, Norwegian, Danish, Icelandic, Estonian, Lithuanian, Latvian, Vietnamese, Indonesian, Tagalog, Swahili
- **Cyrillic scripts**: Russian, Ukrainian, Belarusian, Bulgarian, Serbian, Macedonian, Kazakh
- **Arabic script**: Arabic, Persian, Urdu, Kurdish, Pashto
- **Hebrew**: Hebrew and Yiddish
- **Greek**: Modern Greek
- **Indian scripts**: Hindi (Devanagari), Bengali, Tamil, Telugu, Kannada, Malayalam, Gujarati, Punjabi, Marathi, Odia, Assamese
- **East Asian**: Japanese (Romaji input), Chinese (Pinyin input), Korean
- **Other**: Thai, Lao, Khmer, Burmese, Georgian, Armenian, Amharic

## Dependencies

```toml
[dependencies]
smithay = { git = "https://github.com/Smithay/smithay" }
zbus = "4"
xkbcommon = "0.5"
libc = "0.2"

# For prediction (optional, loaded on-demand)
candle = { version = "0.3", optional = true }
half = { version = "2.3", optional = true }
```

## Future Enhancements

1. **Voice Input**: Speech-to-text integration
2. **Handwriting Recognition**: Draw characters with finger/stylus
3. **Split Keyboard**: For tablets in landscape mode
4. **Floating Keyboard**: Draggable keyboard
5. **Custom Layouts**: User-defined keyboard layouts
6. **Emoji Suggestions**: Contextual emoji recommendations
7. **Clipboard Integration**: Quick paste from clipboard history
