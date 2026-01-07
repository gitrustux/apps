# Accessibility Framework Specification

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Component**: Rustica Shell - Accessibility (RAISED PRIORITY)

## Overview

This specification defines how Rustica Shell implements accessibility, ensuring **screen reader compatibility**, **keyboard navigation**, and **WCAG 2.1 AA compliance**. It provides a **unified accessibility API** that all widgets and applications must implement.

## Design Philosophy

1. **Accessibility by Default** - All components are accessible from day one
2. **Platform Compatibility** - Works with Linux AT-SPI, with extensible design for other platforms
3. **Zero Performance Impact** - <5% overhead when enabled, <1% when disabled
4. **Developer Friendly** - Simple API that's hard to use incorrectly

## AT-SPI Integration

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                  Assistive Technologies                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐           │
│  │ Screen Reader│  │   Magnifier  │  │  On-screen   │           │
│  │  (Orca)      │  │   (Magnus)   │  │  Keyboard    │           │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘           │
└─────────┼──────────────────┼──────────────────┼──────────────────┘
          │                  │                  │
          │         AT-SPI Protocol (DBus)
          │                  │
┌─────────┼──────────────────┼──────────────────┼──────────────────┐
│         │      Rustica AT-SPI Registry        │                  │
│         └──────────────────┬──────────────────┘                  │
│                            │                                     │
│  ┌─────────────────────────┼─────────────────────────────────┐  │
│  │              librustica-accessibility                      │  │
│  │  ┌──────────────────────────────────────────────────────┐ │  │
│  │  │         Accessibility Event Bus                       │ │  │
│  │  └──────────────────────────────────────────────────────┘ │  │
│  │  ┌──────────────────────────────────────────────────────┐ │  │
│  │  │         Widget Accessibility Registry                 │ │  │
│  │  └──────────────────────────────────────────────────────┘ │  │
│  └──────────────────────────────────────────────────────────┘  │
│                            │                                     │
│  ┌─────────────────────────┼─────────────────────────────────┐  │
│  │         Applications (Widgets, Windows)                    │  │
│  └────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### AT-SPI Registry

```rust
use atspi::accessible::Accessible;
use zbus::{Connection, fdo};

pub struct AtSpiRegistry {
    connection: Connection,
    enabled: bool,
    desktop: Option<Accessible>,
}

impl AtSpiRegistry {
    /// Initialize AT-SPI registry
    pub async fn new() -> Result<Self> {
        // Connect to session bus
        let connection = Connection::session().await?;

        // Check if AT-SPI is available
        let enabled = Self::check_atspi_available(&connection).await?;

        let registry = Self {
            connection,
            enabled,
            desktop: None,
        };

        if enabled {
            registry.register_desktop().await?;
        }

        Ok(registry)
    }

    /// Check if AT-SPI is available on the bus
    async fn check_atspi_available(conn: &Connection) -> Result<bool> {
        match fdo::DBusProxy::new(conn).await {
            Ok(proxy) => {
                // Check if org.a11y.Bus is available
                if let Ok(name_has_owner) = proxy.name_has_owner("org.a11y.Bus").await {
                    return Ok(name_has_owner);
                }
            }
            Err(_) => return Ok(false),
        }

        Ok(false)
    }

    /// Register desktop root accessible
    async fn register_desktop(&mut self) -> Result<()> {
        let desktop = Accessible::new(
            &self.connection,
            "org.rustica.Shell",
            "/org/rustica/shell/desktop",
        ).await?;

        desktop.set_name("Rustica Desktop").await?;
        desktop.set_role(Role::Desktop).await?;

        self.desktop = Some(desktop);
        Ok(())
    }

    /// Register an application
    pub async fn register_application(
        &self,
        app_id: &str,
        pid: u32,
    ) -> Result<Accessible> {
        let app = Accessible::new(
            &self.connection,
            app_id,
            &format!("/org/rustica/shell/apps/{}", pid),
        ).await?;

        app.set_role(Role::Application).await?;
        app.set_toolkit_name("librustica").await?;
        app.set_toolkit_version(env!("CARGO_PKG_VERSION")).await?;
        app.set_pid(pid as i32).await?;

        // Add as child of desktop
        if let Some(ref desktop) = self.desktop {
            desktop.add_child(&app).await?;
        }

        Ok(app)
    }

    /// Register a window
    pub async fn register_window(
        &self,
        app: &Accessible,
        window_id: &str,
        title: &str,
    ) -> Result<Accessible> {
        let window = Accessible::new(
            &self.connection,
            app.bus_name(),
            &format!("{}/windows/{}", app.path(), window_id),
        ).await?;

        window.set_name(title).await?;
        window.set_role(Role::Window).await?;

        // Add as child of application
        app.add_child(&window).await?;

        Ok(window)
    }

    /// Register a widget
    pub async fn register_widget(
        &self,
        parent: &Accessible,
        widget: &dyn AccessibilityAccessible,
    ) -> Result<Accessible> {
        let accessible = Accessible::new(
            &self.connection,
            parent.bus_name(),
            &format!("{}/widgets/{}", parent.path(), widget.id()),
        ).await?;

        // Set basic properties
        accessible.set_name(widget.name()).await?;
        accessible.set_role(widget.role()).await?;
        accessible.set_description(widget.description()).await?;

        // Set widget-specific properties
        widget.set_accessible_properties(&accessible).await?;

        // Add as child
        parent.add_child(&accessible).await?;

        Ok(accessible)
    }
}
```

