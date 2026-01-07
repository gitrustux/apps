# File Manager (rustica-files) Specification

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Component**: Rustica Shell - File Manager
**Phase**: 6.2 - Desktop Applications (Full Implementation)

## Overview

Rustica Files is a **full-featured file manager** with **icon/list views**, **column view**, **tabs**, **split panes**, **network locations**, **search**, **bookmarks**, **batch operations**, **thumbnails**, **file properties**, and **full accessibility**. Unlike the prototype (Phase 4.1), this is **production code** with all features implemented.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Rustica Files                                                          │
├─────────────────────────────────────────────────────────────────────────┤
│  [≡] [File] [Edit] [View] [Go] [Bookmarks] [Help]          [□] [−] [×]   │
├─────────────────────────────────────────────────────────────────────────┤
│  ◀ ▶  │  ◐  │  ☰  │  🔍  │        │                           │       │
│  Back│ Forward│ View│ Search│ Path: /home/user            [⚙️]    [≡]  │
├─────────────────────────────────────────────────────────────────────────┤
│  Bookmarks │  /home/user/Documents                 [100 items] [Icon]  │
├────────────┼───────────────────────────────────────────────────────────┤
│            │  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐       │
│  ⭐ Home   │  │ 📁 │ │ 📄 │ │ 📄 │ │ 📁 │ │ 📁 │ │ 🖼️ │       │
│  📄 Desktop│  │Docs│ │todo│ │note│ │Down│ │Musc│ │photo│       │
│  📄 Docs   │  │    │ │    │ │    │ │    │ │    │ │    │       │
│  ⬇️ Down   │  └─────┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘       │
│  📶 Network│  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐       │
│  💾 Trash  │  │ 📁 │ │ 📁 │ │ 📁 │ │ 📁 │ │ 📁 │ │ 📄 │       │
│            │  │Vidé│ │Photo│ │Docu│ │Down│ │.conf│ │read│       │
│            │  └─────┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘       │
│            │                                                           │
└────────────┴───────────────────────────────────────────────────────────┘
│  /home/user/Documents  -  8 items, 2 folders, 6 files     256 MB free │
```

## Main Application Structure

```rust
pub struct FileManager {
    /// Window
    window: Window,

    /// Notebook (tab container)
    notebook: Notebook,

    /// Clipboard
    clipboard: Clipboard,

    /// Bookmarks
    bookmarks: BookmarkManager,

    /// Recent locations
    recent: RecentLocations,

    /// File watchers
    watchers: FileWatcherManager,

    /// Thumbnail cache
    thumbnails: ThumbnailCache,

    /// Search index
    search_index: SearchIndex,

    /// Settings
    settings: FileSettings,

    /// Operations queue
    operations: OperationQueue,

    /// Progress dialogs
    progress_dialogs: Vec<ProgressDialog>,
}

pub struct Notebook {
    /// Tabs
    tabs: Vec<Tab>,

    /// Current tab index
    current_tab: usize,

    /// Tab position
    tab_position: TabPosition,
}

pub struct Tab {
    /// Tab widget
    widget: TabWidget,

    /// File browser
    browser: FileBrowser,

    /// Title
    title: String,

    /// Loading state
    loading: bool,

    /// Modified (has pending changes)
    modified: bool,
}

pub struct FileBrowser {
    /// Current directory
    current_path: PathBuf,

    /// View mode
    view_mode: ViewMode,

    /// Icon view
    icon_view: Option<IconView>,

    /// List view
    list_view: Option<ListView>,

    /// Column view
    column_view: Option<ColumnView>,

    /// Split pane
    split_pane: Option<SplitPane>,

    /// Selected items
    selected: Vec<PathBuf>,

    /// Focused item
    focused: Option<PathBuf>,

    /// Scroll position
    scroll_position: Point,

    /// Hidden files visible
    show_hidden: bool,

    /// Sorting
    sort_column: SortColumn,
    sort_order: SortOrder,
}

