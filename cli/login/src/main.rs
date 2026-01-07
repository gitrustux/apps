// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! login - User login utility
//!
//! Authenticates users and starts login sessions.

use anyhow::{Context, Result};
use clap::Parser;
use rutils::{authenticate_user, get_home_dir, switch_user};
use std::ffi::CString;
use std::fs::OpenOptions;
use std::io::{self, BufRead, Read, Write};
use std::os::fd::BorrowedFd;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::time::SystemTime;

const SPLASH_SCREEN: &str = r#"
──────────────────────────────────────────────────────────────────────────────────────────────────────────
─████████████████───██████──██████─██████████████─██████████████─██████████─██████████████─██████████████─
─██░░░░░░░░░░░░██───██░░██──██░░██─██░░░░░░░░░░██─██░░░░░░░░░░██─██░░░░░░██─██░░░░░░░░░░██─██░░░░░░░░░░██─
─██░░████████░░██───██░░██──██░░██─██░░██████████─██████░░██████─████░░████─██░░██████████─██░░██████░░██─
─██░░██────██░░██───██░░██──██░░██─██░░██─────────────██░░██───────██░░██───██░░██─────────██░░██──██░░██─
─██░░████████░░██───██░░██──██░░██─██░░██████████─────██░░██───────██░░██───██░░██─────────██░░██████░░██─
─██░░░░░░░░░░░░██───██░░██──██░░██─██░░░░░░░░░░██─────██░░██───────██░░██───██░░██─────────██░░░░░░░░░░██─
─██░░██████░░████───██░░██──██░░██─██████████░░██─────██░░██───────██░░██───██░░██─────────██░░██████░░██─
─██░░██──██░░██─────██░░██──██░░██─────────██░░██─────██░░██───────██░░██───██░░██─────────██░░██──██░░██─
─██░░██──██░░██████─██░░██████░░██─██████████░░██─────██░░██─────████░░████─██░░██████████─██░░██──██░░██─
─██░░██──██░░░░░░██─██░░░░░░░░░░██─██░░░░░░░░░░██─────██░░██─────██░░░░░░██─██░░░░░░░░░░██─██░░██──██░░██─
─██████──██████████─██████████████─██████████████─────██████─────██████████─██████████████─██████──██████─
──────────────────────────────────────────────────────────────────────────────────────────────────────────
Operating System version: v.0.0.1
Rustux Kernel version: v.0.0.1
Visit: http://rustux.com
"#;

/// User login utility
#[derive(Parser, Debug)]
#[command(name = "login")]
#[command(about = "User login utility", long_about = None)]
struct Args {
    /// Username
    #[arg(short, long)]
    user: Option<String>,

    /// Skip login prompt (used by init)
    #[arg(long)]
    direct: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Display splash screen on fresh TTY
    display_splash()?;

    // Get username
    let username = if let Some(user) = args.user {
        user
    } else {
        prompt_username()?
    };

    // Authenticate
    if !args.direct {
        authenticate(&username)?;
    }

    // Start session
    start_session(&username)?;

    Ok(())
}

fn display_splash() -> Result<()> {
    // Check if we're on a TTY
    if !atty::is(atty::Stream::Stdout) {
        return Ok(());
    }

    // Clear screen
    print!("\x1b[2J\x1b[H");
    io::stdout().flush()?;

    // Display splash screen
    println!("{}", SPLASH_SCREEN);

    Ok(())
}

fn prompt_username() -> Result<String> {
    let mut username = String::new();

    loop {
        print!("login: ");
        io::stdout().flush()?;

        username.clear();
        io::stdin().read_line(&mut username)?;

        let username = username.trim();

        if !username.is_empty() {
            return Ok(username.to_string());
        }
    }
}

