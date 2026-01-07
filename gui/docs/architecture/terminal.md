# Terminal Emulator (rustica-term) Specification

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Component**: Rustica Shell - Terminal Emulator
**Phase**: 6.1 - Desktop Applications

## Overview

Rustica Term is a **modern, fast, and feature-rich terminal emulator** built on top of **librustica widgets** and using **VTE (Virtual Terminal Emulation)** or **alacritty's terminal emulation library**. It provides **tabbed interface**, **split panes**, **profile management**, **color schemes**, **keyboard shortcuts**, and **full accessibility**.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Rustica Term                                                           │
├─────────────────────────────────────────────────────────────────────────┤
│  [≡] [File] [Edit] [View] [Tabs] [Help]                    [□] [−] [×]  │
├─────────────────────────────────────────────────────────────────────────┤
│  [+] [user@host:~]  [user@host:~/dev]  [root@server]           [≡]     │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌────────────────────────────────────┬──────────────────────────────┐ │
│  │ $ neofetch                         │ $ cd /var/log               │ │
│  │                                   │ $ tail -f syslog           │ │
│  │        .---.                       │ 2025-01-07 10:30:15...     │ │
│  │       /     \                      │ 2025-01-07 10:30:16...     │ │
│  │      | O_O |                      │ ^C                         │ │
│  │      |  _  |                      │ $                           │ │
│  │       \___/                       │                             │ │
│  │                                   │                             │ │
│  │ OS: RUSTUX                        │                             │ │
│  │ Host: Rustica Book                │                             │ │
│  │ Kernel: 6.8.0-rustux              │                             │ │
│  │ Shell: rust                       │                             │ │
│  │                                   │                             │ │
│  └────────────────────────────────────┴──────────────────────────────┘ │
│  ┌──────────────────────────────────────────────────────────────────┐ │
│  │ $ cargo build --release                                         │ │
│  │    Compiling rustica-term v1.0.0                                │ │
│  │    Compiling librustica v0.1.0                                  │ │
│  │    Finished release [optimized] target(s) in 1.23s             │ │
│  │ $                                                               │ │
│  └──────────────────────────────────────────────────────────────────┘ │
│                                                                           │
│  1: bash  2: fish  3: zsh  4: rust    [Split] [New Tab] [Profile]      │
└─────────────────────────────────────────────────────────────────────────┘
```

## Main Application Structure

```rust
pub struct TerminalApp {
    /// Window
    window: Window,

    /// Notebook (tab container)
    notebook: Notebook,

    /// Profiles
    profiles: ProfileManager,

    /// Current profile
    current_profile: Profile,

    /// Configuration
    config: TerminalConfig,

    /// Color schemes
    color_schemes: ColorSchemeManager,

    /// Keyboard shortcuts
    shortcuts: ShortcutManager,

    /// Drag state
    drag_state: Option<DragState>,
}

pub struct Notebook {
    /// Tabs
    tabs: Vec<Tab>,

    /// Current tab index
    current_tab: usize,

    /// Tab position
    tab_position: TabPosition,

    /// Show tabs
    show_tabs: bool,
}

pub enum TabPosition {
    Top,
    Bottom,
    Left,
    Right,
}

pub struct Tab {
    /// Tab widget
    widget: TabWidget,

    /// Terminal container
    terminal_container: TerminalContainer,

    /// Title
    title: String,

    /// Custom title (if set)
    custom_title: Option<String>,

    /// Icon
    icon: Option<Icon>,

    /// Modified (has output)
    modified: bool,

    /// Busy (running command)
    busy: bool,
}

pub struct TerminalContainer {
    /// Split layout
    layout: SplitLayout,

    /// Terminals in this container
    terminals: Vec<Terminal>,
}

pub enum SplitLayout {
    Single(Box<Terminal>),
    Horizontal { left: Box<TerminalContainer>, right: Box<TerminalContainer> },
    Vertical { top: Box<TerminalContainer>, bottom: Box<TerminalContainer> },
}