## Accessibility API

### Core Trait

```rust
use async_trait::async_trait;

/// Core accessibility trait that all widgets must implement
#[async_trait]
pub trait AccessibilityAccessible: Send + Sync {
    /// Unique identifier for this accessible
    fn id(&self) -> String;

    /// Accessible name (label text, button text, etc.)
    fn name(&self) -> String;

    /// Accessible description (tooltip, help text, etc.)
    fn description(&self) -> String;

    /// Accessibility role
    fn role(&self) -> Role;

    /// Set additional AT-SPI properties
    async fn set_accessible_properties(&self, accessible: &Accessible) -> Result<()> {
        // Default implementation - override for specific widgets
        Ok(())
    }

    /// Handle accessibility action
    async fn handle_action(&self, action: AccessibilityAction) -> Result<()>;
}
```

### Role Definitions

```rust
pub enum Role {
    // Container roles
    Window,
    Dialog,
    Frame,
    Panel,

    // Widget roles
    Button,
    CheckBox,
    RadioButton,
    ComboBox,
    TextBox,
    TextArea,
    Slider,
    ProgressBar,
    Spinner,
    Switch,

    // Navigation roles
    Menu,
    MenuItem,
   MenuBar,
    TabBar,
    Tab,
    Tree,
    TreeItem,
    Table,
    TableCell,
    TableRow,

    // Text roles
    Paragraph,
    Heading,
    Label,
    Link,

    // Special roles
    StatusBar,
    ToolBar,
    ToolTip,
    ScrollBar,
    SplitPane,

    // Fallback
    Unknown,
}
```

### State System

```rust
pub struct AccessibleState {
    // Visibility
    pub visible: bool,
    pub showing: bool,
    pub hidden: bool,

    // Focus
    pub focused: bool,
    pub focusable: bool,

    // Interaction
    pub enabled: bool,
    pub sensitive: bool,
    pub editable: bool,
    pub expandable: bool,
    pub expanded: bool,
    pub checkable: bool,
    pub checked: bool,

    // Selection
    pub selected: bool,
    pub selectable: bool,
    pub multi_selectable: bool,

    // Layout
    pub horizontal: bool,
    pub vertical: bool,

    // Other
    pub active: bool,
    pub busy: bool,
    pub modal: bool,
    pub required: bool,
}

impl AccessibleState {
    pub fn to_atspi_state_set(&self) -> atspi::StateSet {
        let mut states = atspi::StateSet::new();

        if self.visible { states.insert(atspi::State::Visible); }
        if self.showing { states.insert(atspi::State::Showing); }
        if self.focused { states.insert(atspi::State::Focused); }
        if self.focusable { states.insert(atspi::State::Focusable); }
        if self.enabled { states.insert(atspi::State::Enabled); }
        if self.editable { states.insert(atspi::State::Editable); }
        if self.expanded { states.insert(atspi::State::Expanded); }
        if self.checked { states.insert(atspi::State::Checked); }
        if self.selected { states.insert(atspi::State::Selected); }
        if self.horizontal { states.insert(atspi::State::Horizontal); }
        // ... etc

        states
    }
}
```

### Actions

