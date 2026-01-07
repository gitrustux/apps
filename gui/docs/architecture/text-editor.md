# Text Editor (rustica-edit) Specification

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Component**: Rustica Shell - Text Editor
**Phase**: 6.3 - Desktop Applications

## Overview

Rustica Edit is a **modern, programmer-focused text editor** with **syntax highlighting**, **code completion**, **multiple cursors**, **split views**, **tabs**, **Git integration**, **LSP support**, **project management**, and **extensive keyboard shortcuts**. It's designed for both **quick editing** and **full development workflows**.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Rustica Edit - main.rs                                                │
├─────────────────────────────────────────────────────────────────────────┤
│  [≡] [File] [Edit] [View] [Go] [Tools] [Help]              [□] [−] [×]  │
├─────────────────────────────────────────────────────────────────────────┤
│  [main.rs] [Cargo.toml] [README.md] [src/] [+ tab]            [⚙️]     │
├─────────────────────────────────────────────────────────────────────────┤
│  1   │  fn main() {                                                    │
│  2   │      println!("Hello, Rustica!");                               │
│  3   │  }                                                              │
│  4   │                                                                  │
│  5   │  impl MyApp {                                                   │
│  6   │      fn new() -> Self {                                         │
│  7   │          Self {                                                 │
│  8   │              window: Window::new(),                             │
│  9   │          }                                                      │
│ 10   │      }                                                          │
│  11   │  }                                                             │
│      │                                                                  │
│      │                                                                  │
│  [rustica-edit] [src] [main.rs]                   L:10 C:25 UTF-8 Rust  │
│  ▶▶  [git: main] [+0] [⚠0] [*1]                          [💬 2] [⚙️]  │
└─────────────────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────────────────┐
│  Symbol Navigator                                       [×]            │
├─────────────────────────────────────────────────────────────────────────┤
│  Structs                                                               │
│    • MyApp                                                             │
│  Functions                                                            │
│    • main()                                                            │
│  Traits                                                               │
│    (none)                                                             │
│  Impls                                                                │
│    • MyApp                                                            │
└─────────────────────────────────────────────────────────────────────────┘
```

## Main Application Structure

```rust
pub struct TextEditor {
    /// Window
    window: Window,

    /// Notebook (tab container)
    notebook: Notebook,

    /// Command palette
    command_palette: CommandPalette,

    /// Sidebar
    sidebar: Sidebar,

    /// Bottom panel
    bottom_panel: BottomPanel,

    /// Status bar
    status_bar: StatusBar,

    /// Settings
    settings: EditorSettings,

    /// Theme registry
    themes: ThemeRegistry,

    /// Language server client
    lsp_client: Option<LspClient>,

    /// Git manager
    git: GitManager,

    /// File watcher
    watcher: FileWatcher,

    /// Recent files
    recent_files: RecentFiles,
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

    /// Document
    document: Document,

    /// Text view
    view: TextView,

    /// Modified
    modified: bool,

    /// Save state
    save_state: SaveState,

    /// Cursor positions (for multi-cursor)
    cursors: Vec<Cursor>,
}

pub struct Document {
    /// Content
    content: Rope,

    /// Path
    path: Option<PathBuf>,

    /// Language
    language: Language,

    /// Encoding
    encoding: Encoding,

    /// Line endings
    line_ending: LineEnding,

    /// Undo history
    undo_stack: UndoStack,

    /// Redo history
    redo_stack: RedoStack,

    /// Bookmarks
    bookmarks: HashSet<LineNumber>,

    /// Breakpoints
    breakpoints: HashSet<LineNumber>,
}

pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    HTML,
    CSS,
    Markdown,
    TOML,
    JSON,
    Shell,
    C,
    Cpp,
    Go,
    Java,
    Kotlin,
    Swift,
    PlainText,
}

pub enum LineEnding {
    LF,   // \n (Unix/Linux)
    CRLF, // \r\n (Windows)
    CR,   // \r (Classic Mac)
}
```

## Text View

```rust
pub struct TextView {
    /// Text buffer
    buffer: Rope,