pub struct Terminal {
    /// PTY master
    pty: Pty,

    /// Terminal widget (renders text)
    widget: TerminalWidget,

    /// Shell process
    shell: Child,

    /// Terminal ID
    id: TerminalId,

    /// Profile
    profile: Profile,

    /// Working directory
    working_directory: PathBuf,

    /// Title
    title: String,

    /// Hold on exit
    hold: bool,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TerminalId(String);
```

## Terminal Widget

```rust
pub struct TerminalWidget {
    /// Grid of characters
    grid: Grid,

    /// Cursor
    cursor: Cursor,

    /// Selection
    selection: Option<Selection>,

    /// Scroll offset
    scroll_offset: usize,

    /// Scrollback
    scrollback: ScrollbackBuffer,

    /// Colors
    colors: ColorPalette,

    /// Font
    font: Font,

    /// Bold font
    bold_font: Font,

    /// Italic font
    italic_font: Font,

    /// Cell size
    cell_size: Size,

    /// Margin
    margin: Padding,

    /// Blink state
    blink_state: bool,

    /// Search
    search: Option<Search>,
}

pub struct Grid {
    /// Rows
    rows: usize,

    /// Columns
    cols: usize,

    /// Cells
    cells: Vec<Vec<Cell>>,
}

pub struct Cell {
    /// Character
    c: char,

    /// Foreground color
    fg: Color,

    /// Background color
    bg: Color,

    /// Attributes
    attrs: CellAttrs,
}

pub struct CellAttrs {
    bold: bool,
    dim: bool,
    italic: bool,
    underline: bool,
    blink: bool,
    reverse: bool,
    hidden: bool,
    strikethrough: bool,
}

pub struct Cursor {
    /// Row (0-indexed)
    row: usize,

    /// Column (0-indexed)
    col: usize,

    /// Visible
    visible: bool,

    /// Shape
    shape: CursorShape,

    /// Blinking
    blinking: bool,
}

pub enum CursorShape {
    Block,
    Underline,
    Bar,
}

pub struct Selection {
    /// Start position
    start: Position,

    /// End position
    end: Position,

    /// Selection type
    kind: SelectionKind,
}

pub enum SelectionKind {
    Normal,
    Rectangular,
    Smart,
}

pub struct ScrollbackBuffer {
    /// Lines
    lines: Vec<Line>,

    /// Max lines
    max_lines: usize,

    /// Current position
    position: usize,
}

pub struct ColorPalette {
    /// Normal colors (0-7)
    normal: [Color; 8],

    /// Bright colors (8-15)
    bright: [Color; 8],

    /// Special colors
    foreground: Color,
    background: Color,
    cursor: Color,
    cursor_foreground: Color,
    highlight: Color,
    highlight_foreground: Color,
}
```

## PTY Handling

```rust
pub struct Pty {
    /// Master file descriptor
    master: RawFd,

    /// Slave file descriptor
    slave: RawFd,

    /// Size
    size: PtySize,

    /// Read buffer
    read_buffer: Vec<u8>,
}

pub struct PtySize {
    pub rows: u16,
    pub cols: u16,
    pub pixel_width: u16,
    pub pixel_height: u16,
}

impl Pty {
    pub fn new(size: PtySize) -> Result<Self, Error> {
        // Open PTY master
        let master = posix_openpt(O_RDWR | O_NOCTTY)?;

        // Grant access
        grantpt(&master)?;
        unlockpt(&master)?;

        // Get slave name
        let slave_name = unsafe {
            std::ffi::CStr::from_ptr(ptsname(&master))
        };

        // Open slave
        let slave = open(slave_name, O_RDWR)?;

        Ok(Self {
            master,
            slave,
            size,
            read_buffer: Vec::new(),
        })
    }