```rust
pub enum AccessibilityAction {
    // Focus actions
    Focus,
    GrabFocus,

    // Component actions
    Click,
    Press,
    Release,

    // Text actions
    Copy,
    Cut,
    Paste,

    // Value actions
    Increment,
    Decrement,
    SetValue(f64),

    // Selection actions
    Select,
    Deselect,
    Toggle,

    // Window actions
    Close,
    Minimize,
    Maximize,
    Restore,
}

pub trait AccessibilityActions {
    /// Get supported actions
    fn supported_actions(&self) -> Vec<AccessibilityAction>;

    /// Perform an action
    async fn do_action(&self, action: AccessibilityAction) -> Result<()>;

    /// Get action description (for screen readers)
    fn action_description(&self, action: &AccessibilityAction) -> String;
}
```

## Widget Accessibility Implementation

### Button

```rust
pub struct Button {
    label: String,
    on_click: Box<dyn Fn()>,
    focused: bool,
}

#[async_trait]
impl AccessibilityAccessible for Button {
    fn id(&self) -> String {
        format!("button_{}", self.label.replace(' ', "_"))
    }

    fn name(&self) -> String {
        self.label.clone()
    }

    fn description(&self) -> String {
        format!("Button: {}", self.label)
    }

    fn role(&self) -> Role {
        Role::Button
    }

    async fn set_accessible_properties(&self, accessible: &Accessible) -> Result<()> {
        // Set state
        let state = AccessibleState {
            visible: true,
            showing: true,
            focusable: true,
            focused: self.focused,
            enabled: true,
            ..Default::default()
        };
        accessible.set_state(state.to_atspi_state_set()).await?;

        // Set supported actions
        accessible.set_actions(vec![
            AccessibilityAction::Focus,
            AccessibilityAction::Click,
        ]).await?;

        Ok(())
    }

    async fn handle_action(&self, action: AccessibilityAction) -> Result<()> {
        match action {
            AccessibilityAction::Click => {
                (self.on_click)();
                Ok(())
            }
            _ => Err(Error::UnsupportedAction),
        }
    }
}

impl AccessibilityActions for Button {
    fn supported_actions(&self) -> Vec<AccessibilityAction> {
        vec![
            AccessibilityAction::Focus,
            AccessibilityAction::Click,
        ]
    }

    async fn do_action(&self, action: AccessibilityAction) -> Result<()> {
        self.handle_action(action).await
    }

    fn action_description(&self, action: &AccessibilityAction) -> String {
        match action {
            AccessibilityAction::Click => format!("Click {}", self.label),
            AccessibilityAction::Focus => format!("Focus {} button", self.label),
            _ => String::new(),
        }
    }
}
```

### TextField

```rust
pub struct TextField {
    value: String,
    placeholder: String,
    focused: bool,
    selection: Option<(usize, usize)>,
}

#[async_trait]
impl AccessibilityAccessible for TextField {
    fn id(&self) -> String {
        "text_field".into()
    }

    fn name(&self) -> String {
        if !self.placeholder.is_empty() {
            self.placeholder.clone()
        } else {
            "Text input".into()
        }
    }

    fn description(&self) -> String {
        format!("Text field: {}", self.value)
    }

    fn role(&self) -> Role {
        Role::TextBox
    }

    async fn set_accessible_properties(&self, accessible: &Accessible) -> Result<()> {
        let state = AccessibleState {
            visible: true,
            showing: true,
            focusable: true,
            focused: self.focused,
            editable: true,
            enabled: true,
            ..Default::default()
        };
        accessible.set_state(state.to_atspi_state_set()).await?;

        // Set text content
        accessible.set_text_content(&self.value).await?;

        // Set selection
        if let Some((start, end)) = self.selection {
            accessible.set_text_selection(start, end).await?;
        }

        // Set caret position
        accessible.set_caret_position(self.value.len()).await?;

        Ok(())
    }

    async fn handle_action(&self, action: AccessibilityAction) -> Result<()> {
        match action {
            AccessibilityAction::Copy => {
                // Copy to clipboard
                clipboard::set_text(&self.value);
                Ok(())
            }
            _ => Err(Error::UnsupportedAction),
        }
    }
}
```

### ListBox

