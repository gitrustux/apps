# App Launcher (rustica-launcher) Specification

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Component**: Rustica Shell - Application Launcher

## Overview

This specification defines the application launcher for Rustica Shell, providing **grid view of apps**, **instant search**, **category filtering**, and **recently used apps**. It ensures **<100ms search results** and **intuitive navigation**.

## Architecture

```
┌──────────────────────────────────────────────────────────────────────┐
│                        Application Launcher                          │
├──────────────────────────────────────────────────────────────────────┤
│  🔍  Search Applications...                           [Category ▼]  │
├──────────────────────────────────────────────────────────────────────┤
│  Recently Used                                                       │
│  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐                                        │
│  │ 🏠 │ │ 📁 │ │ 🌐 │ │ 📝 │                                        │
│  │Home │ │Files│ │ Web │ │Edit │                                        │
│  └─────┘ └─────┘ └─────┘ └─────┘                                        │
├──────────────────────────────────────────────────────────────────────┤
│  All Applications                                                    │
│  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐  │
│  │ 🎮 │ │ 📊 │ │ 💬 │ │ 🎨 │ │ 📧 │ │ 🎵 │ │ ⚙️ │ │ 📺 │  │
│  │Game │ │Chart│ │Chat│ │Draw│ │Mail│ │Music││Set ││Video│  │
│  └─────┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘  │
│  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐  │
│  │ 📷 │ │ 🔬 │ │ 💰 │ │ 📓 │ │ 🎲 │ │ 🔧 │ │ 🛒 │ │ 🚀 │  │
│  │Photo│ │Sci │ │Fin │ │Note│ │RNG ││Tool││Shop││Term│  │
│  └─────┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘  │
└──────────────────────────────────────────────────────────────────────┘
                    ◄─────── 128px icon + 16px gap ─────►
```

## Launcher State

```rust
pub struct Launcher {
    /// All installed applications
    apps: Vec<AppEntry>,

    /// Search query
    search_query: String,

    /// Selected category filter
    category_filter: Option<Category>,

    /// Filtered applications
    filtered_apps: Vec<AppEntry>,

    /// Recently used apps
    recent_apps: Vec<AppEntry>,

    /// Selected app index
    selected_index: usize,

    /// Scroll position
    scroll_offset: f32,

    /// Is visible
    visible: bool,

    /// Animation state
    animation_state: AnimationState,
}

pub struct AppEntry {
    /// App ID
    id: String,

    /// Display name
    name: String,

    /// Icon
    icon: Icon,

    /// Categories
    categories: Vec<Category>,

    /// Description
    description: String,

    /// Executable path
    exec: String,

    /// Keywords for search
    keywords: Vec<String>,

    /// Last used time
    last_used: Option<DateTime<Utc>>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Category {
    All,
    Accessories,
    AudioVideo,
    Development,
    Education,
    Games,
    Graphics,
    Network,
    Office,
    Science,
    Settings,
    System,
    Utility,
}

pub enum AnimationState {
    Hidden,
    Showing { progress: f32 },
    Visible,
    Hiding { progress: f32 },
}
```

## Search Functionality

```rust
impl Launcher {
    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query.to_lowercase();
        self.filter_apps();
        self.selected_index = 0;
    }

    fn filter_apps(&mut self) {
        self.filtered_apps.clear();

        for app in &self.apps {
            // Check category filter
            if let Some(ref category) = self.category_filter {
                if *category != Category::All && !app.categories.contains(category) {
                    continue;
                }
            }

            // Check search query
            if self.search_query.is_empty() {
                self.filtered_apps.push(app.clone());
                continue;
            }

            // Search in name
            if app.name.to_lowercase().contains(&self.search_query) {
                self.filtered_apps.push(app.clone());
                continue;
            }

            // Search in description
            if app.description.to_lowercase().contains(&self.search_query) {
                self.filtered_apps.push(app.clone());
                continue;
            }

            // Search in keywords
            for keyword in &app.keywords {
                if keyword.to_lowercase().contains(&self.search_query) {
                    self.filtered_apps.push(app.clone());
                    break;
                }
            }
        }

        // Sort by relevance
        self.filtered_apps.sort_by(|a, b| {
            // Exact name match first
            if a.name.to_lowercase() == self.search_query {
                return std::cmp::Ordering::Less;
            }
            if b.name.to_lowercase() == self.search_query {
                return std::cmp::Ordering::Greater;
            }

            // Then by last used
            b.last_used.cmp(&a.last_used)
        });
    }

    /// Get search result at index
    pub fn get_selected(&self) -> Option<&AppEntry> {
        self.filtered_apps.get(self.selected_index)
    }
}
```