    pub fn fork_shell(&self, profile: &Profile) -> Result<Child, Error> {
        unsafe {
            match fork() {
                ForkResult::Parent { child } => {
                    // Close slave in parent
                    close(self.slave);

                    Ok(child)
                }

                ForkResult::Child => {
                    // Create new session
                    setsid();

                    // Set slave as controlling terminal
                    ioctl::libc::ioctl(self.slave, TIOCSCTTY, 0);

                    // Duplicate slave to stdin/stdout/stderr
                    dup2(self.slave, 0);
                    dup2(self.slave, 1);
                    dup2(self.slave, 2);

                    // Close master in child
                    close(self.master);
                    close(self.slave);

                    // Set environment
                    for (key, value) in &profile.env {
                        std::env::set_var(key, value);
                    }

                    // Set terminal size
                    self.set_size(&self.size);

                    // Execute shell
                    execvp(&profile.shell, &profile.args)?;

                    // Should not reach here
                    std::process::exit(1);
                }
            }
        }
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let n = posix::read(self.master, buf)?;
        Ok(n)
    }

    pub fn write(&self, buf: &[u8]) -> Result<usize, Error> {
        let n = posix::write(self.master, buf)?;
        Ok(n)
    }

    pub fn set_size(&self, size: &PtySize) -> Result<(), Error> {
        let winsize = Winsize {
            ws_row: size.rows,
            ws_col: size.cols,
            ws_xpixel: size.pixel_width,
            ws_ypixel: size.pixel_height,
        };

        ioctl::libc::ioctl(self.master, TIOCSWINSZ, &winsize)?;

        Ok(())
    }
}
```

## Terminal Emulation

```rust
pub struct Parser {
    /// State
    state: ParserState,

    /// Escape sequence buffer
    escape_buffer: Vec<u8>,

    /// Charset
    charset: Charset,

    /// Mode
    mode: Mode,
}

pub enum ParserState {
    Ground,
    Escape,
    EscapeIntermediate,
    CsiEntry,
    CsiParam,
    CsiIntermediate,
    CsiIgnore,
    DcsEntry,
    DcsParam,
    DcsIntermediate,
    DcsPassthrough,
    DcsIgnore,
    OscString,
    SosPmApcString,
}

pub struct Charset {
    g0: CharsetTable,
    g1: CharsetTable,
    g2: CharsetTable,
    g3: CharsetTable,
    current_g: usize,
}

pub enum CharsetTable {
    Ascii,
    LineDrawing,
}

pub struct Mode {
    /// Application cursor keys
    app_cursor_keys: bool,

    /// Application keypad
    app_keypad: bool,

    /// Insert mode
    insert: bool,

    /// Auto wrap
    auto_wrap: bool,

    /// Origin mode
    origin: bool,

    /// Reverse video
    reverse: bool,

    /// Hide cursor
    hide_cursor: bool,
}

impl Parser {
    pub fn advance(&mut self, terminal: &mut TerminalWidget, byte: u8) {
        match self.state {
            ParserState::Ground => {
                match byte {
                    0x00..=0x1F | 0x7F => {
                        // Control characters
                        self.handle_control(terminal, byte);
                    }

                    0x20..=0x7E => {
                        // Printable characters
                        terminal.print_char(byte as char);
                    }

                    0x1B => {
                        // ESC
                        self.state = ParserState::Escape;
                        self.escape_buffer.clear();
                    }

                    _ => {}
                }
            }

            ParserState::Escape => {
                match byte {
                    b'[' => self.state = ParserState::CsiEntry,
                    b']' => self.state = ParserState::OscString,
                    b'P' => self.state = ParserState::DcsEntry,
                    b'\\' => self.state = ParserState::Ground,
                    b'=' => {
                        self.mode.app_keypad = true;
                        self.state = ParserState::Ground;
                    }
                    b'>' => {
                        self.mode.app_keypad = false;
                        self.state = ParserState::Ground;
                    }
                    _ => {
                        self.state = ParserState::Ground;
                    }
                }
            }

            ParserState::CsiEntry => {
                match byte {
                    b'0'..=b'9' | b';' | b':' => {
                        self.escape_buffer.push(byte);
                        self.state = ParserState::CsiParam;
                    }

                    b'?' => {
                        self.escape_buffer.push(byte);
                        self.state = ParserState::CsiParam;
                    }

                    b'A'..=b'Z' | b'a'..=b'z' => {
                        self.handle_csi_dispatch(terminal, byte);
                        self.state = ParserState::Ground;
                    }

                    _ => {
                        self.state = ParserState::CsiIgnore;
                    }
                }
            }

            _ => {}
        }
    }