```rust
pub struct ListBox {
    items: Vec<String>,
    selected: usize,
    focused: bool,
}

#[async_trait]
impl AccessibilityAccessible for ListBox {
    fn id(&self) -> String {
        "list_box".into()
    }

    fn name(&self) -> String {
        "List".into()
    }

    fn description(&self) -> String {
        format!("List with {} items", self.items.len())
    }

    fn role(&self) -> Role {
        Role::ListBox
    }

    async fn set_accessible_properties(&self, accessible: &Accessible) -> Result<()> {
        let state = AccessibleState {
            visible: true,
            showing: true,
            focusable: true,
            focused: self.focused,
            enabled: true,
            selectable: true,
            ..Default::default()
        };
        accessible.set_state(state.to_atspi_state_set()).await?;

        // Add children for each item
        for (i, item) in self.items.iter().enumerate() {
            let item_accessible = accessible.create_child(i).await?;
            item_accessible.set_name(item).await?;
            item_accessible.set_role(Role::ListItem).await?;

            let item_state = AccessibleState {
                selectable: true,
                selected: i == self.selected,
                ..Default::default()
            };
            item_accessible.set_state(item_state.to_atspi_state_set()).await?;
        }

        Ok(())
    }

    async fn handle_action(&self, action: AccessibilityAction) -> Result<()> {
        match action {
            AccessibilityAction::Select => {
                // Select item
                Ok(())
            }
            _ => Err(Error::UnsupportedAction),
        }
    }
}
```

## Keyboard Navigation

### Focus Manager

```rust
pub struct FocusManager {
    focus_chain: Vec<Box<dyn Focusable>>,
    current_index: usize,
    wrap_around: bool,
}

pub trait Focusable: Send + Sync {
    fn focus(&self);
    fn blur(&self);
    fn is_focused(&self) -> bool;
    fn is_focusable(&self) -> bool;
}

impl FocusManager {
    pub fn new(wrap_around: bool) -> Self {
        Self {
            focus_chain: Vec::new(),
            current_index: 0,
            wrap_around,
        }
    }

    /// Add widget to focus chain
    pub fn add(&mut self, widget: Box<dyn Focusable>) {
        self.focus_chain.push(widget);
    }

    /// Move focus forward
    pub fn focus_next(&mut self) {
        if self.focus_chain.is_empty() {
            return;
        }

        // Blur current
        if let Some(current) = self.focus_chain.get(self.current_index) {
            current.blur();
        }

        // Move to next
        self.current_index = if self.wrap_around {
            (self.current_index + 1) % self.focus_chain.len()
        } else {
            (self.current_index + 1).min(self.focus_chain.len() - 1)
        };

        // Focus new
        if let Some(next) = self.focus_chain.get(self.current_index) {
            next.focus();
        }
    }

    /// Move focus backward
    pub fn focus_previous(&mut self) {
        if self.focus_chain.is_empty() {
            return;
        }

        // Blur current
        if let Some(current) = self.focus_chain.get(self.current_index) {
            current.blur();
        }

        // Move to previous
        self.current_index = if self.wrap_around {
            self.current_index.checked_sub(1)
                .unwrap_or(self.focus_chain.len() - 1)
        } else {
            self.current_index.saturating_sub(1)
        };

        // Focus new
        if let Some(prev) = self.focus_chain.get(self.current_index) {
            prev.focus();
        }
    }

    /// Handle keyboard navigation
    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        match key {
            KeyEvent { key: KeyCode::Tab, modifiers: Modifiers::SHIFT } => {
                self.focus_previous();
                true
            }
            KeyEvent { key: KeyCode::Tab, .. } => {
                self.focus_next();
                true
            }
            _ => false,
        }
    }
}
```

### Keyboard Shortcuts