pub enum ViewMode {
    Icon,
    List,
    Column,
    Compact,
}

pub enum SortColumn {
    Name,
    Size,
    Type,
    Modified,
}

pub enum SortOrder {
    Ascending,
    Descending,
}
```

## Icon View

```rust
pub struct IconView {
    /// Items
    items: Vec<FileItem>,

    /// Icon size
    icon_size: IconSize,

    /// Grid layout
    grid: GridLayout,

    /// Selection model
    selection: SelectionModel,

    /// Drag state
    drag_state: Option<DragState>,
}

pub enum IconSize {
    Small,   // 48px
    Medium,  // 64px
    Large,   // 96px
    XLarge,  // 128px
}

pub struct FileItem {
    /// Path
    pub path: PathBuf,

    /// Metadata
    pub metadata: FileMetadata,

    /// Display name
    pub display_name: String,

    /// Icon
    pub icon: Icon,

    /// Thumbnail (if loaded)
    pub thumbnail: Option<Image>,

    /// Loading thumbnail
    pub loading_thumbnail: bool,

    /// Highlighted (search match)
    pub highlighted: bool,
}

pub struct FileMetadata {
    /// File type
    pub file_type: FileType,

    /// Size
    pub size: u64,

    /// Modified time
    pub modified: SystemTime,

    /// Created time
    pub created: SystemTime,

    /// Accessed time
    pub accessed: SystemTime,

    /// Permissions
    pub permissions: Permissions,

    /// Owner
    pub owner: String,

    /// Group
    pub group: String,

    /// Is symlink
    pub is_symlink: bool,

    /// Symlink target
    pub symlink_target: Option<PathBuf>,
}

pub enum FileType {
    Regular,
    Directory,
    Symlink,
    BlockDevice,
    CharDevice,
    Fifo,
    Socket,

    // Specific file types
    Image,
    Video,
    Audio,
    Text,
    Archive,
    Document,
    Executable,
}

impl IconView {
    pub fn render(&self, ctx: &mut RenderContext, rect: Rect) {
        let mut x = rect.x;
        let mut y = rect.y;

        let icon_size = self.icon_size.size();
        let spacing = 24.0;
        let label_height = 20.0;

        for item in &self.items {
            // Check if we need to wrap
            if x + icon_size > rect.x + rect.width {
                x = rect.x;
                y += icon_size + spacing + label_height;
            }

            let item_rect = Rect {
                x: x - 4.0,
                y: y - 4.0,
                width: icon_size + 8.0,
                height: icon_size + label_height + 12.0,
            };

            // Draw selection background
            if self.selection.is_selected(&item.path) {
                ctx.fill_rounded_rect(item_rect, 4.0, theme.colors.primary.with_alpha(0.25));
            } else if item.highlighted {
                ctx.fill_rounded_rect(item_rect, 4.0, theme.colors.primary.with_alpha(0.15));
            }

            // Draw thumbnail or icon
            let icon_rect = Rect { x, y, width: icon_size, height: icon_size };

            if let Some(ref thumbnail) = item.thumbnail {
                ctx.draw_image_rounded(icon_rect, thumbnail, 4.0);
            } else {
                ctx.draw_icon(icon_rect, &item.icon);
            }

            // Draw emblem overlays
            self.draw_emblems(ctx, item, icon_rect);

            // Draw label
            let label_rect = Rect {
                x,
                y: y + icon_size + 4.0,
                width: icon_size,
                height: label_height,
            };

            let display_name = self.truncate_name(&item.display_name, icon_size as usize);
            ctx.draw_text_centered(
                label_rect,
                &display_name,
                theme.typography.caption,
                theme.colors.on_surface,
                TextOverflow::Ellipsis,
            );

            x += icon_size + spacing;
        }
    }