    fn handle_control(&mut self, terminal: &mut TerminalWidget, byte: u8) {
        match byte {
            0x08 => terminal.backspace(),           // BS
            0x09 => terminal.tab(),                 // TAB
            0x0A => terminal.line_feed(),           // LF
            0x0D => terminal.carriage_return(),     // CR
            0x0E => self.shift_out(),               // SO
            0x0F => self.shift_in(),                // SI
            _ => {}
        }
    }

    fn handle_csi_dispatch(&mut self, terminal: &mut TerminalWidget, byte: u8) {
        let params = self.parse_params();

        match byte {
            b'A' => terminal.cursor_up(params),              // CUU
            b'B' => terminal.cursor_down(params),            // CUD
            b'C' => terminal.cursor_forward(params),         // CUF
            b'D' => terminal.cursor_back(params),            // CUB
            b'E' => terminal.cursor_next_line(params),       // CNL
            b'F' => terminal.cursor_previous_line(params),   // CPL
            b'G' => terminal.cursor_horizontal_abs(params),  // CHA
            b'H' => terminal.cursor_position(params),        // CUP
            b'J' => terminal.erase_in_display(params),       // ED
            b'K' => terminal.erase_in_line(params),          // EL
            b'P' => terminal.delete_character(params),       // DCH
            b'X' => terminal.erase_character(params),        // ECH
            b'm' => terminal.select_graphic_rendition(params), // SGR
            b'r' => terminal.set_margins(params),            // DECSTBM
            b's' => terminal.save_cursor(),                  // SCOSC
            b'u' => terminal.restore_cursor(),               // SCORC
            _ => {}
        }
    }

    fn parse_params(&self) -> Vec<i64> {
        let buffer = &self.escape_buffer;
        let mut params = Vec::new();
        let mut current = String::new();

        for &byte in buffer {
            match byte {
                b'0'..=b'9' | b'-' => current.push(byte as char),
                b';' | b':' => {
                    if !current.is_empty() {
                        params.push(current.parse().unwrap_or(0));
                        current = String::new();
                    } else {
                        params.push(0);
                    }
                }
                _ => {}
            }
        }

        if !current.is_empty() {
            params.push(current.parse().unwrap_or(0));
        }

        if params.is_empty() {
            params.push(0);
        }

        params
    }
}
```

## Profiles

```rust
pub struct Profile {
    /// Profile name
    pub name: String,

    /// Shell command
    pub shell: String,

    /// Shell arguments
    pub args: Vec<String>,

    /// Working directory
    pub working_directory: Option<PathBuf>,

    /// Environment variables
    pub env: HashMap<String, String>,

    /// Color scheme
    pub color_scheme: String,

    /// Font
    pub font: FontConfig,

    /// Cursor shape
    pub cursor_shape: CursorShape,

    /// Cursor blinking
    pub cursor_blink: bool,

    /// Audible bell
    pub audible_bell: bool,

    /// Visible bell
    pub visible_bell: bool,

    /// Scrollback lines
    pub scrollback_lines: usize,

    /// Terminal type string
    pub term: String,

    /// Encoding
    pub encoding: String,
}

pub struct ProfileManager {
    /// Profiles
    profiles: Vec<Profile>,