```rust
pub struct KeyboardShortcut {
    pub key: KeyCode,
    pub modifiers: Modifiers,
    pub description: String,
    pub action: Box<dyn Fn()>,
}

pub struct ShortcutManager {
    shortcuts: Vec<KeyboardShortcut>,
}

impl ShortcutManager {
    pub fn new() -> Self {
        Self {
            shortcuts: Vec::new(),
        }
    }

    pub fn register(&mut self, shortcut: KeyboardShortcut) {
        self.shortcuts.push(shortcut);
    }

    pub fn handle_event(&self, event: KeyEvent) -> bool {
        for shortcut in &self.shortcuts {
            if shortcut.key == event.key && shortcut.modifiers == event.modifiers {
                (shortcut.action)();
                return true;
            }
        }
        false
    }

    /// Get all shortcuts for help menu
    pub fn get_all(&self) -> &[KeyboardShortcut] {
        &self.shortcuts
    }
}

// Default accessibility shortcuts
pub fn register_default_shortcuts(manager: &mut ShortcutManager) {
    // Screen reader
    manager.register(KeyboardShortcut {
        key: KeyCode::Super,
        modifiers: Modifiers::ALT | Modifiers::CONTROL,
        description: "Toggle screen reader".into(),
        action: Box::new(|| {
            accessibility::toggle_screen_reader();
        }),
    });

    // Magnifier
    manager.register(KeyboardShortcut {
        key: KeyCode::Equal,
        modifiers: Modifiers::SUPER | Modifiers::ALT,
        description: "Toggle magnifier".into(),
        action: Box::new(|| {
            accessibility::toggle_magnifier();
        }),
    });

    // High contrast
    manager.register(KeyboardShortcut {
        key: KeyCode::H,
        modifiers: Modifiers::SUPER | Modifiers::CONTROL,
        description: "Toggle high contrast".into(),
        action: Box::new(|| {
            theme::toggle_high_contrast();
        }),
    });
}
```

## Screen Reader Support

### Event Emission

```rust
pub struct AccessibilityEventBus {
    subscribers: Vec<Box<dyn AccessibilityEventSubscriber>>,
}

pub trait AccessibilityEventSubscriber: Send + Sync {
    fn on_event(&self, event: AccessibilityEvent);
}

pub enum AccessibilityEvent {
    /// Object gained focus
    FocusGained {
        object: String,
        name: String,
        role: Role,
    },

    /// Object lost focus
    FocusLost {
        object: String,
    },

    /// Object state changed
    StateChanged {
        object: String,
        state: AccessibleState,
    },

    /// Text changed
    TextChanged {
        object: String,
        position: usize,
        length: usize,
        text: String,
    },

    /// Window opened
    WindowOpened {
        window: String,
        title: String,
    },

    /// Window closed
    WindowClosed {
        window: String,
    },

    /// Notification shown
    Notification {
        title: String,
        body: String,
        urgency: Urgency,
    },
}

impl AccessibilityEventBus {
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
        }
    }

    pub fn subscribe(&mut self, subscriber: Box<dyn AccessibilityEventSubscriber>) {
        self.subscribers.push(subscriber);
    }

    pub fn emit(&self, event: AccessibilityEvent) {
        for subscriber in &self.subscribers {
            subscriber.on_event(event.clone());
        }
    }

    /// Send to AT-SPI
    async fn send_to_atspi(&self, event: AccessibilityEvent) -> Result<()> {
        match event {
            AccessibilityEvent::FocusGained { object, name, role } => {
                atspi::emit_focus_event(&object, &name, role).await?;
            }
            AccessibilityEvent::Notification { title, body, urgency } => {
                atspi::emit_notification(&title, &body, urgency).await?;
            }
            _ => {}
        }
        Ok(())
    }
}
```

### AT-SPI Event Proxy

```rust
pub struct AtSpiEventProxy {
    connection: Connection,
}

impl AtSpiEventProxy {
    pub async fn new() -> Result<Self> {
        let connection = Connection::session().await?;
        Ok(Self { connection })
    }

    /// Emit focus event
    pub async fn emit_focus(
        &self,
        accessible: &Accessible,
        gained: bool,
    ) -> Result<()> {
        let event_name = if gained {
            "Focus"
        } else {
            "Focus:Removed"
        };

        let message = dbus::Message::signal(
            accessible.path(),
            "org.a11y.atspi.Event.Focus",
            event_name,
        )
        .append(accessible)
        .append(0);  // detail

        self.connection.send(message).await?;
        Ok(())
    }

    /// Emit property changed event
    pub async fn emit_property_changed(
        &self,
        accessible: &Accessible,
        property: &str,
        value: Value,
    ) -> Result<()> {
        let message = dbus::Message::signal(
            accessible.path(),
            "org.a11y.atspi.Event.Object",
            "PropertyChanged",
        )
        .append(accessible)
        .append(property)
        .append(value);

        self.connection.send(message).await?;
        Ok(())
    }

    /// Emit text changed event
    pub async fn emit_text_changed(
        &self,
        accessible: &Accessible,
        position: usize,
        length: usize,
        text: &str,
    ) -> Result<()> {
        let message = dbus::Message::signal(
            accessible.path(),
            "org.a11y.atspi.Event.Object",
            "TextChanged",
        )
        .append(accessible)
        .append("insert")  // type of change
        .append(position as i32)
        .append(length as i32)
        .append(text);

        self.connection.send(message).await?;
        Ok(())
    }
}
```