    /// Viewport
    viewport: Viewport,

    /// Cursor
    cursor: Cursor,

    /// Selection
    selection: Selection,

    /// Highlighting
    highlighter: Highlighter,

    /// Gutter
    gutter: Gutter,

    /// Minimap
    minimap: Option<Minimap>,

    /// Wrap mode
    wrap_mode: WrapMode,

    /// Scroll position
    scroll_offset: Point,

    /// Font
    font: Font,

    /// Line height
    line_height: f32,

    /// Char width
    char_width: f32,
}

pub struct Viewport {
    /// Width
    pub width: f32,

    /// Height
    pub height: f32,

    /// First visible line
    pub first_line: LineNumber,

    /// First visible column
    pub first_column: ColumnNumber,
}

pub type LineNumber = usize;
pub type ColumnNumber = usize;

pub struct Cursor {
    /// Position
    pub position: Position,

    /// Anchor (for selection)
    pub anchor: Position,

    /// Preferred column (for vertical movement)
    pub preferred_column: Option<ColumnNumber>,

    /// Insert mode
    pub insert_mode: bool,

    /// Auto-bracket pair
    pub auto_pair: bool,
}

pub struct Position {
    pub line: LineNumber,
    pub column: ColumnNumber,
}

pub enum Selection {
    None,
    Normal { start: Position, end: Position },
    Rectangular { start: Position, end: Position },
}

pub enum WrapMode {
    None,
    Word,
    Char,
}

impl TextView {
    pub fn render(&self, ctx: &mut RenderContext, rect: Rect) {
        // Draw background
        ctx.fill_rect(rect, theme.colors.editor_background);

        // Draw gutter
        self.draw_gutter(ctx, rect);

        // Draw text
        self.draw_text(ctx, rect);

        // Draw cursor
        self.draw_cursor(ctx, rect);

        // Draw selection
        self.draw_selection(ctx, rect);

        // Draw line highlight
        self.draw_line_highlight(ctx, rect);

        // Draw minimap
        if let Some(ref minimap) = self.minimap {
            minimap.render(ctx, Rect {
                x: rect.x + rect.width - 150.0,
                y: rect.y,
                width: 150.0,
                height: rect.height,
            });
        }
    }

    fn draw_text(&self, ctx: &mut RenderContext, rect: Rect) {
        let mut y = rect.y - (self.scroll_offset.y % self.line_height) as f32;

        let start_line = (self.scroll_offset.y / self.line_height as f32) as usize;
        let end_line = start_line + (rect.height / self.line_height) as usize + 1;

        for line_idx in start_line..end_line.min(self.buffer.len_lines()) {
            let line = self.buffer.line(line_idx);
            let line_text = line.to_string();

            // Highlight line
            let highlights = self.highlighter.highlight_line(&line_text, line_idx);

            let mut x = rect.x + 80.0; // Gutter width + padding

            for (segment, style) in highlights {
                let segment_rect = Rect {
                    x,
                    y,
                    width: segment.len() as f32 * self.char_width,
                    height: self.line_height,
                };

                ctx.draw_text(segment_rect, segment, self.font, style.color);

                x += segment_rect.width;
            }

            y += self.line_height;
        }
    }

    fn draw_cursor(&self, ctx: &mut RenderContext, rect: Rect) {
        for cursor in &self.cursors {
            // Calculate cursor position
            let cursor_y = rect.y + (cursor.position.line as f32 * self.line_height) - self.scroll_offset.y;
            let cursor_x = rect.x + 80.0 + (cursor.position.column as f32 * self.char_width) - self.scroll_offset.x;

            // Draw cursor
            let cursor_rect = Rect {
                x: cursor_x,
                y: cursor_y,
                width: if cursor.insert_mode { 2.0 } else { self.char_width },
                height: self.line_height,
            };

            ctx.fill_rect(cursor_rect, theme.colors.cursor);

            // Draw cursor line (if enabled)
            if self.settings.cursor_line {
                let line_rect = Rect {
                    x: rect.x + 80.0,
                    y: cursor_y,
                    width: rect.width - 80.0,
                    height: self.line_height,
                };
                ctx.fill_rect(line_rect, theme.colors.current_line.with_alpha(0.5));
            }
        }
    }