    /// Default profile
    default_profile: String,
}

impl ProfileManager {
    pub fn load() -> Result<Self, Error> {
        let config_path = dirs::config_dir()
            .ok_or(Error::NoConfigDir)?
            .join("rustica-term/profiles.toml");

        let profiles = if config_path.exists() {
            self.load_from_file(&config_path)?
        } else {
            Self::default_profiles()
        };

        Ok(Self {
            profiles,
            default_profile: "Default".into(),
        })
    }

    fn default_profiles() -> Vec<Profile> {
        vec![
            Profile {
                name: "Default".into(),
                shell: "/bin/bash".into(),
                args: vec!["--login".into()],
                working_directory: None,
                env: HashMap::new(),
                color_scheme: "Rustica Dark".into(),
                font: FontConfig {
                    family: "JetBrains Mono".into(),
                    size: 12.0,
                },
                cursor_shape: CursorShape::Block,
                cursor_blink: true,
                audible_bell: false,
                visible_bell: true,
                scrollback_lines: 10000,
                term: "xterm-256color".into(),
                encoding: "UTF-8".into(),
            },
            Profile {
                name: "Rust".into(),
                shell: "/usr/bin/rust".into(),
                args: vec![],
                working_directory: None,
                env: HashMap::new(),
                color_scheme: "Rustica Dark".into(),
                font: FontConfig {
                    family: "JetBrains Mono".into(),
                    size: 12.0,
                },
                cursor_shape: CursorShape::Bar,
                cursor_blink: true,
                audible_bell: false,
                visible_bell: true,
                scrollback_lines: 10000,
                term: "xterm-256color".into(),
                encoding: "UTF-8".into(),
            },
        ]
    }
}
```

## Color Schemes

```rust
pub struct ColorScheme {
    /// Scheme name
    pub name: String,

    /// Palette
    pub palette: ColorPalette,
}

pub struct ColorSchemeManager {
    /// Schemes
    schemes: HashMap<String, ColorScheme>,
}

impl ColorSchemeManager {
    pub fn load() -> Result<Self, Error> {
        let schemes_path = dirs::data_dir()
            .ok_or(Error::NoDataDir)?
            .join("rustica-term/schemes");

        let mut schemes = HashMap::new();

        // Load built-in schemes
        schemes.extend(Self::builtin_schemes());

        // Load user schemes
        if schemes_path.exists() {
            for entry in std::fs::read_dir(schemes_path)? {
                let entry = entry?;
                let path = entry.path();

                if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    if let Ok(scheme) = Self::load_scheme(&path) {
                        schemes.insert(scheme.name.clone(), scheme);
                    }
                }
            }
        }

        Ok(Self { schemes })
    }

