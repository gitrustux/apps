# File Manager Prototype Specification

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Type**: Quick Prototype (UX Validation)
**Timeline**: 1 week (concurrent with shell development)

## Overview

This is a **quick-and-dirty prototype** of the file manager to validate UX design decisions. It is **not production code** and will be replaced with the full implementation in Phase 6.

## Prototype Goals

- Validate directory navigation UX
- Test file/folder operations flow
- Validate icon/list view toggles
- Test drag-and-drop interactions
- Get feedback on visual design
- **DO NOT** implement advanced features (network locations, search, etc.)

## Minimal Feature Set

### What to Build

```rust
// Basic file operations only

pub struct FileManagerPrototype {
    /// Current directory path
    current_path: PathBuf,

    /// View mode
    view_mode: ViewMode,

    /// Selected items
    selected: Vec<PathBuf>,

    /// Clipboard for copy/paste
    clipboard: Option<Clipboard>,
}

pub enum ViewMode {
    Icon,
    List,
}

pub enum Clipboard {
    Copy { items: Vec<PathBuf> },
    Cut { items: Vec<PathBuf> },
}

// Feature checklist
// ✓ Directory navigation (up, back, forward)
// ✓ Enter directory (double-click or Enter key)
// ✓ Create new folder
// ✓ Delete files/folders
// ✓ Rename files/folders
// ✓ Copy/paste files
// ✓ Cut/paste files
// ✓ Icon view vs list view toggle
// ✓ Basic drag-and-drop (move files)
// ✗ Advanced search (skip)
// ✗ Network locations (skip)
// ✗ Archive handling (skip)
// ✗ Properties dialog (skip)
// ✗ Bookmarks (skip)
```

## UI Layout (Simplified)

```
┌─────────────────────────────────────────────────────────────────┐
│ Files                                    📁  [Search]          │
├─────────────────────────────────────────────────────────────────┤
│  ◀ ▶  │  ☰  │  🔍  │                │       │               │
│  Back│ Forward│ View│  Path: /home/user   │ Up   │ New Folder  │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐       │
│  │ 📁 │ │ 📄 │ │ 📄 │ │ 📁 │ │ 📁 │ │ 🖼️ │       │
│  │Docs│ │todo│ │note│ │Down│ │Musc│ │photo│       │
│  └─────┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘       │
│  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐       │
│  │ 📁 │ │ 📁 │ │ 📁 │ │ 📁 │ │ 📁 │ │ 📄 │       │
│  │Vidé│ │Photo│ │Docu│ │Down│ │.conf│ │read│       │
│  └─────┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘       │
└─────────────────────────────────────────────────────────────────┘
│  /home/user/Documents  -  8 items                             │
```

## Simplified Code Structure

```
/var/www/rustux.com/prod/apps/gui/prototypes/rustica-files/
├── Cargo.toml
└── src/
    ├── main.rs               # Entry point - just create window and run
    ├── file_manager.rs       # Basic state management
    ├── navigation.rs         # Directory navigation
    ├── operations.rs         # File operations (copy, move, delete, rename)
    └── ui.rs                 # Simple UI rendering (no proper widgets, just draw)
```

## Implementation Approach

### Use Temporary/Stub UI

```rust
// DON'T build proper widget system
// DO draw directly to surface

impl FileManagerPrototype {
    pub fn render(&mut self, surface: &WaylandSurface) {
        // Clear background
        surface.clear(theme.colors.background);

        // Draw breadcrumb path
        self.draw_breadcrumb(surface);

        // Draw toolbar buttons (just rectangles with text)
        self.draw_toolbar(surface);

        // Draw file icons (grid layout)
        if let ViewMode::Icon = self.view_mode {
            self.draw_icon_grid(surface);
        } else {
            self.draw_list_view(surface);
        }

        // Draw status bar
        self.draw_status_bar(surface);

        surface.commit();
    }

    fn draw_breadcrumb(&self, surface: &WaylandSurface) {
        let mut x = 200;
        let y = 60;

        // Draw root
        surface.draw_text(x, y, "/");

        // Draw path components
        for component in self.current_path.components() {
            x += surface.text_width("/") + 8;
            surface.draw_text(x, y, &component.to_string_lossy());
        }
    }

    fn draw_icon_grid(&self, surface: &WaylandSurface) {
        let mut x = 24;
        let mut y = 120;
        let icon_size = 96;
        let spacing = 24;

        for entry in self.current_entries() {
            if x + icon_size > surface.width() {
                x = 24;
                y += icon_size + spacing + 24;  // +24 for text
            }

            // Draw icon background if selected
            if self.selected.contains(&entry.path) {
                surface.fill_rect(x - 4, y - 4, icon_size + 8, icon_size + 8, theme.colors.primary);
            }

            // Draw icon (use pre-rendered icon images)
            surface.draw_icon(x, y, icon_size, entry.icon_name);

            // Draw filename
            surface.draw_text(x, y + icon_size + 4, &entry.name);

            x += icon_size + spacing;
        }
    }
}
```