## Grid Layout

```rust
pub struct GridLayout {
    /// Icon size
    icon_size: f32,

    /// Columns in grid
    columns: usize,

    /// Row height
    row_height: f32,

    /// Horizontal spacing
    spacing_x: f32,

    /// Vertical spacing
    spacing_y: f32,

    /// Padding
    padding: f32,
}

impl GridLayout {
    pub fn new() -> Self {
        Self {
            icon_size: 128.0,
            columns: 6,
            row_height: 144.0,  // 128px icon + 16px text
            spacing_x: 16.0,
            spacing_y: 24.0,
            padding: 24.0,
        }
    }

    /// Calculate cell position
    pub fn cell_position(&self, index: usize) -> Point {
        let row = index / self.columns;
        let col = index % self.columns;

        Point {
            x: self.padding + col as f32 * (self.icon_size + self.spacing_x),
            y: self.padding + row as f32 * (self.row_height + self.spacing_y),
        }
    }

    /// Get cell rect
    pub fn cell_rect(&self, index: usize) -> Rect {
        let pos = self.cell_position(index);

        Rect {
            x: pos.x,
            y: pos.y,
            width: self.icon_size,
            height: self.row_height,
        }
    }

    /// Get number of visible rows
    pub fn visible_rows(&self, height: f32) -> usize {
        ((height - 2 * self.padding) / (self.row_height + self.spacing_y)) as usize
    }
}
```

## Rendering

```rust
impl Launcher {
    pub fn render(&mut self, ctx: &mut RenderContext) {
        // Skip if hidden
        if !matches!(self.animation_state, AnimationState::Visible | AnimationState::Showing { .. }) {
            return;
        }

        // Render background with blur
        self.render_background(ctx);

        // Render header (search + category)
        self.render_header(ctx);

        // Render recently used
        self.render_recent(ctx);

        // Render app grid
        self.render_app_grid(ctx);
    }

    fn render_background(&self, ctx: &mut RenderContext) {
        let rect = ctx.screen_rect();

        // Semi-transparent background
        ctx.fill_rect(rect, theme.colors.scrim);

        // Apply blur to background content
        ctx.apply_blur(rect, 40);
    }

    fn render_header(&self, ctx: &mut RenderContext) {
        // Search bar
        let search_rect = Rect {
            x: 48,
            y: 48,
            width: 600,
            height: 48,
        };

        ctx.draw_rounded_rect(
            search_rect,
            24,
            theme.colors.surface,
        );

        // Search icon
        let icon_rect = Rect {
            x: search_rect.x + 16,
            y: search_rect.y + (search_rect.height - 24) / 2,
            width: 24,
            height: 24,
        };
        ctx.draw_icon(icon_rect, &Icon::from_name("search"));

        // Search text
        let text_rect = search_rect.inset_left(56);
        if self.search_query.is_empty() {
            ctx.draw_text(
                text_rect,
                "Search Applications...",
                theme.typography.body,
                theme.colors.on_surface_variant,
                TextAlignment::Left,
            );
        } else {
            ctx.draw_text(
                text_rect,
                &self.search_query,
                theme.typography.body,
                theme.colors.on_surface,
                TextAlignment::Left,
            );
        }

        // Category dropdown
        let category_rect = Rect {
            x: search_rect.x + search_rect.width + 16,
            y: search_rect.y,
            width: 200,
            height: search_rect.height,
        };

        self.render_category_button(ctx, category_rect);
    }

    fn render_category_button(&self, ctx: &mut RenderContext, rect: Rect) {
        ctx.draw_rounded_rect(
            rect,
            24,
            theme.colors.surface_variant,
        );

        let category_name = match &self.category_filter {
            None => "All Categories".into(),
            Some(cat) => format!("{:?}", cat),
        };

        ctx.draw_text(
            rect.inset(16),
            &category_name,
            theme.typography.body,
            theme.colors.on_surface,
            TextAlignment::Left,
        );

        // Dropdown arrow
        let arrow_rect = Rect {
            x: rect.x + rect.width - 32,
            y: rect.y + (rect.height - 16) / 2,
            width: 16,
            height: 16,
        };
        ctx.draw_icon(arrow_rect, &Icon::from_name("arrow-down"));
    }

    fn render_recent(&self, ctx: &mut RenderContext) {
        if self.recent_apps.is_empty() || !self.search_query.is_empty() {
            return;
        }

        let header_rect = Rect {
            x: 48,
            y: 120,
            width: 800,
            height: 32,
        };

        ctx.draw_text(
            header_rect,
            "Recently Used",
            theme.typography.h4,
            theme.colors.on_background,
            TextAlignment::Left,
        );

        let mut x = 48;
        for app in self.recent_apps.iter().take(6) {
            let icon_rect = Rect {
                x,
                y: 160,
                width: 64,
                height: 64,
            };

            ctx.draw_icon(icon_rect, &app.icon);

            x += 64 + 16;
        }
    }

    fn render_app_grid(&self, ctx: &mut RenderContext) {
        let layout = GridLayout::new();
        let start_y = if self.search_query.is_empty() { 280 } else { 120 };

        for (index, app) in self.filtered_apps.iter().enumerate() {
            let cell_rect = layout.cell_rect(index);
            let adjusted_rect = Rect {
                x: cell_rect.x + 48,
                y: cell_rect.y + start_y,
                width: cell_rect.width,
                height: cell_rect.height,
            };

            // Skip if off-screen
            if adjusted_rect.y > ctx.screen_height() {
                break;
            }

            self.render_app_entry(ctx, adjusted_rect, app, index == self.selected_index);
        }
    }

    fn render_app_entry(&self, ctx: &mut RenderContext, rect: Rect, app: &AppEntry, selected: bool) {
        let icon_rect = Rect {
            x: rect.x + (rect.width - 128) / 2,
            y: rect.y,
            width: 128,
            height: 128,
        };

        // Selection background
        if selected {
            ctx.draw_rounded_rect(
                rect.inset(-8),
                16,
                theme.colors.primary.with_alpha(0.2),
            );
        }

        // App icon
        ctx.draw_icon(icon_rect, &app.icon);

        // App name
        let text_rect = Rect {
            x: rect.x,
            y: rect.y + 128 + 4,
            width: rect.width,
            height: 16,
        };

        ctx.draw_text(
            text_rect,
            &app.name,
            theme.typography.caption,
            theme.colors.on_background,
            TextAlignment::Center,
        );
    }
}
```