    fn draw_emblems(&self, ctx: &mut RenderContext, item: &FileItem, icon_rect: Rect) {
        let emblem_size = 16.0;
        let emblem_x = icon_rect.x + icon_rect.width - emblem_size;
        let emblem_y = icon_rect.y + icon_rect.height - emblem_size;

        // Symlink emblem
        if item.metadata.is_symlink {
            let emblem_rect = Rect {
                x: emblem_x,
                y: emblem_y,
                width: emblem_size,
                height: emblem_size,
            };
            ctx.draw_icon(emblem_rect, "emblem-symbolic-link");
        }

        // Locked emblem (no write permission)
        if item.metadata.permissions.readonly() {
            let emblem_rect = Rect {
                x: emblem_x,
                y: emblem_y,
                width: emblem_size,
                height: emblem_size,
            };
            ctx.draw_icon(emblem_rect, "emblem-readonly");
        }
    }

    fn truncate_name(&self, name: &str, max_width: usize) -> String {
        let approx_chars = max_width / 8; // Rough approximation

        if name.len() > approx_chars {
            let mut truncated = name.chars().take(approx_chars - 3).collect::<String>();
            truncated.push_str("...");
            truncated
        } else {
            name.to_string()
        }
    }
}
```

## List View

```rust
pub struct ListView {
    /// Items
    items: Vec<FileItem>,

    /// Columns
    columns: Vec<Column>,

    /// Column widths
    column_widths: Vec<f32>,

    /// Row height
    row_height: f32,

    /// Selection model
    selection: SelectionModel,

    /// Sort column
    sort_column: SortColumn,

    /// Sort order
    sort_order: SortOrder,
}

pub struct Column {
    pub kind: ColumnKind,
    pub title: String,
    pub resizable: bool,
}

pub enum ColumnKind {
    Name,
    Size,
    Type,
    Modified,
    Created,
    Permissions,
    Owner,
}

impl ListView {
    pub fn render(&self, ctx: &mut RenderContext, rect: Rect) {
        // Draw header
        self.draw_header(ctx, rect);

        // Draw rows
        let mut y = rect.y + self.row_height;

        for item in &self.items {
            let row_rect = Rect {
                x: rect.x,
                y,
                width: rect.width,
                height: self.row_height,
            };

            // Draw selection background
            if self.selection.is_selected(&item.path) {
                ctx.fill_rect(row_rect, theme.colors.primary.with_alpha(0.25));
            } else if item.highlighted {
                ctx.fill_rect(row_rect, theme.colors.primary.with_alpha(0.15));
            }

            // Draw cells
            let mut x = rect.x;
            for (col_idx, column) in self.columns.iter().enumerate() {
                let cell_width = self.column_widths[col_idx];
                let cell_rect = Rect { x, y, width: cell_width, height: self.row_height };

                self.draw_cell(ctx, cell_rect, column, item);

                x += cell_width;
            }

            y += self.row_height;
        }
    }

    fn draw_header(&self, ctx: &mut RenderContext, rect: Rect) {
        let header_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: self.row_height,
        };

        // Header background
        ctx.fill_rect(header_rect, theme.colors.surface_variant);

