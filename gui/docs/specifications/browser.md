# Rustica OS GUI - Web Browser Specification

**Component**: rustica-web (WebKit-based Browser)
**Version**: 1.0
**Last Updated**: 2025-01-07
**Status**: Final Specification

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [WebKitGTK Integration](#webkitgtk-integration)
4. [Wayland Integration](#wayland-integration)
5. [Security Model](#security-model)
6. [User Interface](#user-interface)
7. [Features](#features)
8. [Performance](#performance)
9. [Privacy](#privacy)
10. [Extension System](#extension-system)
11. [Sync Services](#sync-services)
12. [Build System](#build-system)

---

## Overview

### Purpose

The Rustica Web Browser (`rustica-web`) is a modern, secure, and privacy-focused web browser built on WebKitGTK and designed specifically for the Rustica OS desktop environment. It provides a native Wayland experience with seamless integration into the Rustica Shell.

### Design Goals

1. **Native Wayland Integration**: First-class Wayland citizen with no X11 dependencies
2. **Security First**: Sandboxed tabs, site isolation, secure defaults
3. **Privacy Focused**: No tracking, no telemetry, user data protection
4. **Performance**: Fast rendering, efficient memory usage, hardware acceleration
5. **Integration**: Deep integration with Rustica OS settings, notifications, and services
6. **Extensibility**: Support for extensions and themes

### Technology Stack

- **Rendering Engine**: WebKitGTK 2.50+
- **Graphics**: Wayland + EGL/Vulkan for GPU acceleration
- **UI Toolkit**: GTK4 (for WebKitGTK integration) with custom Rustica theming
- **Language**: C++ (WebKit core) + Rust (browser shell and extensions)
- **Sandboxing**: Landlock + bubblewrap for tab isolation
- **Networking**: libsoup with custom privacy enhancements

---

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Rustica Web Browser UI                       │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐   │
│  │  Tab Bar    │ │ Navigation  │ │   Web Content View      │   │
│  │  (GTK4)     │ │  Bar        │ │   (WebKitWebView)       │   │
│  └─────────────┘ └─────────────┘ └─────────────────────────┘   │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐   │
│  │ Bookmarks   │ │ Downloads   │ │   Developer Tools       │   │
│  │  Bar        │ │  Panel      │ │   (WebKitInspector)     │   │
│  └─────────────┘ └─────────────┘ └─────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Browser Shell (Rust)                           │
│  - Tab Management                                              │
│  - Session Management                                          │
│  - Extension Management                                        │
│  - D-Bus Integration (org.rustica.Browser)                     │
│  - Portal Integration (screenshots, file picker, etc.)         │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                   WebKitGTK Engine                              │
│  - WebCore (rendering, DOM, CSS)                               │
│  - JavaScriptCore (JS engine)                                  │
│  - Network Process (HTTP, WebSocket)                           │
│  - Storage Process (IndexedDB, Cache)                          │
│  - GPU Process (compositing, 3D)                               │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Multi-Process Architecture                     │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐          │
│  │   UI     │ │  Web     │ │ Network  │ │   GPU    │          │
│  │ Process  │ │ Process  │ │ Process  │ │ Process  │          │
│  │ (Sandbox)│ │ (Sandbox)│ │ (Sandbox)│ │ (Sandbox)│          │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘          │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                   System Integration                           │
│  - Wayland (display, input)                                    │
│  - PipeWire (audio/video capture)                              │
│  - D-Bus (system services)                                     │
│  - Portals (file picker, screenshot, etc.)                     │
└─────────────────────────────────────────────────────────────────┘
```

### Process Model

WebKitGTK uses a multi-process architecture for security and stability:

1. **UI Process**: Main browser process (Rust shell + GTK4 UI)
2. **Web Process**: Per-tab or per-site-isolated rendering processes
3. **Network Process**: Handles all network requests (single shared process)
4. **GPU Process**: Handles graphics and compositing (single shared process)

### IPC Architecture

```
UI Process (Rust) ──D-Bus──▶ Shell Services
        │                           │
        └────WebKitGTK IPC─────┬───┴──► Portals
                                │
                ┌───────────────┼───────────────┐
                ▼               ▼               ▼
            Web Process    Network Process   GPU Process
            (per-tab)      (shared)         (shared)
```

---

## WebKitGTK Integration

### Building WebKitGTK for Rustica

**Build Configuration**:
```cmake
# WebKitGTK CMake configuration
cmake -DPORT=GTK \
      -DCMAKE_BUILD_TYPE=Release \
      -DENABLE_ACCELERATED_2D_CANVAS=ON \
      -DENABLE_WAYLAND_TARGET=ON \
      -DENABLE_X11_TARGET=OFF \
      -DENABLE_MINIBROWSER=ON \
      -DENABLE_WEB_AUDIO=ON \
      -DENABLE_VIDEO=ON \
      -DENABLE_MEDIA_STREAM=ON \
      -DENABLE_SPELLCHECK=ON \
      -DGAMEPAD=OFF \
      -DUSE_LIBHYPHEN=ON \
      -DUSE_SOUP2=OFF \
      -DUSE_WPE_RENDERER=OFF
```

**Rust Wrapper**:
```rust
// src/webkit/mod.rs
use gtk::{glib, Application, ApplicationWindow};
use webkit2gtk::{WebView, WebViewExt, SettingsExt};

pub struct BrowserTab {
    webview: WebView,
    title: String,
    url: String,
}

impl BrowserTab {
    pub fn new(parent: &ApplicationWindow) -> Self {
        let webview = WebView::builder()
            .build();

        // Configure settings
        let settings = webview.settings().unwrap();
        settings.set_enable_javascript(true);
        settings.set_enable_developer_extras(true);
        settings.set_hardware_acceleration_policy(
            webkit2gtk::HardwareAccelerationPolicy::Always
        );

        BrowserTab {
            webview,
            title: String::new(),
            url: String::new(),
        }
    }

    pub fn load_url(&self, url: &str) {
        self.webview.load_uri(url);
    }

    pub fn webview(&self) -> &WebView {
        &self.webview
    }
}
```

### Initialization

```rust
use gtk::prelude::*;
use webkit2gtk::*;

fn main() {
    // Create GTK application
    let app = gtk::Application::builder()
        .application_id("org.rustica.WebBrowser")
        .build();

    app.connect_activate(|app| {
        build_ui(app);
    });

    app.run();
}

fn build_ui(app: &gtk::Application) {
    // Main window
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Rustica Web")
        .default_width(1280)
        .default_height(720)
        .build();

    // Create web view
    let webview = WebView::new();

    // Load homepage
    webview.load_uri("https://start.rustica-os.org");

    // Add to window
    window.set_child(Some(&webview));
    window.present();
}
```

---

## Wayland Integration

### Native Wayland Support

WebKitGTK has full Wayland support. We disable X11 completely:

```cmake
# In CMakeLists.txt
set(ENABLE_WAYLAND_TARGET ON)
set(ENABLE_X11_TARGET OFF)
```

**Display Configuration**:
```rust
// Ensure Wayland backend is used
use gtk::gdk::Display;

if let Some(display) = Display::default() {
    // Verify we're on Wayland
    if !display.backend().is_wayland() {
        eprintln!("Rustica Web requires Wayland");
        std::process::exit(1);
    }
}
```

### Input Handling

**Touch and Gesture Support**:
```rust
use gtk::GestureZoom;

fn setup_gestures(webview: &WebView) {
    // Pinch-to-zoom
    let zoom_gesture = GestureZoom::builder()
        .propagation_phase(gtk::PropagationPhase::Bubble)
        .build();

    zoom_gesture.connect_scale_changed(|gesture, scale| {
        // Apply zoom to web view
        // ...
    });

    webview.add_controller(&zoom_gesture);
}
```

**Touch Events**:
```rust
use gtk::GestureClick;

fn setup_touch_events(webview: &WebView) {
    let touch_gesture = GestureClick::builder()
        .button(0)  // Any button
        .build();

    touch_gesture.connect_pressed(|gesture, n_press, x, y| {
        // Handle touch/click
        // ...
    });

    webview.add_controller(&touch_gesture);
}
```

### Hi-DPI Support

```rust
// Configure for Hi-DPI displays
use webkit2gtk::SettingsExt;

let settings = webview.settings().unwrap();

// Enable device pixel ratio support
settings.set_enable_write_console_messages_to_stdout(true);

// Set zoom level based on display scale
let display = window.display().unwrap();
let scale = display.primary_monitor().unwrap().scale_factor();

// Apply to web view
// Note: WebKit handles this automatically via Wayland
```

---

## Security Model

### Sandboxing

**Per-Process Sandboxing with Landlock**:

```c
// Source/WebKit/UIProcess/Launcher/glib/BrowserLauncherGtk.cpp
#include <linux/landlock.h>
#include <sys/syscall.h>

void applyLandlockSandbox() {
    struct landlock_ruleset_attr attr = {
        .handled_access_fs = LANDLOCK_ACCESS_FS_READ_FILE |
                            LANDLOCK_ACCESS_FS_READ_DIR |
                            LANDLOCK_ACCESS_FS_WRITE_FILE |
                            LANDLOCK_ACCESS_FS_REMOVE_DIR |
                            LANDLOCK_ACCESS_FS_REMOVE_FILE |
                            LANDLOCK_ACCESS_FS_MAKE_CHAR |
                            LANDLOCK_ACCESS_FS_MAKE_DIR |
                            LANDLOCK_ACCESS_FS_MAKE_REG |
                            LANDLOCK_ACCESS_FS_MAKE_SOCK |
                            LANDLOCK_ACCESS_FS_EXECUTE,
    };

    int ruleset_fd = syscall(SYS_landlock_create_ruleset,
                            &attr, sizeof(attr), 0);

    // Add rules for allowed paths
    // /tmp, /proc, /sys (read-only)
    // ~/.cache, ~/.config (read-write)

    // Apply to current thread
    syscall(SYS_landlock_restrict_self, ruleset_fd, 0);
}
```

**Bubblewrap Integration**:
```bash
# Launch web processes with bubblewrap
bwrap \
  --ro-bind /usr /usr \
  --ro-bind /etc /etc \
  --dev /dev \
  --proc /proc \
  --tmpfs /tmp \
  --dir ~/.cache/webkit \
  --dir ~/.config/webkit \
  --unshare-all \
  --share-net \
  --setenv WEBKIT_SANDBOXED 1 \
  /usr/libexec/webkitwebkit-web-process
```

### Site Isolation

```rust
// Enable site isolation for security
use webkit2gtk::WebsiteDataManagerExt;

let data_manager = WebsiteDataManager::builder()
    .build();

// Enable per-site process isolation
data_manager.set_isolated_session(true);

// Configure memory limits per site
let website_data_store = WebsiteDataStore::builder()
    .website_data_manager(&data_manager)
    .build();
```

### Content Security

**Default Security Policies**:
```rust
use webkit2gtk::WebContextExt;

let context = WebContext::default();

// Enable strict mixed content mode
context.set_tls_errors_policy(webkit2gtk::TLSErrorsPolicy::Fail);

// Disable insecure features by default
let settings = WebView::settings_ext();
settings.set_enable_javascript(false);  // Enable per-site
settings.set_enable_plugins(false);
settings.set_enable_webgl(false);  // Enable per-site

// Set up content security policy
let user_content = UserContentManager::new();
user_content.add_style(
    "default-security",
    "* { -webkit-touch-callout: none; }"  // Disable callouts
);
```

**Certificate Handling**:
```rust
use webkit2gtk::SecurityManagerExt;

let security_manager = context.security_manager().unwrap();

// Enable certificate pinning for important sites
security_manager.add_certificate_pinning_rule(
    "rustica-os.org",
    &certificate_pem
);

// Enable HSTS preload
security_manager.enable_hsts_preload();
```

### Permission Management

```rust
use webkit2gtk::PermissionRequestExt;

// Handler for permission requests
fn handle_permission_request(request: &PermissionRequest) {
    match request.request_type() {
        PermissionType::Geolocation => {
            // Show permission prompt
            show_permission_dialog(
                "Allow this site to access your location?",
                request
            );
        }
        PermissionType::UserMedia => {
            // Check if we're on HTTPS
            if !request.is_secure_context() {
                request.deny();
                return;
            }

            // Show camera/mic permission prompt
            show_media_permission_dialog(request);
        }
        PermissionType::Notifications => {
            // Check if notifications are enabled in settings
            if rustica_settings::notifications_allowed() {
                request.allow();
            } else {
                request.deny();
            }
        }
        _ => request.deny(),
    }
}
```

### Network Security

**Privacy Enhancements**:
```rust
use soup::{Session, SessionExt};

// Configure HTTP session
let session = Session::new();

// Enable HTTPS-only mode
session.set_https_only_mode(true);

// Disable tracking
session.set_accept_language_auto(false);
session.set_accept_language_auto(false);

// Block known trackers
let adblock_filter = AdblockFilter::new(
    "/usr/share/rustica-web/easylist.txt"
);
session.add_request_filter(&adblock_filter);

// Set user agent (privacy-preserving)
session.set_user_agent(
    "Mozilla/5.0 (X11; Linux x86_64; RusticaOS) \
     AppleWebKit/605.1.15 (KHTML, like Gecko) \
     Version/15.0 Safari/605.1.15"
);
```

---

## User Interface

### Main Browser Window

```
┌────────────────────────────────────────────────────────────┐
│  [≡] [←] [→] [↻]  🔍 Search or enter website...    [⋮]    │  ← Navigation Bar
├────────────────────────────────────────────────────────────┤
│  [Tab 1] [Tab 2] [Tab 3] [+]                        [⊞]    │  ← Tab Bar
├────────────────────────────────────────────────────────────┤
│                                                            │
│                                                            │
│                     Web Content                            │
│                     (WebKitWebView)                        │
│                                                            │
│                                                            │
└────────────────────────────────────────────────────────────┘
│  [📁] [⬇] [🔖] [≡]                                        │  ← Status Bar
└────────────────────────────────────────────────────────────┘
```

### Navigation Bar

```rust
use gtk::{Box, Button, Entry, Orientation};

fn create_navigation_bar(webview: &WebView) -> Box {
    let nav_bar = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .margin_start(6)
        .margin_end(6)
        .margin_top(6)
        .margin_bottom(6)
        .build();

    // Menu button
    let menu_btn = Button::from_icon_name("open-menu-symbolic");
    nav_bar.append(&menu_btn);

    // Back button
    let back_btn = Button::from_icon_name("go-previous-symbolic");
    back_btn.connect_clicked(clone!(@weak webview => move |_| {
        webview.go_back();
    }));
    nav_bar.append(&back_btn);

    // Forward button
    let fwd_btn = Button::from_icon_name("go-next-symbolic");
    fwd_btn.connect_clicked(clone!(@weak webview => move |_| {
        webview.go_forward();
    }));
    nav_bar.append(&fwd_btn);

    // Refresh button
    let refresh_btn = Button::from_icon_name("view-refresh-symbolic");
    refresh_btn.connect_clicked(clone!(@weak webview => move |_| {
        webview.reload();
    }));
    nav_bar.append(&refresh_btn);

    // URL bar
    let url_entry = Entry::builder()
        .placeholder_text("Search or enter website")
        .hexpand(true)
        .build();

    url_entry.connect_activate(clone!(@weak webview => move |entry| {
        let text = entry.text().to_string();
        if looks_like_url(&text) {
            webview.load_uri(&text);
        } else {
            // Search engine query
            let search_url = format!(
                "https://search.rustica-os.org/search?q={}",
                urlencoding::encode(&text)
            );
            webview.load_uri(&search_url);
        }
    }));

    nav_bar.append(&url_entry);

    // Menu button (right side)
    let menu_btn2 = Button::from_icon_name("view-more-symbolic");
    nav_bar.append(&menu_btn2);

    nav_bar
}
```

### Tab Bar

```rust
use gtk::Notebook;

fn create_tab_bar() -> Notebook {
    let tabs = Notebook::builder()
        .scrollable(true)
        .show_border(false)
        .build();

    // New tab button
    let new_tab_btn = Button::from_icon_name("tab-new-symbolic");
    new_tab_btn.connect_clicked(clone!(@weak tabs => move |_| {
        add_new_tab(&tabs);
    }));

    tabs
}

fn add_new_tab(tabs: &Notebook) {
    // Create new web view
    let webview = WebView::new();

    // Create tab label
    let tab_label = gtk::Label::new(Some("New Tab"));

    // Create close button
    let close_btn = Button::from_icon_name("window-close-symbolic");
    close_btn.connect_clicked(clone!(@weak tabs, @weak webview => move |_| {
        // Remove tab
        // ...
    }));

    // Create tab header
    let tab_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(4)
        .build();
    tab_box.append(&tab_label);
    tab_box.append(&close_btn);

    // Add to notebook
    tabs.append_page(&webview, Some(&tab_box));

    // Load homepage
    webview.load_uri("https://start.rustica-os.org");
}
```

### Context Menus

```rust
use webkit2gtk::{ContextMenuItem, ContextMenuExt};

fn setup_context_menu(webview: &WebView) {
    webview.connect_context_menu(|_, menu, event, hit_test_result| {
        // Remove default items
        menu.remove_all();

        // Add custom items
        if hit_test_result.context_is_link() {
            let open_link = ContextMenuItem::new(
                "Open Link",
                true,  // enabled
            );
            open_link.connect_activated(clone!(@weak hit_test_result => move |_| {
                let uri = hit_test_result.link_uri().unwrap();
                // Open link
            }));
            menu.append(&open_link);

            let copy_link = ContextMenuItem::new(
                "Copy Link Address",
                true,
            );
            copy_link.connect_activated(move |_| {
                let uri = hit_test_result.link_uri().unwrap();
                // Copy to clipboard
            });
            menu.append(&copy_link);
        }

        if hit_test_result.context_is_image() {
            let open_image = ContextMenuItem::new(
                "Open Image in New Tab",
                true,
            );
            open_image.connect_activated(move |_| {
                let uri = hit_test_result.image_uri().unwrap();
                // Open image
            });
            menu.append(&open_image);
        }

        // Always show "Inspect Element" if dev tools enabled
        let inspect = ContextMenuItem::new(
            "Inspect Element",
            true,
        );
        inspect.connect_activated(move |_| {
            // Open inspector
        });
        menu.append(&inspect);

        false  // Don't show default menu
    });
}
```

---

## Features

### Core Features

**Navigation**:
- Forward/back navigation
- URL bar with search integration
- History navigation
- Bookmark management
- Session restore

**Tabs**:
- Unlimited tabs
- Tab pinning
- Tab grouping
- Tab search
- Reopen closed tabs

**Privacy**:
- Private browsing mode
- Clear browsing data
- Cookie controls
- Tracker blocking
- Fingerprinting protection

**Downloads**:
- Download manager
- Pause/resume downloads
- Download history
- Auto-open options
- Scan for malware

**Bookmarks**:
- Bookmark bar
- Bookmark folders
- Import/export
- Sync across devices
- Smart bookmarks

### Reading Features

**Reader Mode**:
```rust
use webkit2gtk::WebViewExt;

fn enable_reader_mode(webview: &WebView) {
    // WebKit has built-in reader mode
    webview.run_javascript(
        "if (window.readerMode) window.readerMode.activate();",
        None::<&gio::Cancellable>,
        |result| {
            // Handle result
        }
    );
}
```

**Reading List**:
```rust
#[derive(Serialize, Deserialize)]
struct ReadingListItem {
    url: String,
    title: String,
    added_at: chrono::DateTime<chrono::Utc>,
    read: bool,
}

fn add_to_reading_list(url: &str, title: &str) {
    let item = ReadingListItem {
        url: url.to_string(),
        title: title.to_string(),
        added_at: chrono::Utc::now(),
        read: false,
    };

    // Save to ~/.config/rustica-web/reading-list.json
    // ...
}
```

### Developer Tools

**WebKit Inspector Integration**:
```rust
use webkit2gtk::InspectorExt;

fn toggle_inspector(webview: &WebView) {
    let inspector = webview.inspector().unwrap();

    if inspector.is_attached() {
        inspector.detach();
    } else {
        inspector.show();
        // Open on same window
        inspector.attach(&webview);
    }
}
```

---

## Performance

### Memory Management

**Per-Tab Memory Limits**:
```rust
use webkit2gtk::WebsiteDataManagerExt;

// Configure memory limits
let data_manager = WebsiteDataManager::builder()
    .build();

// Limit cache size
data_manager.set_disk_cache(500 * 1024 * 1024);  // 500MB

// Limit per-process memory
// Use cgroups or seccomp to enforce
```

**Tab Discarding**:
```rust
// Discard tabs when memory is low
fn discard_low_priority_tabs(tabs: &Vec<BrowserTab>) {
    let available_memory = get_available_memory();

    if available_memory < THRESHOLD {
        // Discard oldest tabs first
        for tab in tabs.iter().rev() {
            if tab.is_suspended() {
                continue;
            }

            if tab.can_discard() {
                tab.discard();
                break;
            }
        }
    }
}
```

### Hardware Acceleration

**GPU Acceleration**:
```rust
use webkit2gtk::SettingsExt;

let settings = webview.settings().unwrap();

// Enable GPU acceleration
settings.set_hardware_acceleration_policy(
    webkit2gtk::HardwareAccelerationPolicy::Always
);

// Enable WebGL
settings.set_enable_webgl(true);

// Enable WebRTC
settings.set_enable_media_stream(true);
```

---

## Privacy

### Privacy Features

**Tracking Protection**:
```rust
use std::collections::HashSet;

// EasyList + EasyPrivacy filters
let tracker_blocklist = HashSet::from([
    "doubleclick.net",
    "google-analytics.com",
    "facebook.com/tr",
    "googlesyndication.com",
    // ... more
]);

fn should_block_request(url: &str) -> bool {
    let parsed = url::Url::parse(url).unwrap();
    let domain = parsed.domain().unwrap();

    tracker_blocklist.contains(&domain.to_string())
}
```

**HTTPS-Only Mode**:
```rust
use webkit2gtk::TLSErrorsPolicy;

let context = WebContext::default();
context.set_tls_errors_policy(TLSErrorsPolicy::Fail);

// Redirect HTTP to HTTPS
context.register_uri_scheme_handler("http", |request| {
    let https_url = request.uri().replace("http:", "https:");
    request.finish_with_redirect_uri(&https_url);
});
```

**Clear Browsing Data**:
```rust
use webkit2gtk::WebsiteDataManagerExt;

fn clear_browsing_data(data_manager: &WebsiteDataManager) {
    // Clear all data
    data_manager.clear(
        WebsiteDataTypes::all(),  // All types
        0,  // Since beginning of time
        None::<&gio::Cancellable>,
        |result| {
            match result {
                Ok(_) => println!("Data cleared"),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
    );
}

// Clear specific types
fn clear_cookies(data_manager: &WebsiteDataManager) {
    data_manager.clear(
        WebsiteDataTypes::COOKIES,
        0,
        None::<&gio::Cancellable>,
        |_| {}
    );
}
```

### Cookie Controls

```rust
use webkit2gtk::CookieManagerExt;

let cookie_manager = context.cookie_manager().unwrap();

// Third-party cookie blocking
cookie_manager.set_accept_policy(
    CookieAcceptPolicy::NO_THIRD_PARTY
);

// Session-only cookies
cookie_manager.set_persistent_storage(
    "/dev/null",  // Don't persist
    CookiePersistentStorage::TEXT
);
```

---

## Extension System

### WebExtension Support

WebKitGTK supports WebExtensions (Chrome-compatible):

```rust
use webkit2gtk::WebContextExt;

let context = WebContext::default();

// Enable extensions
context.set_web_extensions_directory(
    "/usr/share/rustica-web/extensions"
);

// Set up extension message handler
context.connect_initialize_web_extensions(|_| {
    // Initialize extensions
});

// Register user extension directory
context.set_web_extensions_additional_directory(
    dirs::home_dir().unwrap()
        .join(".local/share/rustica-web/extensions")
);
```

### Rust Extension API

**Native Rust Extensions**:
```rust
// Extension API
pub trait BrowserExtension {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn on_page_load(&self, url: &str, webview: &WebView);
    fn on_context_menu(&self, menu: &ContextMenu, url: &str);
}

// Example: Password manager extension
pub struct PasswordManagerExtension;

impl BrowserExtension for PasswordManagerExtension {
    fn name(&self) -> &str {
        "Password Manager"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn on_page_load(&self, url: &str, webview: &WebView) {
        // Inject password manager script
        webview.run_javascript(
            &format!("initPasswordManager('{}')", url),
            None::<&gio::Cancellable>,
            |_| {}
        );
    }

    fn on_context_menu(&self, menu: &ContextMenu, url: &str) {
        if url.contains("login") {
            menu.add_item("Fill Password", || {
                // Fill saved password
            });
        }
    }
}
```

---

## Sync Services

### Account Integration

```rust
// Sync with Rustica Account
use rustica_account::AccountService;

pub struct SyncService {
    account: AccountService,
    local_data: LocalStore,
    remote_data: RemoteStore,
}

impl SyncService {
    pub fn sync_bookmarks(&self) -> Result<()> {
        // Get local bookmarks
        let local_bookmarks = self.local_data.get_bookmarks()?;

        // Get remote bookmarks
        let remote_bookmarks = self.remote_data.get_bookmarks()?;

        // Merge
        let merged = merge_bookmarks(local_bookmarks, remote_bookmarks);

        // Upload to server
        self.remote_data.put_bookmarks(merged)?;

        Ok(())
    }

    pub fn sync_history(&self) -> Result<()> {
        // Similar for browsing history
        // ...
    }

    pub fn sync_settings(&self) -> Result<()> {
        // Sync browser settings
        // ...
    }
}
```

### End-to-End Encryption

```rust
use rustica_crypto::{Encrypt, Decrypt};

// Encrypt bookmarks before syncing
pub fn encrypt_bookmarks(
    bookmarks: &[Bookmark],
    key: &EncryptionKey
) -> Result<Vec<u8>> {
    let json = serde_json::to_string(bookmarks)?;
    key.encrypt(json.as_bytes())
}

// Decrypt after downloading
pub fn decrypt_bookmarks(
    data: &[u8],
    key: &EncryptionKey
) -> Result<Vec<Bookmark>> {
    let decrypted = key.decrypt(data)?;
    let json = String::from_utf8(decrypted)?;
    Ok(serde_json::from_str(&json)?)
}
```

---

## Build System

### Rust Wrapper Build

**Cargo.toml**:
```toml
[package]
name = "rustica-web"
version = "1.0.0"
edition = "2021"

[dependencies]
gtk = { version = "0.8", features = ["v4_0"] }
webkit2gtk = "0.20"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
dirs = "5.0"
url = "2.5"
urlencoding = "2.1"
once_cell = "1.19"

[build-dependencies]
gtk-rs-lgpl-docs = "0.1"
```

### Build Script

**build.rs**:
```rust
use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo where to find WebKitGTK
    println!("cargo:rustc-link-lib=webkit2gtk-4.1");
    println!("cargo:rustc-link-lib=soup-3.0");
    println!("cargo:rustc-link-lib=javascriptcoregtk-4.1");

    // Find pkg-config paths
    let webkit_lib = pkg_config::probe_library("webkit2gtk-4.1")
        .unwrap()
        .include_paths
        .get(0)
        .unwrap()
        .clone();

    println!("cargo:include={}", webkit_lib.display());
}
```

### Installation

**Dependencies**:
```bash
# On Ubuntu/Debian
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  libsoup-3.0-dev \
  libjavascriptcoregtk-4.1-dev \
  libgtk-4-dev

# On Arch
sudo pacman -S \
  webkit2gtk-4.1 \
  gtk4
```

**Build**:
```bash
# Build Rust shell
cargo build --release

# Install to system
sudo cp target/release/rustica-web /usr/bin/
sudo cp org.rustica.WebBrowser.desktop /usr/share/applications/
sudo cp icons/*.svg /usr/share/icons/hicolor/scalable/apps/
```

### Desktop Entry

**org.rustica.WebBrowser.desktop**:
```ini
[Desktop Entry]
Name=Rustica Web
GenericName=Web Browser
Comment=Browse the web
Exec=rustica-web %U
Terminal=false
Type=Application
Icon=org.rustica.WebBrowser
Categories=Network;WebBrowser;
StartupNotify=true
MimeType=text/html;text/xml;application/xhtml+xml;xml;application/vnd.mozilla.xul+xml;text/mml;x-scheme-handler/http;x-scheme-handler/https;x-scheme-handler/ftp;
```

---

## Appendix

### Configuration Files

**~/.config/rustica-web/settings.json**:
```json
{
  "homepage": "https://start.rustica-os.org",
  "search_engine": "https://search.rustica-os.org/search?q={}",
  "download_dir": "~/Downloads",
  "theme": "dark",
  "font_size": 16,
  "default_zoom": 1.0,
  "privacy": {
    "block_trackers": true,
    "block_ads": false,
    "https_only": true,
    "send_referrer": false,
    "send_do_not_track": true
  },
  "permissions": {
    "geolocation": "ask",
    "notifications": "ask",
    "camera": "ask",
    "microphone": "ask"
  },
  "sync": {
    "enabled": false,
    "server": "https://sync.rustica-os.org",
    "interval": 1800
  }
}
```

### D-Bus Interface

**org.rustica.WebBrowser.xml**:
```xml
<node name="/org/rustica/WebBrowser">
  <interface name="org.rustica.WebBrowser">
    <method name="OpenURL">
      <arg name="url" type="s" direction="in"/>
    </method>

    <method name="NewTab">
      <arg name="url" type="s" direction="in"/>
    </method>

    <method name="Search">
      <arg name="query" type="s" direction="in"/>
    </method>

    <signal name="PageLoaded">
      <arg name="url" type="s"/>
      <arg name="title" type="s"/>
    </signal>
  </interface>
</node>
```

### Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-01-07 | Initial specification |

---

**Rustica Web Browser** - Fast, secure, privacy-focused web browsing for Rustica OS.