## Input Handling

```rust
impl Launcher {
    pub fn handle_event(&mut self, event: &Event) -> LauncherResult {
        match event {
            // Search input
            Event::Key { key: KeyCode::Backspace, state: ButtonState::Pressed } => {
                self.search_query.pop();
                self.filter_apps();
                LauncherResult::SearchChanged
            }

            Event::Char { c } => {
                if c.is_ascii() && !c.is_ascii_control() {
                    self.search_query.push(c);
                    self.filter_apps();
                    LauncherResult::SearchChanged
                } else {
                    LauncherResult::NotHandled
                }
            }

            // Navigation
            Event::Key { key: KeyCode::Up, .. } => {
                if self.selected_index >= GridLayout::new().columns {
                    self.selected_index -= GridLayout::new().columns;
                }
                LauncherResult::SelectionChanged
            }

            Event::Key { key: KeyCode::Down, .. } => {
                self.selected_index = (self.selected_index + GridLayout::new().columns)
                    .min(self.filtered_apps.len() - 1);
                LauncherResult::SelectionChanged
            }

            Event::Key { key: KeyCode::Left, .. } => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                LauncherResult::SelectionChanged
            }

            Event::Key { key: KeyCode::Right, .. } => {
                self.selected_index = (self.selected_index + 1)
                    .min(self.filtered_apps.len() - 1);
                LauncherResult::SelectionChanged
            }

            // Launch app
            Event::Key { key: KeyCode::Enter, .. } => {
                if let Some(app) = self.get_selected() {
                    self.launch_app(app);
                    LauncherResult::AppLaunched
                } else {
                    LauncherResult::NotHandled
                }
            }

            // Click
            Event::PointerButton { button: PointerButton::Left, state: ButtonState::Pressed } => {
                if let Some(index) = self.find_app_at(event.pointer_position()) {
                    self.selected_index = index;
                    self.launch_app(&self.filtered_apps[index]);
                    LauncherResult::AppLaunched
                } else {
                    LauncherResult::NotHandled
                }
            }

            // Close launcher
            Event::Key { key: KeyCode::Escape, .. } => {
                self.hide();
                LauncherResult::Closed
            }

            _ => LauncherResult::NotHandled,
        }
    }

    fn find_app_at(&self, position: Point) -> Option<usize> {
        let layout = GridLayout::new();
        let start_y = if self.search_query.is_empty() { 280 } else { 120 };

        for (index, _app) in self.filtered_apps.iter().enumerate() {
            let cell_rect = layout.cell_rect(index);
            let adjusted_rect = Rect {
                x: cell_rect.x + 48,
                y: cell_rect.y + start_y,
                width: cell_rect.width,
                height: cell_rect.height,
            };

            if adjusted_rect.contains(position) {
                return Some(index);
            }
        }

        None
    }

    fn launch_app(&self, app: &AppEntry) {
        // Execute app
        std::process::Command::new(&app.exec)
            .spawn()
            .ok();

        // Update recent apps
        self.update_recent(app);

        // Hide launcher
        self.hide();
    }

    fn update_recent(&self, app: &AppEntry) {
        // Store in recent apps list
        // Persist to disk
        config::add_recent_app(app.id.clone());
    }
}

pub enum LauncherResult {
    SearchChanged,
    SelectionChanged,
    AppLaunched,
    Closed,
    NotHandled,
}
```