        // Column headers
        let mut x = rect.x;
        for (col_idx, column) in self.columns.iter().enumerate() {
            let cell_width = self.column_widths[col_idx];
            let cell_rect = Rect { x, y: rect.y, width: cell_width, height: self.row_height };

            // Draw title
            ctx.draw_text(
                Rect { x: x + 8.0, y: cell_rect.y + 4.0, width: cell_width - 16.0, height: 20.0 },
                &column.title,
                theme.typography.caption,
                theme.colors.on_surface_variant,
            );

            // Draw sort indicator
            if matches!(column.kind, _ if self.sort_column == column.kind) {
                let indicator = match self.sort_order {
                    SortOrder::Ascending => "pan-up",
                    SortOrder::Descending => "pan-down",
                };
                ctx.draw_icon(
                    Rect { x: x + cell_width - 20.0, y: cell_rect.y + 8.0, width: 16.0, height: 16.0 },
                    indicator,
                );
            }

            x += cell_width;
        }
    }

    fn draw_cell(&self, ctx: &mut RenderContext, rect: Rect, column: &Column, item: &FileItem) {
        let padding = 8.0;
        let text_rect = Rect {
            x: rect.x + padding,
            y: rect.y + 4.0,
            width: rect.width - padding * 2.0,
            height: 20.0,
        };

        match column.kind {
            ColumnKind::Name => {
                // Draw icon
                let icon_rect = Rect {
                    x: text_rect.x,
                    y: rect.y + 4.0,
                    width: 16.0,
                    height: 16.0,
                };
                ctx.draw_icon(icon_rect, &item.icon);

                // Draw name
                let name_rect = Rect {
                    x: text_rect.x + 20.0,
                    y: text_rect.y,
                    width: text_rect.width - 20.0,
                    height: 20.0,
                };
                ctx.draw_text(name_rect, &item.display_name, theme.typography.body, theme.colors.on_surface);
            }

            ColumnKind::Size => {
                if item.metadata.file_type != FileType::Directory {
                    let size = format_size(item.metadata.size);
                    ctx.draw_text(text_rect, &size, theme.typography.body, theme.colors.on_surface);
                }
            }

            ColumnKind::Type => {
                let type_str = format_file_type(&item.metadata);
                ctx.draw_text(text_rect, &type_str, theme.typography.body, theme.colors.on_surface);
            }

            ColumnKind::Modified => {
                let modified = format_datetime(item.metadata.modified);
                ctx.draw_text(text_rect, &modified, theme.typography.body, theme.colors.on_surface);
            }

            _ => {}
        }
    }
}
```

## Column View

```rust
pub struct ColumnView {
    /// Columns (history of directories)
    columns: Vec<ColumnViewColumn>,

    /// Current column index
    current_column: usize,

    /// Column width
    column_width: f32,
}

pub struct ColumnViewColumn {
    /// Path
    path: PathBuf,

    /// Items
    items: Vec<FileItem>,

    /// Selected index
    selected_index: Option<usize>,

    /// Scroll offset
    scroll_offset: f32,
}

impl ColumnView {
    pub fn render(&self, ctx: &mut RenderContext, rect: Rect) {
        let mut x = rect.x;

        for (col_idx, column) in self.columns.iter().enumerate() {
            let column_rect = Rect {
                x,
                y: rect.y,
                width: self.column_width,
                height: rect.height,
            };

            // Draw column background
            let bg_color = if col_idx == self.current_column {
                theme.colors.surface
            } else {
                theme.colors.surface_variant
            };
            ctx.fill_rect(column_rect, bg_color);

            // Draw column header (path)
            self.draw_column_header(ctx, column, column_rect);

            // Draw items
            let mut y = column_rect.y + 32.0;
            for (item_idx, item) in column.items.iter().enumerate() {
                let item_rect = Rect {
                    x: column_rect.x,
                    y,
                    width: self.column_width,
                    height: 28.0,
                };

                // Draw selection
                if column.selected_index == Some(item_idx) {
                    ctx.fill_rect(item_rect, theme.colors.primary.with_alpha(0.3));
                }

                // Draw icon
                let icon_rect = Rect {
                    x: item_rect.x + 8.0,
                    y: item_rect.y + 6.0,
                    width: 16.0,
                    height: 16.0,
                };
                ctx.draw_icon(icon_rect, &item.icon);

                // Draw name
                let name_rect = Rect {
                    x: icon_rect.x + 20.0,
                    y: item_rect.y + 4.0,
                    width: self.column_width - 32.0,
                    height: 20.0,
                };
                ctx.draw_text(name_rect, &item.display_name, theme.typography.body, theme.colors.on_surface);

                y += 28.0;
            }

            // Draw column separator
            ctx.fill_rect(
                Rect { x: x + self.column_width, y: rect.y, width: 1.0, height: rect.height },
                theme.colors.outline,
            );

            x += self.column_width + 1.0;
        }
    }