fn authenticate(username: &str) -> Result<()> {
    // Disable echo
    let stdout = io::stdout();
    let _stdout = stdout.lock();
    let stdin = io::stdin();
    let mut stdin_locked = stdin.lock();

    // Use termios for password handling
    use nix::sys::termios::{tcgetattr, tcsetattr, SetArg, LocalFlags};

    let fd = io::stdin().as_raw_fd();
    let borrowed_fd = unsafe { BorrowedFd::borrow_raw(fd) };
    let mut termios = tcgetattr(&borrowed_fd)?;

    // Save original settings
    let original = termios.clone();

    // Disable echo
    termios.local_flags.remove(LocalFlags::ECHO);
    tcsetattr(&borrowed_fd, SetArg::TCSANOW, &termios)?;

    print!("Password: ");
    io::stdout().flush()?;

    let mut password = String::new();
    stdin_locked.read_line(&mut password)?;
    println!();

    // Restore terminal
    tcsetattr(&borrowed_fd, SetArg::TCSANOW, &original)?;

    let password = password.trim();

    // Authenticate
    match authenticate_user(username, password) {
        Ok(true) => Ok(()),
        Ok(false) => {
            eprintln!("Login incorrect");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Authentication error: {}", e);
            std::process::exit(1);
        }
    }
}

fn start_session(username: &str) -> Result<()> {
    use nix::unistd::{setgid, setuid, Gid, Uid};
    use std::env;

    // Get user info
    let user = nix::unistd::User::from_name(username)
        .context("failed to lookup user")?
        .ok_or_else(|| anyhow::anyhow!("user not found: {}", username))?;

    // Get home directory
    let home = user.dir.to_string_lossy().to_string();

    // Set environment variables
    env::set_var("HOME", &home);
    env::set_var("USER", username);
    env::set_var("LOGNAME", username);
    env::set_var("SHELL", &user.shell);
    env::set_var("PATH", "/usr/local/bin:/usr/bin:/bin");
    env::set_var("TERM", "xterm-256color");
    env::set_var("LANG", "en_US.UTF-8");

    // Change to home directory
    std::fs::create_dir_all(&home)?;
    env::set_current_dir(&home)?;

    // Log the login
    log_login(username)?;

    // Switch to user
    setgid(Gid::from_raw(u32::from(user.gid))).context("failed to set group ID")?;
    setuid(Uid::from_raw(u32::from(user.uid))).context("failed to set user ID")?;

    // Start the shell
    let shell = user.shell.to_string_lossy().into_owned();

    println!();
    println!("Welcome to Rustica v.0.0.1!");
    println!("Last login: {}", get_last_login(username));
    println!();

    // Execute shell
    use std::ffi::OsString;
    let shell_path = Path::new(&shell);
    let shell_cstring = CString::new(shell.clone())?;

    // Get shell name (last component)
    let shell_name = shell_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("-sh");

    let shell_name_cstring = CString::new(format!("-{}", shell_name))?;

    unsafe {
        // Prepare argv
        let argv: Vec<*const libc::c_char> = vec![
            shell_name_cstring.as_ptr(),
            std::ptr::null(),
        ];

        libc::execvp(
            shell_cstring.as_ptr(),
            argv.as_ptr(),
        );
    }

    Err(anyhow::anyhow!("execvp failed for shell: {}", shell))
}

fn log_login(username: &str) -> Result<()> {
    let _lastlog_file = "/var/log/lastlog";

    // Log to wtmp
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::io::AsRawFd;

        if let Ok(mut file) = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open("/var/log/wtmp")
        {
            // Write utmp entry (simplified - real implementation would use libc utmp struct)
            let _now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;

            // In production, would write proper utmp struct
            let _ = file.write_all(format!("LOGIN:{}\n", username).as_bytes());
        }
    }

    Ok(())
}

fn get_last_login(username: &str) -> String {
    // In production, would read from lastlog database
    format!("Never (first login for {})", username)
}

// Helper for TTY detection
mod atty {
    use std::os::unix::io::AsRawFd;

    pub fn is(stream: Stream) -> bool {
        let fd = match stream {
            Stream::Stdin => std::io::stdin().as_raw_fd(),
            Stream::Stdout => std::io::stdout().as_raw_fd(),
            Stream::Stderr => std::io::stderr().as_raw_fd(),
        };

        unsafe {
            let mut termios: libc::termios = std::mem::zeroed();
            libc::isatty(fd) != 0 && libc::tcgetattr(fd, &mut termios) == 0
        }
    }

    pub enum Stream {
        Stdin,
        Stdout,
        Stderr,
    }
}