## Testing Accessibility

### Automated Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_accessible() {
        let button = Button::new("Click me", || {});
        let accessible = button.to_accessible().unwrap();

        assert_eq!(accessible.name(), "Click me");
        assert_eq!(accessible.role(), Role::Button);
        assert!(accessible.state().focusable);
    }

    #[test]
    fn test_focus_chain() {
        let mut manager = FocusManager::new(true);
        manager.add(Box::new(Button::new("A", || {})));
        manager.add(Box::new(Button::new("B", || {})));
        manager.add(Box::new(Button::new("C", || {})));

        // Focus next
        manager.focus_next();
        assert!(manager.is_focused("A"));

        manager.focus_next();
        assert!(manager.is_focused("B"));

        // Wrap around
        manager.focus_next();
        manager.focus_next();
        assert!(manager.is_focused("A"));
    }

    #[test]
    fn test_atspi_registration() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let registry = AtSpiRegistry::new().await.unwrap();

            let app = registry.register_application(
                "org.test.App",
                1234,
            ).await.unwrap();

            assert_eq!(app.role(), Role::Application);
            assert_eq!(app.pid(), 1234);
        });
    }
}
```

## Performance Requirements

| Metric | Target | Measurement |
|--------|--------|-------------|
| AT-SPI overhead | <5% | With accessibility enabled |
| Disabled overhead | <1% | With accessibility disabled |
| Focus move | <10ms | Tab press to focus change |
| Event emit | <1ms | Event to bus |
| State announce | <50ms | State change to spoken |

## File Structure

```
/var/www/rustux.com/prod/apps/gui/
├── libs/librustica-accessibility/
│   ├── src/
│   │   ├── mod.rs
│   │   ├── registry.rs         # AT-SPI registry
│   │   ├── accessible.rs       # Core accessibility trait
│   │   ├── role.rs             # Role definitions
│   │   ├── state.rs            # State system
│   │   ├── action.rs           # Action system
│   │   ├── focus.rs            # Focus management
│   │   ├── events.rs           # Event emission
│   │   └── widgets/
│   │       ├── button.rs
│   │       ├── text_field.rs
│   │       ├── list_box.rs
│   │       └── ...
│   └── Cargo.toml
│
└── rustica-comp/src/
    └── accessibility/
        ├── mod.rs
        ├── event_proxy.rs      # AT-SPI event proxy
        └── screen_reader.rs    # Screen reader integration
```

## Example Usage

```rust
use librustica_accessibility::*;

// Initialize accessibility
let registry = AtSpiRegistry::new().await?;

// Register app
let app = registry.register_application(
    "com.example.myapp",
    std::process::id(),
).await?;

// Create accessible button
let button = Button::new("Save", || save());
let accessible_button = registry.register_widget(&app, &button).await?;

// Set up focus
let mut focus_manager = FocusManager::new(true);
focus_manager.add(Box::new(button.clone()));

// Handle events
focus_manager.handle_key_event(KeyEvent {
    key: KeyCode::Tab,
    modifiers: Modifiers::empty(),
});
```

## Success Criteria

- [ ] AT-SPI registry initializes correctly
- [ ] All widgets implement AccessibilityAccessible
- [ ] Screen reader announces all UI elements
- [ ] Keyboard navigation works throughout
- [ ] Focus tracking is accurate
- [ ] Events emit in <1ms
- [ ] Performance overhead <5%
- [ ] WCAG 2.1 AA compliant
- [ ] Tests pass

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| AT-SPI not available | Graceful degradation, disable features |
| High CPU usage | Lazy registration, event throttling |
| Screen reader conflicts | Detect other ATs, coordinate |
| Developer burden | Provide default implementations, macros |

## Sign-Off

**Architect**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅

---

## References

- [AT-SPI 2 Specification](https://accessibility.linuxfoundation.org/a11yspecs/atspi2/atspi2.html)
- [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
- [GNOME Accessibility](https://developer.gnome.org/gtk3/stable/GtkAccessible.html)
- [Qt Accessibility](https://doc.qt.io/qt-6/accessible.html)
- [Orca Screen Reader](https://wiki.gnome.org/Projects/Orca)