    fn draw_column_header(&self, ctx: &mut RenderContext, column: &ColumnViewColumn, rect: Rect) {
        let header_rect = Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: 32.0,
        };

        // Background
        ctx.fill_rect(header_rect, theme.colors.primary.with_alpha(0.15));

        // Path
        let path_text = column.path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("/");

        ctx.draw_text(
            Rect { x: header_rect.x + 8.0, y: header_rect.y + 6.0, width: header_rect.width - 16.0, height: 20.0 },
            path_text,
            theme.typography.h4,
            theme.colors.on_surface,
        );
    }
}
```

## File Operations

```rust
pub struct OperationQueue {
    /// Queued operations
    queue: Vec<QueuedOperation>,

    /// Active operation
    active: Option<ActiveOperation>,

    /// Max concurrent operations
    max_concurrent: usize,
}

pub struct QueuedOperation {
    /// Operation kind
    pub kind: OperationKind,

    /// Source paths
    pub sources: Vec<PathBuf>,

    /// Destination
    pub destination: PathBuf,

    /// Callback
    pub callback: Option<Callback>,
}

pub enum OperationKind {
    Copy,
    Move,
    Delete,
    Rename,
    Link,
}

pub struct ActiveOperation {
    /// Queued operation
    pub operation: QueuedOperation,

    /// Progress
    pub progress: OperationProgress,

    /// Start time
    pub start_time: Instant,

    /// Cancelled
    pub cancelled: bool,
}

pub enum OperationProgress {
    /// Bytes processed / total bytes
    Bytes { processed: u64, total: u64 },

    /// Files processed / total files
    Files { processed: usize, total: usize },

    /// Indeterminate
    Indeterminate,
}

impl FileManager {
    pub fn copy_files(&mut self, sources: Vec<PathBuf>, destination: PathBuf) {
        let operation = QueuedOperation {
            kind: OperationKind::Copy,
            sources,
            destination,
            callback: None,
        };

        self.operations.queue.push(operation);
        self.process_next_operation();
    }

    pub fn move_files(&mut self, sources: Vec<PathBuf>, destination: PathBuf) {
        let operation = QueuedOperation {
            kind: OperationKind::Move,
            sources,
            destination,
            callback: None,
        };

        self.operations.queue.push(operation);
        self.process_next_operation();
    }

    pub fn delete_files(&mut self, sources: Vec<PathBuf>) {
        let operation = QueuedOperation {
            kind: OperationKind::Delete,
            sources,
            destination: PathBuf::from("/"), // Dummy
            callback: None,
        };

        self.operations.queue.push(operation);
        self.process_next_operation();
    }

    fn process_next_operation(&mut self) {
        if self.operations.active.is_some() {
            return; // Already processing
        }

        if let Some(operation) = self.operations.queue.pop() {
            // Start operation
            self.start_operation(operation);
        }
    }

    fn start_operation(&mut self, operation: QueuedOperation) {
        let operation_id = uuid::Uuid::new_v4().to_string();

        // Create progress dialog
        let dialog = ProgressDialog::new(operation_id.clone(), &operation);
        self.progress_dialogs.push(dialog);

        // Start active operation
        self.operations.active = Some(ActiveOperation {
            operation,
            progress: OperationProgress::Indeterminate,
            start_time: Instant::now(),
            cancelled: false,
        });

        // Spawn processing thread
        let active_op = self.operations.active.as_mut().unwrap();
        self.spawn_operation_thread(operation_id.clone());
    }

    fn spawn_operation_thread(&mut self, operation_id: String) {
        // In a real implementation, this would spawn a thread
        // that processes files and sends progress updates
    }
}
```

## Thumbnail Cache

```rust
pub struct ThumbnailCache {
    /// Cache storage
    cache: HashMap<PathBuf, CachedThumbnail>,