    fn draw_selection(&self, ctx: &mut RenderContext, rect: Rect) {
        for cursor in &self.cursors {
            if let Selection::Normal { start, end } = &cursor.selection {
                let (start, end) = if *start < *end {
                    (start, end)
                } else {
                    (end, start)
                };

                // Draw selection for each line
                for line_idx in start.line..=end.line {
                    let line_y = rect.y + (line_idx as f32 * self.line_height) - self.scroll_offset.y;

                    let start_col = if line_idx == start.line { start.column } else { 0 };
                    let end_col = if line_idx == end.line { end.column } else { self.buffer.line(line_idx).len_chars() };

                    let sel_rect = Rect {
                        x: rect.x + 80.0 + (start_col as f32 * self.char_width),
                        y: line_y,
                        width: (end_col - start_col) as f32 * self.char_width,
                        height: self.line_height,
                    };

                    ctx.fill_rect(sel_rect, theme.colors.selection);
                }
            }
        }
    }
}
```

## Syntax Highlighting

```rust
pub struct Highlighter {
    /// Language syntax
    syntax: Syntax,

    /// Theme
    theme: HighlightTheme,

    /// Cache
    cache: HighlightCache,
}

pub struct Syntax {
    /// Language definition
    pub language: Language,

    /// Token definitions
    pub token_defs: Vec<TokenDef>,

    /// Contexts
    pub contexts: Vec<SyntaxContext>,
}

pub struct TokenDef {
    /// Name
    pub name: String,

    /// Match regex
    pub regex: Regex,

    /// Capture groups
    pub captures: Vec<Capture>,
}

pub struct Capture {
    /// Group index
    pub index: usize,

    /// Token type
    pub token: TokenType,
}

pub enum TokenType {
    Keyword,
    String,
    Number,
    Comment,
    Function,
    Variable,
    Type,
    Operator,
    Punctuation,
    Constant,
    Tag,
    Attribute,
    Error,
}

pub struct SyntaxContext {
    /// Name
    pub name: String,

    /// Rules
    pub rules: Vec<SyntaxRule>,

    /// Meta scope
    pub meta_scope: Option<TokenType>,
}

pub struct SyntaxRule {
    /// Match regex
    pub match_regex: Regex,

    /// Scope
    pub scope: TokenType,

    /// Push context
    pub push: Option<String>,

    /// Pop context
    pub pop: bool,
}

impl Highlighter {
    pub fn highlight_line(&self, line: &str, line_idx: usize) -> Vec<(&str, HighlightStyle)> {
        // Check cache
        let cache_key = (line.to_string(), line_idx);
        if let Some(cached) = self.cache.get(&cache_key) {
            return cached.clone();
        }

        // Tokenize line
        let tokens = self.tokenize(line);

        // Apply styles
        let mut highlights = Vec::new();
        for token in tokens {
            let style = self.theme.get_style(token.token_type);
            highlights.push((token.text, style));
        }

        // Cache result
        self.cache.insert(cache_key, highlights.clone());

        highlights
    }

    fn tokenize(&self, line: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut pos = 0;

        while pos < line.len() {
            // Find next token
            let mut longest_match = None;
            let mut longest_len = 0;

            for token_def in &self.syntax.token_defs {
                if let Some(matched) = token_def.regex.find_at(line, pos) {
                    let match_len = matched.end() - matched.start();
                    if match_len > longest_len {
                        longest_match = Some((token_def, matched));
                        longest_len = match_len;
                    }
                }
            }

            if let Some((token_def, matched)) = longest_match {
                tokens.push(Token {
                    text: &line[matched.start()..matched.end()],
                    token_type: self.map_token_type(&token_def.name),
                });
                pos = matched.end();
            } else {
                // No match, emit as text
                tokens.push(Token {
                    text: &line[pos..pos + 1],
                    token_type: TokenType::Punctuation,
                });
                pos += 1;
            }
        }

        tokens
    }
}

