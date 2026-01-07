# Rustica OS GUI - Contributing Guidelines

**Version**: 1.0
**Last Updated**: 2025-01-07
**Status**: Final Specification

## Table of Contents

1. [Introduction](#introduction)
2. [Getting Started](#getting-started)
3. [Development Workflow](#development-workflow)
4. [Code Standards](#code-standards)
5. [Commit Guidelines](#commit-guidelines)
6. [Pull Request Process](#pull-request-process)
7. [Review Process](#review-process)
8. [Testing Requirements](#testing-requirements)
9. [Documentation Requirements](#documentation-requirements)
10. [Community Guidelines](#community-guidelines)

---

## Introduction

### Welcome to Rustica OS GUI

Thank you for your interest in contributing to Rustica OS GUI! This document provides guidelines and instructions for contributing to the project.

### Project Goals

Rustica OS GUI aims to provide:
- A modern, performant desktop environment for Rustux OS
- A user-friendly interface for both desktop and mobile devices
- A stable, secure, and accessible computing platform
- A welcoming community for contributors of all skill levels

### Types of Contributions

We welcome contributions in many forms:
- **Code**: Bug fixes, new features, performance improvements
- **Documentation**: Improvements to docs, guides, tutorials
- **Testing**: Test cases, test infrastructure
- **Design**: UI/UX improvements, visual design
- **Bug Reports**: Detailed issue reports
- **Feature Requests**: Well-thought-out feature proposals
- **Community**: Helping other users, answering questions

---

## Getting Started

### Prerequisites

**System Requirements**:
- Linux distribution (Ubuntu 22.04+, Arch, Fedora 37+, or Rustux OS)
- Rust 1.70 or later
- Git
- Wayland development libraries
- EGL/Vulkan drivers
- C compiler and build tools

**Install Dependencies**:

Ubuntu/Debian:
```bash
sudo apt update
sudo apt install -y \
  build-essential \
  pkg-config \
  libwayland-dev \
  libegl1-mesa-dev \
  libxkbcommon-dev \
  libinput-dev \
  libudev-dev \
  libdbus-1-dev \
  libpulse-dev \
  libsystemd-dev \
  git \
  curl
```

Arch:
```bash
sudo pacman -S \
  base-devel \
  wayland \
  mesa \
  libxkbcommon \
  libinput \
  dbus \
  pulseaudio \
  systemd \
  git
```

### Setup Development Environment

**1. Install Rust**:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

**2. Clone Repository**:
```bash
git clone https://github.com/rustux-os/rustica-gui.git
cd rustica-gui
```

**3. Install Development Tools**:
```bash
# Pre-commit hooks
cargo install cargo-hack
cargo install cargo-watch
cargo install typos  # Spelling checker

# Rust analyzer for your editor
rustup component add rust-analyzer
```

**4. Build Project**:
```bash
cargo build --release
```

**5. Run Tests**:
```bash
cargo test --all
```

### Development Workflow Overview

```
1. Fork repository
   ↓
2. Create branch from main
   ↓
3. Make changes
   ↓
4. Write tests
   ↓
5. Run tests and linting
   ↓
6. Commit with clear message
   ↓
7. Push to fork
   ↓
8. Create Pull Request
   ↓
9. Address review feedback
   ↓
10. Get approved and merged
```

---

## Development Workflow

### Branch Naming

Use descriptive branch names:
```bash
# Good
feature/workspace-overview
bugfix/panel-crash-on-startup
docs/installation-guide
refactor/rendering-pipeline

# Less clear
my-branch
fix-stuff
update
```

### Making Changes

**1. Update Your Branch**:
```bash
git fetch upstream
git rebase upstream/main
```

**2. Make Atomic Commits**:
Each commit should do one thing:
```bash
# Good: Separate commits
git commit -m "Add workspace overview panel"
git commit -m "Fix keyboard navigation in overview"
git commit -m "Add tests for workspace switching"

# Bad: One commit for everything
git commit -m "Add workspace feature and fix bugs"
```

**3. Test Your Changes**:
```bash
# Run all tests
cargo test --all

# Run specific test
cargo test --test workspace_tests

# Run with logging
RUST_LOG=debug cargo run --release
```

### Pre-Commit Checklist

Before committing:
- [ ] Code compiles without warnings (`cargo clippy -- -D warnings`)
- [ ] Tests pass (`cargo test --all`)
- [ ] Code formatted (`cargo fmt --all`)
- [ ] No typos (`typos`)
- [ ] Documentation updated (if needed)
- [ ] Tests added/updated (if applicable)
- [ ] Commit message follows guidelines

### Pre-Commit Hooks (Optional)

Install git hooks to automate checks:

`.git/hooks/pre-commit`:
```bash
#!/bin/sh
set -e

echo "Running pre-commit checks..."

# Format check
cargo fmt --all -- --check

# Clippy
cargo clippy --all-targets --all-features -- -D warnings

# Typos
typos

# Tests (quick only)
cargo test --all --no-fail-fast --quiet

echo "All checks passed!"
```

Enable:
```bash
chmod +x .git/hooks/pre-commit
```

---

## Code Standards

### Rust Code Style

**Follow Standard Rust Style**:
```bash
# Auto-format your code
cargo fmt --all
```

**Use Meaningful Names**:
```rust
// Good
pub fn create_window_with_dimensions(width: u32, height: u32) -> Result<Window> {
    // ...
}

// Bad
pub fn cw(w: u32, h: u32) -> Res<Win> {
    // ...
}
```

**Document Public APIs**:
```rust
/// Creates a new window with the specified dimensions.
///
/// # Arguments
///
/// * `width` - The width of the window in pixels
/// * `height` - The height of the window in pixels
///
/// # Returns
///
/// Returns a `Result` containing the `Window` handle or an error.
///
/// # Examples
///
/// ```rust
/// let window = create_window_with_dimensions(800, 600)?;
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The dimensions exceed maximum size
/// - Memory allocation fails
pub fn create_window_with_dimensions(width: u32, height: u32) -> Result<Window> {
    // ...
}
```

**Error Handling**:
```rust
// Use Result for recoverable errors
pub fn open_file(path: &Path) -> Result<File, io::Error> {
    File::open(path)
}

// Use Option for optional values
pub fn get_active_window() -> Option<Window> {
    // ...
}

// Use custom error types for libraries
#[derive(Debug, thiserror::Error)]
pub enum CompositorError {
    #[error("Wayland display failed: {0}")]
    Wayland(String),

    #[error("GPU initialization failed: {0}")]
    GPU(String),

    #[error("IO error: {0}")]
    IO(#[from] io::Error),
}
```

**Use Type System**:
```rust
// Good: Newtype for type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowId(u32);

impl WindowId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

// Bad: Use raw u32 everywhere
pub fn get_window(id: u32) -> Option<&Window> {
    // Which u32 is this? WindowId? SurfaceId?
}
```

### Performance Guidelines

**Avoid Allocations in Hot Paths**:
```rust
// Bad: Allocates every frame
pub fn render(&mut self) {
    let vertices = vec![/* ... */];  // Allocates!
    self.gpu.draw_vertices(&vertices);
}

// Good: Reuse buffer
pub struct Renderer {
    vertex_buffer: Vec<Vertex>,
}

impl Renderer {
    pub fn render(&mut self) {
        self.vertex_buffer.clear();
        self.vertex_buffer.extend_from_slice(&[/* ... */]);
        self.gpu.draw_vertices(&self.vertex_buffer);
    }
}
```

**Use Appropriate Collections**:
```rust
// Fast lookup: HashMap
let windows: HashMap<WindowId, Window> = HashMap::new();

// Ordered iteration: BTreeMap
let sorted: BTreeMap<String, Window> = BTreeMap::new();

// Small collection: Array
const MAX_LAYERS: usize = 4;
let layers: [Layer; MAX_LAYERS] = [/* ... */];
```

### Security Guidelines

**Validate Input**:
```rust
pub fn set_window_size(&mut self, width: u32, height: u32) -> Result<()> {
    const MAX_SIZE: u32 = 16384;

    if width > MAX_SIZE || height > MAX_SIZE {
        return Err(CompositorError::InvalidSize);
    }

    if width == 0 || height == 0 {
        return Err(CompositorError::InvalidSize);
    }

    self.width = width;
    self.height = height;
    Ok(())
}
```

**Prevent Integer Overflow**:
```rust
// Use checked arithmetic
pub fn allocate_buffer(size: usize) -> Result<*mut u8> {
    let total = size.checked_mul(4)
        .ok_or(Error::AllocationTooLarge)?;

    // ...
}

// Or use saturating arithmetic for display values
pub fn clamp_value(value: u32) -> u32 {
    value.saturating_add(10).min(MAX_VALUE)
}
```

---

## Commit Guidelines

### Commit Message Format

Follow conventional commits:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Test changes
- `chore`: Maintenance tasks
- `revert`: Revert a commit

**Examples**:
```
feat(panel): add workspace overview button

Implement a new button in the panel that opens the workspace
overview when clicked. The button is positioned between the
Activities button and the app menu.

Closes #1234
```

```
fix(compositor): resolve crash on monitor hotplug

The compositor would crash when a monitor was unplugged due to
a null pointer dereference in the output destruction code.

Fixes #5678
```

```
docs(installation): update dependencies for Ubuntu 24.04

Update the installation guide to include the new package names
in Ubuntu 24.04.
```

### Commit Message Best Practices

**DO**:
- Use imperative mood ("Add feature", not "Added feature")
- Keep first line under 50 characters
- Explain why, not just what
- Reference issues with "Fixes #123" or "Closes #456"
- Keep body wrapped at 72 characters

**DON'T**:
- Use "and" to combine unrelated changes
- Be vague ("Update code", "Fix stuff")
- Mix multiple changes in one commit
- Write novels (keep it concise)

---

## Pull Request Process

### Creating a Pull Request

**1. Prepare Your Branch**:
```bash
# Ensure it's up to date
git fetch upstream
git rebase upstream/main

# Push to your fork
git push origin feature/my-feature
```

**2. Create Pull Request**:
- Go to GitHub
- Click "New Pull Request"
- Select your branch
- Fill out the PR template

**3. PR Template**:
```markdown
## Description
Brief description of the changes.

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Performance improvement
- [ ] Documentation update
- [ ] Refactoring

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing completed

## Checklist
- [ ] Code follows style guidelines
- [ ] Documentation updated
- [ ] Tests added/updated
- [ ] No warnings in `cargo clippy`
- [ ] Commit messages follow guidelines

## Related Issues
Fixes #1234
Related to #5678
```

### Pull Request Guidelines

**Keep PRs Focused**:
- One PR should address one issue or feature
- If you have multiple unrelated changes, make multiple PRs
- Large PRs are harder to review and more likely to have issues

**Small PRs are Better**:
- Aim for <400 lines of code changes
- Split large features into smaller, reviewable chunks
- Easier to review, test, and merge

**Update PRs**:
```bash
# Add new commits to your branch
git commit -m "feat: add missing feature"

# Rebase onto latest main
git fetch upstream
git rebase upstream/main

# Force push to update PR
git push origin feature/my-feature --force
```

---

## Review Process

### Reviewer Responsibilities

**Reviewers Should**:
- Review PRs within 48 hours
- Provide clear, constructive feedback
- Explain why changes are requested
- Approve when all requirements are met

**Review Criteria**:
1. **Correctness**: Does the code do what it claims?
2. **Design**: Is the approach sound?
3. **Style**: Does it follow code standards?
4. **Testing**: Are there adequate tests?
5. **Documentation**: Is the code documented?
6. **Performance**: Are there performance concerns?

### Author Responsibilities

**Authors Should**:
- Address all review comments
- Push updated commits to the branch
- Respond to every comment
- Mark conversations as resolved when addressed

**Response Types**:
```markdown
# Fixing a comment
"Fixed in abc123 - changed to use Arc as suggested."

# Explaining a decision
"Left as-is because X - the overhead of Arc isn't justified
for this use case where we know the lifetime is bounded."

# Asking for clarification
"Could you elaborate on why you think we need Arc here?"
```

### Approval Requirements

**Before Merging**:
- At least one approval from maintainer
- All CI checks pass
- No unresolved conversations
- At least 24 hours since submission (for non-trivial changes)

**Fast-Track Cases**:
- Typo fixes
- Documentation improvements
- Test additions
- Obvious bug fixes (with tests)

---

## Testing Requirements

### Test Coverage

**Minimum Coverage**:
- New code: 80% coverage
- Critical paths: 90% coverage
- Overall: 70% coverage

**Check Coverage**:
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html
```

### Writing Tests

**Unit Tests**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_creation() {
        let window = Window::new(800, 600).unwrap();
        assert_eq!(window.width(), 800);
        assert_eq!(window.height(), 600);
    }

    #[test]
    fn test_invalid_size_rejected() {
        let result = Window::new(0, 600);
        assert!(matches!(result, Err(CompositorError::InvalidSize)));
    }
}
```

**Integration Tests**:
```rust
// tests/compositor_integration.rs
use rustica_comp::Compositor;

#[test]
fn test_full_render_cycle() {
    let compositor = Compositor::new().unwrap();
    let surface = compositor.create_surface().unwrap();
    compositor.commit_surface(&surface).unwrap();
    compositor.render_frame().unwrap();
    assert!(compositor.surface_is_visible(&surface));
}
```

**Property-Based Tests**:
```rust
#[cfg(test)]
mod proptests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_window_size_clamping(width in 0u32..100000) {
            let window = Window::new(width, 600).unwrap();
            assert!(window.width() <= MAX_WIDTH);
        }
    }
}
```

### Running Tests

**All Tests**:
```bash
cargo test --all
```

**Specific Test**:
```bash
cargo test test_window_creation
```

**With Output**:
```bash
cargo test -- --nocapture
```

**Release Mode**:
```bash
cargo test --release
```

---

## Documentation Requirements

### Code Documentation

**Public APIs Must Be Documented**:
```rust
/// Manages the lifecycle and rendering of Wayland surfaces.
///
/// The [`Compositor`] is the core component that handles all window
/// management, input routing, and display rendering.
///
/// # Architecture
///
/// The compositor is organized into several subsystems:
/// - Display: Wayland protocol handling
/// - Render: GPU-accelerated rendering
/// - Input: Device and event handling
///
/// # Examples
///
/// ```rust
/// let compositor = Compositor::new()?;
/// compositor.run()?;
/// ```
pub struct Compositor {
    // ...
}
```

### Documentation Comments

**Use Proper Markdown**:
```rust
/// # Example
///
/// ```rust
/// let window = Window::new(800, 600)?;
/// ```
///
/// # Panics
///
/// This function will panic if the GPU is not available.
///
/// # Errors
///
/// Returns an error if:
/// - The window size is invalid
/// - GPU initialization fails
///
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer.
pub unsafe fn create_window_raw(ptr: *mut Window) -> Result<Window> {
    // ...
}
```

### Documentation Tests

**Examples in Docs Are Tests**:
```rust
/// Adds a number to another number.
///
/// # Examples
///
/// ```
/// use mylib::add;
///
/// let result = add(2, 3);
/// assert_eq!(result, 5);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

**Run Doc Tests**:
```bash
cargo test --doc
```

### User Documentation

**Update User Guides**:
- Installation guides for new dependencies
- Feature documentation for new features
- Troubleshooting for known issues

**Update Developer Docs**:
- Architecture diagrams for new components
- API documentation for new APIs
- Contribution guides for new workflows

---

## Community Guidelines

### Code of Conduct

**Our Pledge**:
- Be inclusive and respectful
- Welcome diverse perspectives
- Focus on what's best for the community
- Show empathy toward other community members

**Standards**:
- Use welcoming and inclusive language
- Respect differing viewpoints and experiences
- Gracefully accept constructive criticism
- Focus on what's best for the community

**Unacceptable Behavior**:
- Harassment, trolling, or derogatory comments
- Personal or political attacks
- Public or private harassment
- Publishing private information

**Reporting Issues**:
Contact: conduct@rustux-os.org

### Communication Channels

**GitHub**:
- Issues: Bug reports, feature requests
- PRs: Code contributions
- Discussions: General questions

**Discord**:
- #general: Chat and questions
- #dev: Development discussion
- #reviews: PR reviews
- #help: Getting help

**Mailing List**:
- dev@rustux-os.org: Development discussions
- announce@rustux-os.org: Announcements

### Getting Help

**Before Asking**:
1. Search existing issues and discussions
2. Read the documentation
3. Try to debug the issue yourself

**When Asking**:
1. Describe what you're trying to do
2. Show what you've tried
3. Include error messages
4. Provide minimal reproduction case

**Good Question Example**:
```markdown
## Issue
I'm trying to create a custom Wayland surface but getting
a protocol error.

## What I've Tried
I've read the Wayland protocol docs and looked at the
examples in the repo.

## Code
```rust
let surface = compositor.create_surface()?;
surface.commit();  // This panics with protocol error
```

## Error
```
thread 'main' panicked at 'Protocol error: invalid surface state'
```

## Environment
- Rust 1.75
- Ubuntu 24.04
- Wayland 1.22
```

### Recognition

**Contributors Are Recognized**:
- Contributors list in README
- Release notes for significant contributions
- Annual contributor appreciation post

**Becoming a Maintainer**:
- Consistent, quality contributions
- Good review skills
- Understanding of the codebase
- Community trust and respect

---

## Appendix

### Useful Commands

**Development**:
```bash
# Watch for changes and recompile
cargo watch -x check -x test -x run

# Check for unused dependencies
cargo +nightly udeps

# Update dependencies
cargo update

# Check for security vulnerabilities
cargo audit
```

**Git**:
```bash
# View commit history
git log --oneline --graph

# Interactive rebase
git rebase -i HEAD~3

# Cherry-pick commit
git cherry-pick abc123

# Stash changes
git stash push -m "Work in progress"
```

**Testing**:
```bash
# Run tests in parallel
cargo test --release -- --test-threads=4

# Run only integration tests
cargo test --test '*'

# Generate HTML coverage
cargo tarpaulin --out Html --output-dir coverage
```

### Resources

**Learning Rust**:
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rustlings](https://github.com/rust-lang/rustlings)

**Wayland Resources**:
- [Wayland Protocol](https://wayland.freedesktop.org/docs/html/)
- [Wayland Book](https://wayland-book.com/)
- [Smithay](https://github.com/Smithay/smithay) (Wayland lib in Rust)

**Project Resources**:
- Architecture docs: `/docs/architecture/`
- API docs: `/docs/api.md`
- User docs: `/docs/user.md`

### Contact

- Project Lead: dev@rustux-os.org
- Security: security@rustux-os.org
- Website: https://rustux-os.org

---

**Thank you for contributing to Rustica OS GUI!**

Every contribution, no matter how small, helps make the project better for everyone. We appreciate your time and effort in improving Rustica OS.