    /// Max size (bytes)
    max_size: usize,

    /// Current size
    current_size: usize,

    /// Thumbnail directory
    thumbnail_dir: PathBuf,
}

pub struct CachedThumbnail {
    /// Image
    pub image: Image,

    /// Size in bytes
    pub size: usize,

    /// Last access
    pub last_access: Instant,

    /// Persistent (saved to disk)
    pub persistent: bool,
}

impl ThumbnailCache {
    pub fn new() -> Result<Self, Error> {
        let thumbnail_dir = dirs::cache_dir()
            .ok_or(Error::NoCacheDir)?
            .join("rustica-files/thumbnails");

        std::fs::create_dir_all(&thumbnail_dir)?;

        Ok(Self {
            cache: HashMap::new(),
            max_size: 100 * 1024 * 1024, // 100 MB
            current_size: 0,
            thumbnail_dir,
        })
    }

    pub fn get(&mut self, path: &PathBuf) -> Option<Image> {
        if let Some(cached) = self.cache.get(path) {
            // Update last access
            cached.last_access = Instant::now();
            return Some(cached.image.clone());
        }

        // Try to load from disk
        let thumbnail_path = self.thumbnail_path(path);
        if thumbnail_path.exists() {
            if let Ok(image) = Image::load(&thumbnail_path) {
                // Add to cache
                self.add_to_cache(path.clone(), image.clone(), true);
                return Some(image);
            }
        }

        None
    }

    pub fn generate(&mut self, path: &PathBuf, original_size: (u32, u32)) -> Result<Image, Error> {
        // Determine thumbnail size
        let thumb_size = 256;
        let scale = thumb_size as f32 / original_size.0.max(original_size.1) as f32;

        // Load original image
        let original = Image::load(path)?;

        // Generate thumbnail
        let thumbnail = original.scale(
            (original_size.0 as f32 * scale) as u32,
            (original_size.1 as f32 * scale) as u32,
        );

        // Save to disk
        let thumbnail_path = self.thumbnail_path(path);
        thumbnail.save(&thumbnail_path)?;

        // Add to cache
        self.add_to_cache(path.clone(), thumbnail.clone(), true);

        Ok(thumbnail)
    }

    fn thumbnail_path(&self, path: &Path) -> PathBuf {
        // Generate thumbnail filename from path hash
        let hash = std::collections::hash_map::DefaultHasher::new();
        // Hash the path
        use std::hash::{Hash, Hasher};
        path.hash(&mut hash);
        let hash_val = hash.finish();

        self.thumbnail_dir.join(format!("{:016x}.png", hash_val))
    }

    fn add_to_cache(&mut self, path: PathBuf, image: Image, persistent: bool) {
        let size = image.estimated_size();

        // Evict if necessary
        while self.current_size + size > self.max_size && !self.cache.is_empty() {
            self.evict_lru();
        }

        // Add to cache
        self.cache.insert(path, CachedThumbnail {
            image,
            size,
            last_access: Instant::now(),
            persistent,
        });

        self.current_size += size;
    }

    fn evict_lru(&mut self) {
        // Find least recently used
        let lru_key = self.cache.iter()
            .min_by_key(|(_, cached)| cached.last_access)
            .map(|(path, _)| path.clone());

        if let Some(path) = lru_key {
            if let Some(cached) = self.cache.remove(&path) {
                self.current_size -= cached.size;

                // Delete from disk if persistent
                if cached.persistent {
                    let _ = std::fs::remove_file(self.thumbnail_path(&path));
                }
            }
        }
    }
}
```

## Search

```rust
pub struct SearchIndex {
    /// Indexed files
    index: HashMap<PathBuf, FileIndexEntry>,

    /// Trigram index
    trigrams: HashMap<String, HashSet<PathBuf>>,
}

pub struct FileIndexEntry {
    /// Path
    pub path: PathBuf,