### Basic Operations

```rust
impl FileManagerPrototype {
    // Navigation
    pub fn navigate_up(&mut self) {
        if let Some(parent) = self.current_path.parent() {
            self.current_path = parent.to_path_buf();
            self.selected.clear();
        }
    }

    pub fn enter_directory(&mut self, path: PathBuf) {
        if path.is_dir() {
            self.current_path = path.clone();
            self.selected.clear();
        } else {
            // Open file with default app
            open::that(path).ok();
        }
    }

    // File operations
    pub fn create_folder(&mut self) {
        let name = self.prompt_for_name("New Folder");

        if let Some(name) = name {
            let mut path = self.current_path.clone();
            path.push(&name);
            fs::create_dir(&path).ok();
        }
    }

    pub fn delete_selected(&mut self) {
        for path in &self.selected {
            if path.is_dir() {
                fs::remove_dir_all(path).ok();
            } else {
                fs::remove_file(path).ok();
            }
        }
        self.selected.clear();
    }

    pub fn rename_selected(&mut self) {
        if self.selected.len() == 1 {
            let path = &self.selected[0];
            let new_name = self.prompt_for_rename(path.file_name());

            if let Some(new_name) = {
                let mut new_path = path.parent().unwrap();
                new_path.push(&new_name);
                fs::rename(path, &new_path).ok();
            }
        }
    }

    pub fn copy_selected(&mut self) {
        self.clipboard = Some(Clipboard::Copy {
            items: self.selected.clone(),
        });
    }

    pub fn cut_selected(&mut self) {
        self.clipboard = Some(Clipboard::Cut {
            items: self.selected.clone(),
        });
    }

    pub fn paste(&mut self) {
        if let Some(clipboard) = &self.clipboard {
            match clipboard {
                Clipboard::Copy { items } => {
                    for item in items {
                        let filename = item.file_name().unwrap();
                        let mut dest = self.current_path.clone();
                        dest.push(filename);

                        // Copy recursively
                        if item.is_dir() {
                            dir_copy::copy_dir(item, &dest, &OverwriteResult).ok();
                        } else {
                            fs::copy(item, &dest).ok();
                        }
                    }
                }
                Clipboard::Cut { items } => {
                    for item in items {
                        let filename = item.file_name().unwrap();
                        let mut dest = self.current_path.clone();
                        dest.push(filename);
                        fs::rename(item, &dest).ok();
                    }
                }
            }
        }
    }
}
```

### Input Handling

```rust
impl FileManagerPrototype {
    pub fn handle_key_event(&mut self, key: KeyEvent) {
        match key {
            // Navigation
            KeyEvent { key: KeyCode::Back, .. } => {
                self.navigate_up();
            }

            KeyEvent { key: KeyCode::Enter, .. } => {
                if let Some(path) = self.selected.first() {
                    self.enter_directory(path.clone());
                }
            }

            KeyEvent { key: KeyCode::Delete, modifiers: Modifiers::SHIFT } => {
                self.delete_selected();
            }

            // Operations
            KeyEvent { key: KeyCode::N, modifiers: Modifiers::CONTROL } => {
                self.create_folder();
            }

            KeyEvent { key: KeyCode::C, modifiers: Modifiers::CONTROL } => {
                self.copy_selected();
            }

            KeyEvent { key: KeyCode::X, modifiers: Modifiers::CONTROL } => {
                self.cut_selected();
            }

            KeyEvent { key: KeyCode::V, modifiers: Modifiers::CONTROL } => {
                self.paste();
            }

            KeyEvent { key: KeyCode::F2, .. } => {
                self.rename_selected();
            }

            // View toggle
            KeyEvent { key: KeyCode::Key1, modifiers: Modifiers::CONTROL } => {
                self.view_mode = ViewMode::Icon;
            }

            KeyEvent { key: KeyCode::Key2, modifiers: Modifiers::CONTROL } => {
                self.view_mode = ViewMode::List;
            }

            _ => {}
        }
    }

    pub fn handle_click(&mut self, position: Point) {
        // Find clicked item
        if let Some(path) = self.find_item_at(position) {
            // Single click: select
            self.selected.clear();
            self.selected.push(path);
        }
    }

    pub fn handle_double_click(&mut self, position: Point) {
        if let Some(path) = self.find_item_at(position) {
            self.enter_directory(path.clone());
        }
    }
}
```