pub struct Token<'a> {
    pub text: &'a str,
    pub token_type: TokenType,
}

pub struct HighlightStyle {
    pub color: Color,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}
```

## Code Completion

```rust
pub struct Completer {
    /// LSP client
    lsp: Option<LspClient>,

    /// Local completions
    local_completions: LocalCompleter,
}

pub struct LocalCompleter {
    /// Keywords
    keywords: Vec<String>,

    /// Snippets
    snippets: Vec<Snippet>,
}

pub struct Snippet {
    /// Trigger
    pub trigger: String,

    /// Description
    pub description: String,

    /// Content
    pub content: String,

    /// Tab stops
    pub tab_stops: Vec<TabStop>,
}

pub struct TabStop {
    /// Index
    pub index: usize,

    /// Placeholder
    pub placeholder: Option<String>,

    /// Default value
    pub default: String,
}

pub struct CompletionItem {
    /// Label
    pub label: String,

    /// Kind
    pub kind: CompletionItemKind,

    /// Detail
    pub detail: Option<String>,

    /// Documentation
    pub documentation: Option<String>,

    /// Text edit
    pub text_edit: Option<TextEdit>,

    /// Sort text
    pub sort_text: Option<String>,

    /// Filter text
    pub filter_text: Option<String>,

    /// Insert text format
    pub insert_text_format: InsertTextFormat,
}

pub enum CompletionItemKind {
    Text,
    Method,
    Function,
    Constructor,
    Field,
    Variable,
    Class,
    Interface,
    Module,
    Property,
    Unit,
    Value,
    Enum,
    Keyword,
    Snippet,
    Color,
    File,
    Reference,
    Folder,
    EnumMember,
    Constant,
    Struct,
    Event,
    Operator,
    TypeParameter,
}

impl Completer {
    pub fn complete(&mut self, document: &Document, position: Position) -> Vec<CompletionItem> {
        // Try LSP completion first
        if let Some(ref lsp) = self.lsp {
            if let Ok(items) = lsp.complete(document, position) {
                return items;
            }
        }

        // Fall back to local completions
        self.local_completions.complete(document, position)
    }

    pub fn resolve_snippet(&self, snippet: &Snippet) -> String {
        // Expand snippet with tab stops
        let mut expanded = snippet.content.clone();

        // Replace tab stops with placeholders
        for tab_stop in &snippet.tab_stops {
            let placeholder = tab_stop.placeholder.as_ref().unwrap_or(&tab_stop.default);
            expanded = expanded.replace(&format!("${{{}}}", tab_stop.index), placeholder);
        }

        expanded
    }
}
```

## Multi-Cursor Editing

```rust
impl TextView {
    pub fn add_cursor(&mut self, position: Position) {
        self.cursors.push(Cursor {
            position,
            anchor: position,
            preferred_column: Some(position.column),
            insert_mode: false,
            auto_pair: true,
        });
    }

    pub fn remove_cursor(&mut self, index: usize) {
        if self.cursors.len() > 1 {
            self.cursors.remove(index);
        }
    }

    pub fn select_all_occurrences(&mut self, search_term: &str) {
        self.cursors.clear();

        let line_count = self.buffer.len_lines();
        for line_idx in 0..line_count {
            let line = self.buffer.line(line_idx).to_string();

            for (col_idx, _) in line.match_indices(search_term) {
                let start = Position {
                    line: line_idx,
                    column: col_idx,
                };
                let end = Position {
                    line: line_idx,
                    column: col_idx + search_term.len(),
                };

                let mut cursor = Cursor {
                    position: end,
                    anchor: start,
                    preferred_column: Some(end.column),
                    insert_mode: false,
                    auto_pair: true,
                };

                self.cursors.push(cursor);
            }
        }
    }

    pub fn insert_text_at_all_cursors(&mut self, text: &str) {
        for cursor in &mut self.cursors {
            let pos = cursor.position;
            self.buffer.insert(pos.line, pos.column, text);
            cursor.position.column += text.len();
            cursor.anchor = cursor.position;
        }
    }