## App Discovery

```rust
impl Launcher {
    pub fn discover_apps(&mut self) {
        self.apps.clear();

        // Search standard desktop entry locations
        let paths = vec![
            PathBuf::from("/usr/share/applications"),
            PathBuf::from("/usr/local/share/applications"),
            PathBuf::from("~/.local/share/applications"),
        ];

        for path in paths {
            self.scan_directory(&path);
        }

        // Sort by name
        self.apps.sort_by(|a, b| a.name.cmp(&b.name));
    }

    fn scan_directory(&mut self, dir: &PathBuf) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();

                if path.extension() == Some(OsStr::new("desktop")) {
                    if let Ok(app) = self.parse_desktop_entry(&path) {
                        self.apps.push(app);
                    }
                }
            }
        }
    }

    fn parse_desktop_entry(&self, path: &PathBuf) -> Result<AppEntry> {
        let content = fs::read_to_string(path)?;
        let mut in_desktop_entry = false;

        let mut id = String::new();
        let mut name = String::new();
        let mut icon = String::new();
        let mut exec = String::new();
        let mut categories = Vec::new();
        let mut comment = String::new();
        let mut keywords = Vec::new();

        for line in content.lines() {
            let line = line.trim();

            if line == "[Desktop Entry]" {
                in_desktop_entry = true;
                continue;
            }

            if !in_desktop_entry || line.starts_with('[') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "Name" => name = value.to_string(),
                    "Icon" => icon = value.to_string(),
                    "Exec" => exec = value.to_string(),
                    "Categories" => {
                        categories = value.split(';')
                            .filter(|s| !s.is_empty())
                            .map(|s| s.into())
                            .collect();
                    }
                    "Comment" => comment = value.to_string(),
                    "Keywords" => {
                        keywords = value.split(';')
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_lowercase())
                            .collect();
                    }
                    _ => {}
                }
            }
        }

        let app_id = path.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();

        Ok(AppEntry {
            id: app_id,
            name,
            icon: Icon::from_name(&icon),
            categories,
            description: comment,
            exec,
            keywords,
            last_used: None,
        })
    }
}
```

## Performance Requirements

| Metric | Target | Measurement |
|--------|--------|-------------|
| Search results | <100ms | Query to displayed |
| App discovery | <2s | Scan to ready |
| Animation FPS | 60 | Smooth transitions |
| Memory | <30MB | Total launcher usage |
| Launch | <50ms | Click to hidden |

## Success Criteria

- [ ] All apps discovered
- [ ] Search works instantly
- [ ] Category filtering works
- [ ] Keyboard navigation complete
- [ ] Touch works correctly
- [ ] Recent apps tracked
- [ ] Performance targets met
- [ ] Accessibility support

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Too many apps | Pagination, lazy loading |
| Slow search | Debounce input, indexing |
| Missing icons | Fallback to default icon |

## Sign-Off

**Architect**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅

---

## References

- [Desktop Entry Specification](https://specifications.freedesktop.org/desktop-entry-spec/desktop-entry-spec-latest.html)
- [GNOME App Launcher](https://gitlab.gnome.org/GNOME/gnome-shell/-/blob/main/js/ui/appDisplay.js)
- [KDE Application Launcher](https://github.com/KDE/kickoff/)
- [Cosmic Launcher](https://github.com/pop-os/cosmic-launcher)