    fn builtin_schemes() -> HashMap<String, ColorScheme> {
        let mut schemes = HashMap::new();

        // Rustica Dark
        schemes.insert(
            "Rustica Dark".into(),
            ColorScheme {
                name: "Rustica Dark".into(),
                palette: ColorPalette {
                    normal: [
                        Color::rgb(39, 40, 34),    // Black
                        Color::rgb(192, 57, 43),   // Red
                        Color::rgb(133, 153, 0),   // Green
                        Color::rgb(220, 189, 68),  // Yellow
                        Color::rgb(88, 126, 172),  // Blue
                        Color::rgb(175, 106, 135), // Magenta
                        Color::rgb(64, 169, 171),  // Cyan
                        Color::rgb(238, 232, 213), // White
                    ],
                    bright: [
                        Color::rgb(117, 117, 110), // Bright Black
                        Color::rgb(237, 100, 81),   // Bright Red
                        Color::rgb(186, 208, 64),   // Bright Green
                        Color::rgb(237, 216, 106),  // Bright Yellow
                        Color::rgb(131, 165, 210),  // Bright Blue
                        Color::rgb(214, 137, 168),  // Bright Magenta
                        Color::rgb(104, 187, 189),  // Bright Cyan
                        Color::rgb(255, 255, 255),  // Bright White
                    ],
                    foreground: Color::rgb(238, 232, 213),
                    background: Color::rgb(27, 29, 30),
                    cursor: Color::rgb(238, 232, 213),
                    cursor_foreground: Color::rgb(27, 29, 30),
                    highlight: Color::rgb(88, 126, 172),
                    highlight_foreground: Color::rgb(238, 232, 213),
                },
            },
        );

        // Solarized Light
        schemes.insert(
            "Solarized Light".into(),
            ColorScheme {
                name: "Solarized Light".into(),
                palette: ColorPalette {
                    normal: [
                        Color::rgb(7, 54, 66),
                        Color::rgb(220, 50, 47),
                        Color::rgb(133, 153, 0),
                        Color::rgb(181, 137, 0),
                        Color::rgb(38, 139, 210),
                        Color::rgb(211, 54, 130),
                        Color::rgb(42, 161, 152),
                        Color::rgb(238, 232, 213),
                    ],
                    bright: [
                        Color::rgb(0, 43, 54),
                        Color::rgb(203, 75, 22),
                        Color::rgb(88, 110, 117),
                        Color::rgb(101, 123, 131),
                        Color::rgb(131, 148, 150),
                        Color::rgb(108, 113, 196),
                        Color::rgb(147, 161, 161),
                        Color::rgb(253, 246, 227),
                    ],
                    foreground: Color::rgb(101, 123, 131),
                    background: Color::rgb(253, 246, 227),
                    cursor: Color::rgb(101, 123, 131),
                    cursor_foreground: Color::rgb(253, 246, 227),
                    highlight: Color::rgb(238, 232, 213),
                    highlight_foreground: Color::rgb(7, 54, 66),
                },
            },
        );

        schemes
    }
}
```

## Split Panes

```rust
impl TerminalContainer {
    pub fn split_horizontal(&mut self) -> Result<(), Error> {
        match &self.layout {
            SplitLayout::Single(terminal) => {
                let left = TerminalContainer {
                    layout: SplitLayout::Single(Box::new(terminal.clone())),
                };

                let right = TerminalContainer {
                    layout: SplitLayout::Single(Box::new(Terminal::new(
                        terminal.profile.clone(),
                        terminal.working_directory.clone(),
                    )?)),
                };

                self.layout = SplitLayout::Horizontal {
                    left: Box::new(left),
                    right: Box::new(right),
                };

                Ok(())
            }

            _ => Err(Error::CannotSplit),
        }
    }

    pub fn split_vertical(&mut self) -> Result<(), Error> {
        match &self.layout {
            SplitLayout::Single(terminal) => {
                let top = TerminalContainer {
                    layout: SplitLayout::Single(Box::new(terminal.clone())),
                };

                let bottom = TerminalContainer {
                    layout: SplitLayout::Single(Box::new(Terminal::new(
                        terminal.profile.clone(),
                        terminal.working_directory.clone(),
                    )?)),
                };

                self.layout = SplitLayout::Vertical {
                    top: Box::new(top),
                    bottom: Box::new(bottom),
                };

                Ok(())
            }

            _ => Err(Error::CannotSplit),
        }
    }

    pub fn close_current(&mut self) -> Result<(), Error> {
        match &mut self.layout {
            SplitLayout::Single(_) => Err(Error::CannotCloseLastPane),

            SplitLayout::Horizontal { left, right } => {
                // Focus the other pane
                Ok(())
            }

            SplitLayout::Vertical { top, bottom } => {
                // Focus the other pane
                Ok(())
            }
        }
    }

    pub fn navigate(&mut self, direction: Direction) {
        match direction {
            Direction::Left => self.navigate_left(),
            Direction::Right => self.navigate_right(),
            Direction::Up => self.navigate_up(),
            Direction::Down => self.navigate_down(),
        }
    }
}
```

## Search

```rust
pub struct Search {
    /// Query
    query: String,

    /// Case sensitive
    case_sensitive: bool,