    pub fn delete_selection_at_all_cursors(&mut self) {
        // Sort cursors in reverse order (end to start) to avoid position shifting
        let mut cursor_positions: Vec<_> = self.cursors.iter()
            .map(|c| c.position)
            .collect();
        cursor_positions.sort_by(|a, b| b.cmp(a));

        for cursor in &mut self.cursors {
            if let Selection::Normal { start, end } = cursor.selection {
                let (start, end) = if start < end { (start, end) } else { (end, start) };

                self.buffer.delete_range(start, end);

                cursor.position = start;
                cursor.anchor = start;
            }
        }
    }
}
```

## Git Integration

```rust
pub struct GitManager {
    /// Repository
    repo: Option<git2::Repository>,

    /// Index status
    index_status: HashMap<PathBuf, FileStatus>,

    /// Diff cache
    diff_cache: HashMap<PathBuf, Diff>,
}

pub enum FileStatus {
    Current,
    Modified,
    Added,
    Deleted,
    Renamed,
    Copied,
    Untracked,
    Ignored,
}

pub struct Diff {
    /// Hunks
    hunks: Vec<DiffHunk>,

    /// Old text
    old: Rope,

    /// New text
    new: Rope,
}

pub struct DiffHunk {
    /// Old range
    pub old_range: Range,

    /// New range
    pub new_range: Range,

    /// Lines
    pub lines: Vec<DiffLine>,
}

pub struct DiffLine {
    /// Line type
    pub line_type: DiffLineType,

    /// Content
    pub content: String,

    /// Old line number
    pub old_line: Option<usize>,

    /// New line number
    pub new_line: Option<usize>,
}

pub enum DiffLineType {
    Context,
    Added,
    Removed,
    Header,
}

impl GitManager {
    pub fn open(&mut self, path: &Path) -> Result<(), Error> {
        self.repo = Some(git2::Repository::open(path)?);
        self.refresh_status()?;
        Ok(())
    }

    pub fn refresh_status(&mut self) -> Result<(), Error> {
        if let Some(ref repo) = self.repo {
            self.index_status.clear();

            let statuses = repo.statuses(None)?;
            for entry in statuses.iter() {
                let path = entry.path().ok_or(Error::InvalidPath)?.into();
                let status = entry.status();

                let file_status = if status.is_index_new()
                    || status.is_worktree_new() {
                    FileStatus::Added
                } else if status.is_index_modified()
                    || status.is_worktree_modified() {
                    FileStatus::Modified
                } else if status.is_index_deleted()
                    || status.is_worktree_deleted() {
                    FileStatus::Deleted
                } else if status.is_index_renamed()
                    || status.is_worktree_renamed() {
                    FileStatus::Renamed
                } else if status.is_index_typechange()
                    || status.is_worktree_typechange() {
                    FileStatus::Copied
                } else if status.is_wt_new() {
                    FileStatus::Untracked
                } else if status.is_ignored() {
                    FileStatus::Ignored
                } else {
                    FileStatus::Current
                };

                self.index_status.insert(path, file_status);
            }
        }

        Ok(())
    }

    pub fn get_diff(&mut self, path: &Path) -> Result<&Diff, Error> {
        if !self.diff_cache.contains_key(path) {
            let diff = self.compute_diff(path)?;
            self.diff_cache.insert(path.into(), diff);
        }

        Ok(self.diff_cache.get(path).unwrap())
    }

    fn compute_diff(&self, path: &Path) -> Result<Diff, Error> {
        let repo = self.repo.as_ref().ok_or(Error::NoRepository)?;

        // Get tree
        let head = repo.head()?;
        let tree = head.peel_to_tree()?;

        // Get blob
        let tree_entry = tree.get_path(path)?;
        let old_blob: git2::Blob = tree_entry.to_object(&repo)?.peel()?;

        // Load new content
        let new_content = std::fs::read(path)?;

        // Compute diff
        let diff = repo.diff_blob_to_buffer(
            Some(&old_blob),
            path,
            Some(&new_content),
            path,
            None,
        )?;

        // Parse diff hunks
        let hunks = self.parse_diff_hunks(&diff)?;

        Ok(Diff {
            hunks,
            old: Rope::from_str(std::str::from_utf8(old_blob.content())?),
            new: Rope::from_str(std::str::from_utf8(&new_content)?),
        })
    }