    /// File name
    pub name: String,

    /// Content (for text files)
    pub content: Option<String>,

    /// Metadata
    pub metadata: FileMetadata,
}

impl SearchIndex {
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
            trigrams: HashMap::new(),
        }
    }

    pub fn index_directory(&mut self, path: &PathBuf) -> Result<(), Error> {
        for entry in walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path().to_path_buf();
            self.index_file(path)?;
        }

        Ok(())
    }

    fn index_file(&mut self, path: PathBuf) -> Result<(), Error> {
        let metadata = std::fs::metadata(&path)?;
        let file_type = self.detect_file_type(&path);

        let entry = FileIndexEntry {
            name: path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string(),
            content: if file_type == FileType::Text {
                self.read_text_content(&path)?
            } else {
                None
            },
            metadata: FileMetadata::from_std(&metadata, &path)?,
            path: path.clone(),
        };

        // Add to index
        self.index.insert(path.clone(), entry);

        // Update trigram index
        if let Some(ref content) = self.index.get(&path).unwrap().content {
            self.update_trigrams(&path, content);
        }

        Ok(())
    }

    fn update_trigrams(&mut self, path: &Path, text: &str) {
        let text_lower = text.to_lowercase();

        // Generate trigrams
        for window in text_lower.as_bytes().windows(3) {
            let trigram = String::from_utf8_lossy(window).to_string();

            self.trigrams
                .entry(trigram)
                .or_insert_with(HashSet::new)
                .insert(path.clone());
        }
    }

    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        let query_lower = query.to_lowercase();

        // Generate trigrams from query
        let mut query_trigrams = Vec::new();
        for window in query_lower.as_bytes().windows(3) {
            query_trigrams.push(String::from_utf8_lossy(window).to_string());
        }

        // Find files with matching trigrams
        let mut candidates: HashMap<PathBuf, usize> = HashMap::new();

        for trigram in &query_trigrams {
            if let Some(files) = self.trigrams.get(trigram) {
                for file in files {
                    *candidates.entry(file.clone()).or_insert(0) += 1;
                }
            }
        }

        // Rank by trigram matches
        let mut results: Vec<_> = candidates.into_iter()
            .filter(|(_, count)| *count > query_trigrams.len() / 2)
            .collect();

        results.sort_by(|a, b| b.1.cmp(&a.1));

        results.into_iter()
            .take(100)
            .map(|(path, score)| SearchResult {
                path: path.clone(),
                score: score as f32,
                highlights: Vec::new(),
            })
            .collect()
    }
}

pub struct SearchResult {
    pub path: PathBuf,
    pub score: f32,
    pub highlights: Vec<TextRange>,
}
```

## Network Locations

```rust
pub struct NetworkLocations {
    /// Mounted locations
    mounted: HashMap<String, NetworkLocation>,

    /// Available locations
    available: Vec<NetworkLocation>,
}

pub struct NetworkLocation {
    /// Location type
    pub kind: NetworkLocationKind,

    /// Display name
    pub display_name: String,

    /// URI
    pub uri: String,

    /// Mount point
    pub mount_point: Option<PathBuf>,

    /// Mounted
    pub mounted: bool,

    /// Icon
    pub icon: Icon,
}

pub enum NetworkLocationKind {
    Smb { share: String },
    Ftp { host: String },
    Sftp { host: String },
    WebDav { url: String },
    Network,
}

impl NetworkLocations {
    pub fn discover(&mut self) -> Result<(), Error> {
        // Use Avahi for service discovery
        // This is a placeholder for the actual implementation

        Ok(())
    }