## Quick Icon Loading

```rust
// Use system icons, don't bundle custom ones
// Don't implement proper icon theme - just look in /usr/share/icons

fn load_icon(name: &str) -> Image {
    let icon_paths = vec![
        format!("/usr/share/icons/hicolor/48x48/{}.png", name),
        format!("/usr/share/icons/hicolor/64x64/{}.png", name),
        format!("/usr/share/icons/hicolor/96x96/{}.png", name),
        format!("/usr/share/icons/Adwaita/48x48/{}.png", name),
        format!("/usr/share/pixmaps/{}.png", name),
    ];

    for path in icon_paths {
        if let Ok(image) = Image::load(&path) {
            return image;
        }
    }

    // Fallback icon
    Image::load("/usr/share/icons/hicolor/48x48/text-x-generic.png")
        .unwrap_or_default()
}

fn get_folder_icon() -> Image {
    load_icon("folder")
}

fn get_file_icon(filename: &str) -> Image {
    let ext = filename.rsplit('.').next().unwrap_or("");

    match ext {
        "png" | "jpg" | "jpeg" | "gif" | "svg" => load_icon("image-x-generic"),
        "pdf" => load_icon("application-pdf"),
        "txt" | "md" => load_icon("text-x-generic"),
        "zip" | "tar" | "gz" => load_icon("package-x-generic"),
        _ => load_icon("text-x-generic"),
    }
}
```

## Minimal Dependencies

```toml
[package]
name = "rustica-files"
version = "0.1.0"
edition = "2021"

[dependencies]
# Wayland client - for display
wayland-client = "0.31"

# Basic UI
softbuffer = "0.3"           # Simple surface rendering
rfd = "0.11"                 # File dialogs (for open/save)

# File operations
dirs = "5.0"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# NO proper widget libraries
# NO accessibility (yet)
# NO theming system (just hardcode colors)
# NO proper icons (use system icons)
```

## Testing Checklist

- [ ] Can navigate directories
- [ ] Can create folders
- [ ] Can delete files
- [ ] Can rename files
- [ ] Can copy/paste files
- [ ] Can cut/paste files
- [ ] Icon view works
- [ ] List view works
- [ ] Double-click opens folders
- [ ] Double-click opens files in default app

## Validation Questions

After prototype is complete, gather feedback on:

1. **Navigation**: Is back/forward flow intuitive?
2. **Selection**: Is single-click vs double-click clear?
3. **Visual hierarchy**: Can users distinguish folders from files?
4. **Spacing**: Is 96px icon size + 24px spacing appropriate?
5. **Toolbar**: Are back/up/new folder buttons obvious?
6. **Status bar**: Is item count and path display useful?

## Success Criteria

- [ ] All basic features work
- [ ] No crashes on normal operations
- [ ] Can handle 1000+ files without major lag
- [ ] Navigation feels responsive
- [ ] Feedback gathered from users

## Deliverable

- Working prototype binary
- Screenshot of UI
- List of UX issues discovered
- List of requested changes for full version

## Sign-Off

**Prototype Developer**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅ (for prototype only)

**Note**: This prototype will be discarded. Full implementation in Phase 6.

---

## Timeline

- Day 1: Project structure + basic window
- Day 2: Directory navigation + icon loading
- Day 3: File operations (copy, move, delete)
- Day 4: List view + polish
- Day 5: Testing + feedback gathering

**Total**: 1 week
