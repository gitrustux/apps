// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! editor - Simple Text Editor
//!
//! A nano-like text editor for Rustica OS.

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self, ClearType},
};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Simple Text Editor
#[derive(Parser, Debug)]
#[command(name = "editor")]
#[command(author = "The Rustux Authors")]
#[command(version = "0.1.0")]
#[command(about = "Simple Text Editor (nano-like)", long_about = None)]
struct Args {
    /// File to edit
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// Line number to start at
    #[arg(short, long)]
    line: Option<usize>,

    /// Read-only mode
    #[arg(short, long)]
    readonly: bool,
}

/// Simple line-based text editor
struct TextEditor {
    lines: Vec<String>,
    filename: Option<PathBuf>,
    modified: bool,
    cursor_row: usize,
    cursor_col: usize,
    row_offset: usize,
    col_offset: usize,
    readonly: bool,
    status_message: String,
    mode: EditorMode,
}

#[derive(PartialEq)]
enum EditorMode {
    Normal,
    Saving,
    Quitting,
    Searching,
}

impl TextEditor {
    fn new() -> Self {
        Self {
            lines: vec![String::new()],
            filename: None,
            modified: false,
            cursor_row: 0,
            cursor_col: 0,
            row_offset: 0,
            col_offset: 0,
            readonly: false,
            status_message: String::new(),
            mode: EditorMode::Normal,
        }
    }