    /// Regex
    regex: bool,

    /// Matches
    matches: Vec<Match>,

    /// Current match index
    current_match: usize,

    /// Wrapped search
    wrapped: bool,
}

pub struct Match {
    /// Start position
    start: Position,

    /// End position
    end: Position,
}

impl TerminalWidget {
    pub fn search(&mut self, query: &str, case_sensitive: bool, regex: bool) {
        self.search = Some(Search {
            query: query.to_string(),
            case_sensitive,
            regex,
            matches: Vec::new(),
            current_match: 0,
            wrapped: false,
        });

        // Perform search
        if let Some(search) = &mut self.search {
            for (row_idx, row) in self.grid.cells.iter().enumerate() {
                for (col_idx, cell) in row.iter().enumerate() {
                    // Check for match
                    if self.cell_matches_query(cell, &search.query, search.case_sensitive) {
                        search.matches.push(Match {
                            start: Position { row: row_idx, col: col_idx },
                            end: Position { row: row_idx, col: col_idx + 1 },
                        });
                    }
                }
            }
        }

        // Jump to first match
        self.jump_to_next_match();
    }

    pub fn jump_to_next_match(&mut self) {
        if let Some(ref mut search) = self.search {
            if !search.matches.is_empty() {
                search.current_match = (search.current_match + 1) % search.matches.len();

                if let Some(m) = search.matches.get(search.current_match) {
                    self.scroll_to(m.start.row);
                }
            }
        }
    }

    pub fn jump_to_prev_match(&mut self) {
        if let Some(ref mut search) = self.search {
            if !search.matches.is_empty() {
                search.current_match = if search.current_match == 0 {
                    search.matches.len() - 1
                } else {
                    search.current_match - 1
                };

                if let Some(m) = search.matches.get(search.current_match) {
                    self.scroll_to(m.start.row);
                }
            }
        }
    }
}
```

## Keyboard Shortcuts

```rust
pub struct ShortcutManager {
    /// Shortcuts
    shortcuts: HashMap<Shortcut, Action>,
}

pub struct Shortcut {
    pub key: Key,
    pub modifiers: Modifiers,
}

pub enum Key {
    Char(char),
    Tab,
    Enter,
    Escape,
    Backspace,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    Up,
    Down,
    Left,
    Right,
    F(u8),
}

pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub logo: bool, // Super/Windows key
}

pub enum Action {
    NewTab,
    NewWindow,
    CloseTab,
    NextTab,
    PreviousTab,
    SplitHorizontal,
    SplitVertical,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    Copy,
    Paste,
    SelectAll,
    Find,
    FindNext,
    FindPrevious,
    ToggleFullscreen,
    Reset,
    Clear,
}