    pub fn stage_file(&self, path: &Path) -> Result<(), Error> {
        let repo = self.repo.as_ref().ok_or(Error::NoRepository)?;
        let mut index = repo.index()?;

        index.add_path(path)?;
        index.write()?;

        Ok(())
    }

    pub fn commit(&self, message: &str) -> Result<git2::Oid, Error> {
        let repo = self.repo.as_ref().ok_or(Error::NoRepository)?;

        let mut index = repo.index()?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;

        let head = repo.head()?;
        let parent = head.peel_to_commit()?;

        let sig = repo.signature()?;
        let tree_id = repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            message,
            &tree,
            &[&parent],
        )?;

        Ok(tree_id)
    }
}
```

## LSP Client

```rust
pub struct LspClient {
    /// Process
    process: Child,

    /// Communication
    transport: LspTransport,

    /// Capabilities
    capabilities: ServerCapabilities,

    /// Version
    version: i32,
}

pub enum LspTransport {
    Stdio,
    Tcp { host: String, port: u16 },
}

impl LspClient {
    pub fn start(command: &str, args: &[String]) -> Result<Self, Error> {
        let process = std::process::Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let transport = LspTransport::Stdio;

        // Initialize
        let mut client = Self {
            process,
            transport,
            capabilities: ServerCapabilities::default(),
            version: 0,
        };

        client.initialize()?;

        Ok(client)
    }

    fn initialize(&mut self) -> Result<(), Error> {
        let params = InitializeParams {
            process_id: std::process::id(),
            root_uri: None,
            initialization_options: None,
            capabilities: ClientCapabilities::default(),
        };

        let result = self.send_request("initialize", params)?;
        self.capabilities = serde_json::from_value(result)?;

        // Send initialized notification
        self.send_notification("initialized", json!({}))?;

        Ok(())
    }