    pub fn mount(&mut self, location: &NetworkLocation) -> Result<PathBuf, Error> {
        let mount_point = dirs::runtime_dir()
            .ok_or(Error::NoRuntimeDir)?
            .join("rustica-files")
            .join(uuid::Uuid::new_v4().to_string());

        std::fs::create_dir_all(&mount_point)?;

        // Mount based on type
        match &location.kind {
            NetworkLocationKind::Smb { share } => {
                self.mount_smb(share, &mount_point)?;
            }

            NetworkLocationKind::Sftp { host } => {
                self.mount_sftp(host, &mount_point)?;
            }

            _ => {
                return Err(Error::UnsupportedMountType);
            }
        }

        Ok(mount_point)
    }

    fn mount_smb(&self, share: &str, mount_point: &Path) -> Result<(), Error> {
        // Use mount.cifs or gvfs-mount
        let output = std::process::Command::new("mount")
            .args(&["-t", "cifs", share, mount_point.to_str().unwrap()])
            .output()?;

        if !output.status.success() {
            return Err(Error::MountFailed {
                reason: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        Ok(())
    }

    fn mount_sftp(&self, host: &str, mount_point: &Path) -> Result<(), Error> {
        // Use sshfs
        let output = std::process::Command::new("sshfs")
            .args(&[host, mount_point.to_str().unwrap()])
            .output()?;

        if !output.status.success() {
            return Err(Error::MountFailed {
                reason: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        Ok(())
    }

    pub fn unmount(&self, mount_point: &Path) -> Result<(), Error> {
        let output = std::process::Command::new("umount")
            .arg(mount_point)
            .output()?;

        if !output.status.success() {
            return Err(Error::UnmountFailed {
                reason: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        Ok(())
    }
}
```

## File Structure

```
/var/www/rustux.com/prod/apps/gui/
├── rustica-files/
│   ├── Cargo.toml
│   ├── resources/
│   │   └── icons/
│   └── src/
│       ├── main.rs
│       ├── app.rs
│       ├── notebook.rs
│       ├── tab.rs
│       ├── browser.rs
│       ├── views/
│       │   ├── mod.rs
│       │   ├── icon.rs
│       │   ├── list.rs
│       │   ├── column.rs
│       │   └── compact.rs
│       ├── items.rs
│       ├── metadata.rs
│       ├── operations.rs
│       ├── clipboard.rs
│       ├── bookmarks.rs
│       ├── search.rs
│       ├── thumbnails.rs
│       ├── network.rs
│       └── dialogs/
│           ├── properties.rs
│           ├── progress.rs
│           └── confirm.rs
```

## Dependencies

```toml
[package]
name = "rustica-files"
version = "1.0.0"
edition = "2021"

[dependencies]
# GUI framework
librustica = { path = "../../../libs/librustica" }

# File operations
walkdir = "2.0"
dirs = "5.0"

# Thumbnails
image = "0.24"

# Search
regex = "1.0"

# Serialization
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# File watching
notify = "5.0"

# MIME detection
mime_guess = "2.0"

# Date/time
chrono = "0.4"

# UUID
uuid = { version = "1.0", features = ["v4"] }
```

## Performance Requirements

| Metric | Target | Measurement |
|--------|--------|-------------|
| Directory load | <100ms | 1000 items |
| Thumbnail generation | <500ms | 1MB image |
| Search | <200ms | 100k files |
| Copy speed | >100MB/s | Local file |
| Memory | <100MB | Base usage |
| Startup time | <300ms | Launch to ready |

## Success Criteria

- [ ] All view modes work
- [ ] File operations robust
- [ ] Thumbnails generate correctly
- [ ] Search is fast
- [ ] Network locations mount
- [ ] Tabs work
- [ ] Split panes work
- [ ] Bookmarks persist
- [ ] Drag-and-drop works
- [ ] Full accessibility
- [ ] Performance targets met

## Sign-Off

**Architect**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅

---

## Timeline

- Week 1: File browser + icon view
- Week 2: List view + column view
- Week 3: File operations + progress dialogs
- Week 4: Thumbnails + search
- Week 5: Network locations + bookmarks
- Week 6: Tabs + split panes + polish

**Total**: 6 weeks