    fn load_file(&mut self, path: &PathBuf) -> Result<()> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        self.lines = content.lines().map(|s| s.to_string()).collect();
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }

        self.filename = Some(path.clone());
        self.modified = false;
        self.status_message = format!("Read {} lines", self.lines.len());

        Ok(())
    }

    fn save_file(&mut self) -> Result<()> {
        if let Some(ref path) = self.filename {
            let mut file = File::create(path)
                .with_context(|| format!("Failed to create file: {}", path.display()))?;

            for line in &self.lines {
                writeln!(file, "{}", line)
                    .with_context(|| format!("Failed to write to file: {}", path.display()))?;
            }

            self.modified = false;
            self.status_message = format!("Saved {} lines", self.lines.len());
            Ok(())
        } else {
            anyhow::bail!("No filename specified");
        }
    }

    fn insert_char(&mut self, c: char) {
        if self.readonly {
            self.status_message = "Read-only mode".to_string();
            return;
        }

        let row = &mut self.lines[self.cursor_row];
        if self.cursor_col <= row.len() {
            row.insert(self.cursor_col, c);
            self.cursor_col += 1;
            self.modified = true;
        }
    }

    fn delete_char(&mut self) {
        if self.readonly {
            self.status_message = "Read-only mode".to_string();
            return;
        }

        let row = &mut self.lines[self.cursor_row];
        if self.cursor_col > 0 {
            row.remove(self.cursor_col - 1);
            self.cursor_col -= 1;
            self.modified = true;
        } else if self.cursor_row > 0 {
            // Join with previous line
            let prev_row = self.lines.remove(self.cursor_row - 1);
            let curr_row = self.lines.remove(self.cursor_row - 1);
            self.cursor_col = prev_row.len();
            self.lines.insert(self.cursor_row, prev_row + &curr_row);
            self.modified = true;
        }
    }

    fn enter_new_line(&mut self) {
        if self.readonly {
            self.status_message = "Read-only mode".to_string();
            return;
        }

        let row = &mut self.lines[self.cursor_row];
        let before: String = row.chars().take(self.cursor_col).collect();
        let after: String = row.chars().skip(self.cursor_col).collect();

        self.lines[self.cursor_row] = before;
        self.lines.insert(self.cursor_row + 1, after);
        self.cursor_row += 1;
        self.cursor_col = 0;
        self.modified = true;
    }

    fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            let max_col = self.lines[self.cursor_row].len();
            if self.cursor_col > max_col {
                self.cursor_col = max_col;
            }
        }
    }

    fn move_down(&mut self) {
        if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            let max_col = self.lines[self.cursor_row].len();
            if self.cursor_col > max_col {
                self.cursor_col = max_col;
            }
        }
    }

    fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        }
    }

    fn move_right(&mut self) {
        let max_col = self.lines[self.cursor_row].len();
        if self.cursor_col < max_col {
            self.cursor_col += 1;
        }
    }

    fn render(&mut self) -> Result<()> {
        let (width, height) = terminal::size()?;
        let height = height.saturating_sub(2) as usize; // Reserve space for status bar

        // Adjust offset if cursor is off screen
        if self.cursor_row < self.row_offset {
            self.row_offset = self.cursor_row;
        } else if self.cursor_row >= self.row_offset + height {
            self.row_offset = self.cursor_row - height + 1;
        }

        if self.cursor_col < self.col_offset {
            self.col_offset = self.cursor_col;
        } else if self.cursor_col >= self.col_offset + width as usize {
            self.col_offset = self.cursor_col - width as usize + 1;
        }

        execute!(io::stdout(), terminal::Clear(ClearType::All))?;

        for i in 0..height {
            let row_idx = self.row_offset + i;
            if row_idx < self.lines.len() {
                let row = &self.lines[row_idx];
                let display_row: String = row.chars().skip(self.col_offset).take(width as usize).collect();
                println!("{}", display_row);
            } else if row_idx == self.lines.len() - 1 {
                println!("~");
            } else {
                println!("~");
            }
        }

        // Status bar
        let filename = self.filename.as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("New Buffer");

        let modified_flag = if self.modified { " [Modified]" } else { "" };
        let readonly_flag = if self.readonly { " [Readonly]" } else { "" };
        let pos = format!("{}, {}", self.cursor_row + 1, self.cursor_col + 1);

        execute!(
            io::stdout(),
            cursor::MoveTo(1, height as u16 + 2),
            terminal::Clear(ClearType::CurrentLine),
            crossterm::style::SetForegroundColor(crossterm::style::Color::Blue)
        )?;

        print!("{}{}{} | {}", filename, modified_flag, readonly_flag, pos);

        if !self.status_message.is_empty() {
            execute!(io::stdout(), cursor::MoveTo(1, height as u16 + 3))?;
            println!("{}", self.status_message);
            self.status_message.clear();
        }

        execute!(io::stdout(), cursor::MoveTo(
            (self.cursor_col - self.col_offset + 1) as u16,
            (self.cursor_row - self.row_offset + 1) as u16,
        ))?;

        io::stdout().flush()?;

        Ok(())
    }

    fn run(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            loop {
                match event::poll(Duration::from_millis(100)) {
                    Ok(true) => {
                        if let Ok(event) = event::read() {
                            if let Event::Key(key) = event {
                                let _ = tx.send(key);
                            }
                        }
                    }
                    Ok(false) => {
                        continue;
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
        });

        let result = self.event_loop(rx);

        terminal::disable_raw_mode()?;
        execute!(io::stdout(), terminal::Clear(ClearType::All))?;

        result
    }

    fn event_loop(&mut self, rx: mpsc::Receiver<KeyEvent>) -> Result<()> {
        loop {
            self.render()?;

            let key = rx.recv()?;

            if self.mode == EditorMode::Quitting {
                if self.modified {
                    self.mode = EditorMode::Normal;
                    self.status_message = "File modified. Use Ctrl+O to save or Ctrl+X again to quit without saving.".to_string();
                } else {
                    return Ok(());
                }
                continue;
            }

            match key.code {
                KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if self.modified {
                        self.mode = EditorMode::Quitting;
                    } else {
                        return Ok(());
                    }
                }
                KeyCode::Char('o') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if let Some(_) = self.filename {
                        self.save_file()?;
                    } else {
                        self.status_message = "No filename - cannot save".to_string();
                    }
                }
                KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.mode = EditorMode::Saving;
                    self.status_message = "Enter filename: ".to_string();
                }
                KeyCode::Char('x') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if self.modified {
                        self.mode = EditorMode::Quitting;
                    } else {
                        return Ok(());
                    }
                }
                KeyCode::Up => self.move_up(),
                KeyCode::Down => {
                    self.move_down();
                }
                KeyCode::Left => self.move_left(),
                KeyCode::Right => self.move_right(),
                KeyCode::Enter => self.enter_new_line(),
                KeyCode::Backspace => self.delete_char(),
                KeyCode::Char(c) => self.insert_char(c),
                KeyCode::Esc => {
                    return Ok(());
                }
                _ => {}
            }
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut editor = TextEditor::new();
    editor.readonly = args.readonly;

    // Load file if specified
    if let Some(ref path) = args.file {
        if path.exists() {
            editor.load_file(path)?;
        } else {
            editor.filename = Some(path.clone());
            editor.lines = vec![String::new()];
        }
    }

    // Set initial cursor position
    if let Some(line) = args.line {
        if line > 0 && line <= editor.lines.len() {
            editor.cursor_row = line - 1;
        }
    }

    println!("Rustux Text Editor v.0.1.0");
    println!("----------------------------");
    println!("Ctrl+O - Save | Ctrl+X - Quit");
    println!("Arrow keys - Move | Enter - New line");
    println!("Ctrl+Q - Force quit");
    println!();

    // Run the editor
    editor.run()?;

    Ok(())
}