    pub fn open_document(&mut self, document: &Document) -> Result<(), Error> {
        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: document.uri(),
                language_id: document.language.id(),
                version: self.version,
                text: document.content.to_string(),
            },
        };

        self.send_notification("textDocument/didOpen", params)?;
        self.version += 1;

        Ok(())
    }

    pub fn change_document(&mut self, document: &Document, changes: &[TextChange]) -> Result<(), Error> {
        let params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                version: self.version,
                uri: document.uri(),
            },
            content_changes: changes.iter().map(|c| c.to_lsp()).collect(),
        };

        self.send_notification("textDocument/didChange", params)?;
        self.version += 1;

        Ok(())
    }

    pub fn complete(&mut self, document: &Document, position: Position) -> Result<Vec<CompletionItem>, Error> {
        let params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: document.uri(),
                },
                position: position.to_lsp(),
            },
            context: None,
        };

        let result = self.send_request("textDocument/completion", params)?;

        // Parse completion items
        Ok(serde_json::from_value(result)?)
    }

    pub fn goto_definition(&mut self, document: &Document, position: Position) -> Result<Vec<Location>, Error> {
        let params = GotoDefinitionParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: document.uri(),
                },
                position: position.to_lsp(),
            },
            work_done_progress_params: Default::default(),
        };

        let result = self.send_request("textDocument/definition", params)?;

        Ok(serde_json::from_value(result)?)
    }

    pub fn hover(&mut self, document: &Document, position: Position) -> Result<Hover, Error> {
        let params = HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: document.uri(),
                },
                position: position.to_lsp(),
            },
            work_done_progress_params: Default::default(),
        };

        let result = self.send_request("textDocument/hover", params)?;

        Ok(serde_json::from_value(result)?)
    }

    fn send_request(&mut self, method: &str, params: impl Serialize) -> Result<serde_json::Value, Error> {
        // Send request
        let request = json!({
            "jsonrpc": "2.0",
            "id": self.next_id(),
            "method": method,
            "params": params,
        });

        self.send(request)?;

        // Read response
        let response = self.read()?;
        Ok(response)
    }

    fn send_notification(&mut self, method: &str, params: impl Serialize) -> Result<(), Error> {
        let notification = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });

        self.send(notification)
    }

    fn send(&mut self, value: serde_json::Value) -> Result<(), Error> {
        let content = serde_json::to_vec(&value)?;
        let header = format!("Content-Length: {}\r\n\r\n", content.len());

        // Write to stdin
        let stdin = self.process.stdin.as_mut().ok_or(Error::NoStdin)?;
        stdin.write_all(header.as_bytes())?;
        stdin.write_all(&content)?;
        stdin.flush()?;

        Ok(())
    }

    fn read(&mut self) -> Result<serde_json::Value, Error> {
        // Read from stdout
        let stdout = self.process.stdout.as_mut().ok_or(Error::NoStdout)?;

        // Read Content-Length header
        let mut header = String::new();
        loop {
            let mut byte = [0u8; 1];
            stdout.read_exact(&mut byte)?;
            header.push(byte[0] as char);

            if header.ends_with("\r\n\r\n") {
                break;
            }
        }

        // Parse Content-Length
        let length_pos = header.find("Content-Length: ").ok_or(Error::InvalidHeader)?;
        let length_start = length_pos + 16;
        let length_end = header[length_start..].find("\r\n").ok_or(Error::InvalidHeader)?;
        let length: usize = header[length_start..length_start + length_end].parse()?;

        // Read content
        let mut content = vec![0u8; length];
        stdout.read_exact(&mut content)?;

        // Parse JSON
        Ok(serde_json::from_slice(&content)?)
    }

    fn next_id(&self) -> i32 {
        self.version + 1
    }
}
```

## File Structure

```
/var/www/rustux.com/prod/apps/gui/
├── rustica-edit/
│   ├── Cargo.toml
│   ├── resources/
│   │   ├── snippets/
│   │   ├── syntax/
│   │   └── themes/
│   └── src/
│       ├── main.rs
│       ├── app.rs
│       ├── notebook.rs
│       ├── tab.rs
│       ├── document.rs
│       ├── view.rs
│       ├── cursor.rs
│       ├── selection.rs
│       ├── highlighting.rs
│       ├── completion.rs
│       ├── multi_cursor.rs
│       ├── git.rs
│       ├── lsp.rs
│       ├── sidebar.rs
│       ├── minimap.rs
│       ├── command_palette.rs
│       └── settings.rs
```

## Dependencies

```toml
[package]
name = "rustica-edit"
version = "1.0.0"
edition = "2021"

[dependencies]
# GUI framework
librustica = { path = "../../../libs/librustica" }

# Text handling
ropey = "1.6"
unicode-segmentation = "1.0"

# Syntax highlighting
regex = "1.0"

# Git
git2 = "0.17"

# LSP
serde_json = "1.0"
serde = { version = "1.0", features = ["derive"] }

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# XDG
dirs = "5.0"

# File watching
notify = "5.0"

# Date/time
chrono = "0.4"
```

## Performance Requirements

| Metric | Target | Measurement |
|--------|--------|-------------|
| File open | <100ms | 1000 line file |
| Rendering | 60fps | Scrolling |
| Syntax highlight | <50ms | Per line |
| Completion | <200ms | Show items |
| Large file | <500ms | Open 100k lines |
| Memory | <200MB | Base usage |

## Success Criteria

- [ ] Basic editing works
- [ ] Syntax highlighting works
- [ ] Multi-cursor works
- [ ] LSP integration works
- [ ] Git integration works
- [ ] Tabs work
- [ ] Split views work
- [ ] Command palette works
- [ ] Snippets work
- [ ] Full accessibility
- [ ] Performance targets met

## Sign-Off

**Architect**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅

---

## Timeline

- Week 1: Basic editor + text view
- Week 2: Syntax highlighting + themes
- Week 3: Multi-cursor + selection
- Week 4: Code completion + snippets
- Week 5: Git integration
- Week 6: LSP client
- Week 7: Sidebar + minimap + polish

**Total**: 7 weeks