impl ShortcutManager {
    pub fn default_shortcuts() -> HashMap<Shortcut, Action> {
        let mut shortcuts = HashMap::new();

        // Tab management
        shortcuts.insert(Shortcut {
            key: Key::Char('t'),
            modifiers: Modifiers { ctrl: true, ..Default::default() },
        }, Action::NewTab);

        shortcuts.insert(Shortcut {
            key: Key::Char('w'),
            modifiers: Modifiers { ctrl: true, ..Default::default() },
        }, Action::CloseTab);

        shortcuts.insert(Shortcut {
            key: Key::PageDown,
            modifiers: Modifiers { ctrl: true, ..Default::default() },
        }, Action::NextTab);

        shortcuts.insert(Shortcut {
            key: Key::PageUp,
            modifiers: Modifiers { ctrl: true, ..Default::default() },
        }, Action::PreviousTab);

        // Splitting
        shortcuts.insert(Shortcut {
            key: Key::Char('d'),
            modifiers: Modifiers { ctrl: true, shift: true, ..Default::default() },
        }, Action::SplitHorizontal);

        shortcuts.insert(Shortcut {
            key: Key::Char('d'),
            modifiers: Modifiers { ctrl: true, alt: true, ..Default::default() },
        }, Action::SplitVertical);

        // Zoom
        shortcuts.insert(Shortcut {
            key: Key::Char('+'),
            modifiers: Modifiers { ctrl: true, ..Default::default() },
        }, Action::ZoomIn);

        shortcuts.insert(Shortcut {
            key: Key::Char('-'),
            modifiers: Modifiers { ctrl: true, ..Default::default() },
        }, Action::ZoomOut);

        shortcuts.insert(Shortcut {
            key: Key::Char('0'),
            modifiers: Modifiers { ctrl: true, ..Default::default() },
        }, Action::ZoomReset);

        // Copy/Paste
        shortcuts.insert(Shortcut {
            key: Key::Char('c'),
            modifiers: Modifiers { ctrl: true, shift: true, ..Default::default() },
        }, Action::Copy);

        shortcuts.insert(Shortcut {
            key: Key::Char('v'),
            modifiers: Modifiers { ctrl: true, shift: true, ..Default::default() },
        }, Action::Paste);

        // Search
        shortcuts.insert(Shortcut {
            key: Key::Char('f'),
            modifiers: Modifiers { ctrl: true, shift: true, ..Default::default() },
        }, Action::Find);

        // Fullscreen
        shortcuts.insert(Shortcut {
            key: Key::F(11),
            modifiers: Default::default(),
        }, Action::ToggleFullscreen);

        shortcuts
    }
}
```

## File Structure

```
/var/www/rustux.com/prod/apps/gui/
├── rustica-term/
│   ├── Cargo.toml
│   ├── resources/
│   │   ├── icons/
│   │   └── schemes/
│   │       ├── rustica-dark.toml
│   │       ├── solarized-light.toml
│   │       └── nord.toml
│   └── src/
│       ├── main.rs
│       ├── app.rs
│       ├── notebook.rs
│       ├── tab.rs
│       ├── container.rs
│       ├── terminal.rs
│       ├── widget.rs
│       ├── pty.rs
│       ├── parser.rs
│       ├── grid.rs
│       ├── vte/
│       │   ├── parser.rs
│       │   └── ansi.rs
│       ├── profiles.rs
│       ├── colors.rs
│       ├── shortcuts.rs
│       ├── search.rs
│       └── split.rs
```

## Dependencies

```toml
[package]
name = "rustica-term"
version = "1.0.0"
edition = "2021"

[dependencies]
# GUI framework
librustica = { path = "../../../libs/librustica" }

# Terminal emulation
vte = "0.13"

# PTY handling
libc = "0.2"
nix = "0.26"

# Fonts
fontconfig = "0.5"
freetype-rs = "0.36"

# Text handling
unicode-width = "0.1"
unicode-segmentation = "1.0"

# Regular expressions (for search)
regex = "1.0"

# Serialization
serde = "1.0"
toml = "0.8"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# XDG
dirs = "5.0"
```

## Performance Requirements

| Metric | Target | Measurement |
|--------|--------|-------------|
| Render FPS | 60fps | Full scroll |
| Input latency | <10ms | Keystroke to visible |
| Startup time | <200ms | Launch to usable |
| Memory per terminal | <50MB | After 10k lines |
| CPU usage | <1% | Idle |

## Success Criteria

- [ ] Terminal emulation works correctly
- [ ] PTY handling is robust
- [ ] Tabs and splits work
- [ ] Color schemes apply correctly
- [ ] Keyboard shortcuts configurable
- [ ] Search works
- [ ] Copy/paste works
- [ ] Full accessibility support
- [ ] Performance targets met

## Sign-Off

**Architect**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅

---

## Timeline

- Week 1: Basic terminal widget + PTY handling
- Week 2: VTE parser + ANSI escape sequences
- Week 3: Tabs + split panes
- Week 4: Profiles + color schemes + shortcuts
- Week 5: Search + polish + testing

**Total**: 5 weeks
