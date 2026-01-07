//! Rustica Web Browser
//!
//! A modern, secure, privacy-focused web browser built on WebKitGTK
//! for the Rustica OS desktop environment.

use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box as GtkBox, Orientation,
    Entry, Button, Label, Separator, GestureSwipe, GestureZoom,
    EventControllerScroll, EventControllerScrollFlags
};
use webkit2gtk::{WebView, WebViewExt, LoadEvent, SettingsExt, UserContentManager};
use std::rc::Rc;
use std::cell::RefCell;
use std::env;

/// Browser configuration
#[derive(Debug, Clone, Copy)]
pub struct BrowserConfig {
    /// Mobile mode (touch-optimized UI)
    pub mobile_mode: bool,
    /// Use mobile user agent
    pub mobile_user_agent: bool,
}

impl BrowserConfig {
    /// Detect if we should run in mobile mode
    pub fn detect() -> Self {
        // Check environment variables
        let mobile_mode = env::var("RUSTICA_MOBILE_MODE")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(false);

        // Check for touch devices via GDK display
        let is_touch_device = Self::has_touch_support();

        // Check screen size (mobile if < 768px width)
        let is_small_screen = Self::is_small_screen();

        let mobile_mode = mobile_mode || is_touch_device || is_small_screen;
        let mobile_user_agent = mobile_mode;

        BrowserConfig {
            mobile_mode,
            mobile_user_agent,
        }
    }

    /// Check if display has touch support
    fn has_touch_support() -> bool {
        // Simplified touch detection - check environment variable
        // In a real implementation, this would query the display device list
        // For GTK3, device enumeration is more complex
        env::var("RUSTICA_TOUCH_DEVICE").is_ok()
    }

    /// Check if screen is small (mobile/tablet size)
    fn is_small_screen() -> bool {
        if let Some(display) = gtk::gdk::Display::default() {
            if let Some(monitor) = display.primary_monitor() {
                let geom = monitor.geometry();
                return geom.width() < 768 || geom.height() < 768;
            }
        }
        false
    }

    /// Get user agent string
    pub fn user_agent(&self) -> &'static str {
        if self.mobile_user_agent {
            "Mozilla/5.0 (Linux; Android 13) AppleWebKit/537.36 (KHTML, like Gecko) \
             Chrome/120.0.0.0 Mobile Safari/537.36 RusticaMobile/1.0"
        } else {
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) \
             Chrome/120.0.0.0 Safari/537.36 Rustica/1.0"
        }
    }
}

/// Browser application state
struct BrowserApp {
    /// Current URL
    current_url: Rc<RefCell<String>>,
    /// Browser configuration
    config: BrowserConfig,
    /// User content manager
    user_content: UserContentManager,
}

impl BrowserApp {
    /// Create a new browser application
    fn new() -> Self {
        let config = BrowserConfig::detect();

        if config.mobile_mode {
            println!("Rustica Web: Mobile mode enabled");
        }

        let user_content = UserContentManager::new();

        BrowserApp {
            current_url: Rc::new(RefCell::new(String::from("about:blank"))),
            config,
            user_content,
        }
    }

    /// Build and run the browser UI
    fn build(&self, app: &Application) {
        // Create main window
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Rustica Web")
            .default_width(1280)
            .default_height(720)
            .build();

        // Main vertical box
        let vbox = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(0)
            .build();

        // Navigation bar
        let (nav_bar, url_entry, webview) = if self.config.mobile_mode {
            self.create_mobile_navigation_bar(&window, &vbox)
        } else {
            self.create_desktop_navigation_bar(&window, &vbox)
        };

        // Separator
        let separator = Separator::builder()
            .orientation(Orientation::Horizontal)
            .build();
        vbox.add(&separator);

        // Status bar (desktop only, hidden in mobile)
        let status_bar = if !self.config.mobile_mode {
            let sb = self.create_status_bar();
            vbox.add(&sb);
            Some(sb)
        } else {
            None
        };

        // Configure web view
        self.configure_webview(&webview, &status_bar, &window);

        // Add web view to main layout
        let scrolled_window = gtk::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .build();

        // Enable kinetic scrolling for mobile
        if self.config.mobile_mode {
            scrolled_window.set_kinetic_scrolling(true);
            scrolled_window.set_capture_button_press(true);
        }

        scrolled_window.add(&webview);
        vbox.add(&scrolled_window);

        // Add status bar separator for desktop
        if !self.config.mobile_mode {
            let separator2 = Separator::builder()
                .orientation(Orientation::Horizontal)
                .build();
            vbox.add(&separator2);
        }

        // Show all children
        vbox.show_all();

        // Set main content
        window.add(&vbox);

        // Connect URL entry handler
        self.setup_url_handler(&url_entry, &webview);

        // Setup gestures (mobile)
        if self.config.mobile_mode {
            self.setup_touch_gestures(&window, &webview);
        }

        // Load default page
        self.load_default_page(&webview);

        // Show window
        window.show_all();
    }

    /// Create desktop navigation bar
    fn create_desktop_navigation_bar(
        &self,
        window: &ApplicationWindow,
        vbox: &GtkBox
    ) -> (GtkBox, Entry, WebView) {
        let nav_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .margin_start(6)
            .margin_end(6)
            .margin_top(6)
            .margin_bottom(6)
            .build();

        // Create webview first
        let webview = WebView::builder()
            .user_content_manager(&self.user_content)
            .build();

        // Menu button
        let menu_btn = Button::from_icon_name(Some("open-menu-symbolic"), gtk::IconSize::Button);
        nav_box.add(&menu_btn);

        // Back button
        let back_btn = Button::from_icon_name(Some("go-previous-symbolic"), gtk::IconSize::Button);
        let webview_back = webview.clone();
        back_btn.connect_clicked(move |_| {
            webview_back.go_back();
        });
        nav_box.add(&back_btn);

        // Forward button
        let fwd_btn = Button::from_icon_name(Some("go-next-symbolic"), gtk::IconSize::Button);
        let webview_fwd = webview.clone();
        fwd_btn.connect_clicked(move |_| {
            webview_fwd.go_forward();
        });
        nav_box.add(&fwd_btn);

        // Refresh button
        let refresh_btn = Button::from_icon_name(Some("view-refresh-symbolic"), gtk::IconSize::Button);
        let webview_refresh = webview.clone();
        refresh_btn.connect_clicked(move |_| {
            webview_refresh.reload();
        });
        nav_box.add(&refresh_btn);

        // URL entry
        let url_entry = Entry::builder()
            .placeholder_text("Search or enter website")
            .hexpand(true)
            .build();
        nav_box.add(&url_entry);

        // Add navigation bar to main layout
        vbox.add(&nav_box);

        (nav_box, url_entry, webview)
    }

    /// Create mobile navigation bar
    fn create_mobile_navigation_bar(
        &self,
        window: &ApplicationWindow,
        vbox: &GtkBox
    ) -> (GtkBox, Entry, WebView) {
        // Mobile navigation - top bar with URL
        let top_bar = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(4)
            .margin_start(4)
            .margin_end(4)
            .margin_top(4)
            .margin_bottom(4)
            .build();

        // Create webview
        let webview = WebView::builder()
            .user_content_manager(&self.user_content)
            .build();

        // Menu button (hamburger)
        let menu_btn = Button::from_icon_name(Some("open-menu-symbolic"), gtk::IconSize::LargeToolbar);
        top_bar.add(&menu_btn);

        // URL entry (larger for touch)
        let url_entry = Entry::builder()
            .placeholder_text("Search or enter URL")
            .hexpand(true)
            .height_request(40)  // Larger for touch
            .build();
        top_bar.add(&url_entry);

        // Tabs button
        let tabs_btn = Button::from_icon_name(Some("view-list-symbolic"), gtk::IconSize::LargeToolbar);
        top_bar.add(&tabs_btn);

        // Add top bar
        vbox.add(&top_bar);

        // Bottom navigation bar (mobile-style)
        let bottom_bar = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(0)
            .homogeneous(true)
            .build();

        // Back button
        let back_btn = Button::from_icon_name(Some("go-previous-symbolic"), gtk::IconSize::LargeToolbar);
        let webview_back = webview.clone();
        back_btn.connect_clicked(move |_| {
            webview_back.go_back();
        });
        bottom_bar.add(&back_btn);

        // Forward button
        let fwd_btn = Button::from_icon_name(Some("go-next-symbolic"), gtk::IconSize::LargeToolbar);
        let webview_fwd = webview.clone();
        fwd_btn.connect_clicked(move |_| {
            webview_fwd.go_forward();
        });
        bottom_bar.add(&fwd_btn);

        // Home button
        let home_btn = Button::from_icon_name(Some("go-home-symbolic"), gtk::IconSize::LargeToolbar);
        let webview_home = webview.clone();
        home_btn.connect_clicked(move |_| {
            webview_home.load_uri("about:blank");
        });
        bottom_bar.add(&home_btn);

        // Refresh button
        let refresh_btn = Button::from_icon_name(Some("view-refresh-symbolic"), gtk::IconSize::LargeToolbar);
        let webview_refresh = webview.clone();
        refresh_btn.connect_clicked(move |_| {
            webview_refresh.reload();
        });
        bottom_bar.add(&refresh_btn);

        // Tabs button
        let tabs2_btn = Button::from_icon_name(Some("view-list-symbolic"), gtk::IconSize::LargeToolbar);
        bottom_bar.add(&tabs2_btn);

        // Add bottom bar at end (will be added after webview)
        vbox.add(&bottom_bar);

        (top_bar, url_entry, webview)
    }

    /// Configure web view settings
    fn configure_webview(
        &self,
        webview: &WebView,
        status_bar: &Option<Label>,
        window: &ApplicationWindow
    ) {
        // Configure settings
        if let Some(settings) = WebViewExt::settings(webview) {
            settings.set_enable_javascript(true);
            settings.set_enable_developer_extras(true);
            settings.set_enable_webgl(true);

            // Set user agent
            settings.set_user_agent(Some(self.config.user_agent()));

            // Mobile-specific settings
            if self.config.mobile_mode {
                // Enable smooth scrolling
                settings.set_enable_smooth_scrolling(true);

                // Enable spatial navigation (D-pad navigation)
                settings.set_enable_spatial_navigation(true);

                // Enable media playback with user gesture
                settings.set_media_playback_requires_user_gesture(false);

                // Enable fullscreen
                settings.set_enable_fullscreen(true);
            }
        }

        // Connect load events
        if let Some(sb) = status_bar {
            let sb_clone = sb.clone();
            webview.connect_load_changed(move |_, event| {
                match event {
                    LoadEvent::Started => {
                        sb_clone.set_text("Loading...");
                    }
                    LoadEvent::Committed => {
                        sb_clone.set_text("Loading...");
                    }
                    LoadEvent::Finished => {
                        sb_clone.set_text("Done");
                    }
                    _ => {}
                }
            });
        }

        // Connect title changes
        let window_clone = window.clone();
        webview.connect_title_notify(move |webview: &webkit2gtk::WebView| {
            if let Some(title) = webview.title() {
                if !title.is_empty() {
                    window_clone.set_title(&format!("{} - Rustica Web", title));
                }
            }
        });
    }

    /// Setup URL entry handler
    fn setup_url_handler(&self, url_entry: &Entry, webview: &WebView) {
        let webview_clone = webview.clone();
        let current_url_clone = self.current_url.clone();

        url_entry.connect_activate(move |entry| {
            let text = entry.text().to_string();

            // Check if it looks like a URL
            if text.contains('.') && !text.contains(' ') {
                // Add https:// if no scheme
                let url = if text.starts_with("http://") || text.starts_with("https://") {
                    text
                } else {
                    format!("https://{}", text)
                };

                webview_clone.load_uri(&url);
                *current_url_clone.borrow_mut() = url.clone();
                entry.set_text(&url);
            } else {
                // It's a search query
                let search_url = format!(
                    "https://html.duckduckgo.com/html/?q={}",
                    urlencoding::encode(&text)
                );
                webview_clone.load_uri(&search_url);
                *current_url_clone.borrow_mut() = search_url.clone();
                entry.set_text(&search_url);
            }
        });
    }

    /// Setup touch gestures for mobile
    fn setup_touch_gestures(&self, window: &ApplicationWindow, webview: &WebView) {
        // For GTK3, we need to add gestures to the widget directly
        // Note: Full gesture support would require GTK4, but we can setup basic handling

        // The WebView handles most touch gestures internally in WebKitGTK
        // We enable touch handling in the settings

        // Note: Swipe and zoom gestures would need to be implemented at
        // the WebView level using JavaScript or WebKit extensions
        // For now, WebKit's built-in touch handling is enabled
    }

    /// Load default/welcome page
    fn load_default_page(&self, webview: &WebView) {
        let page_title = if self.config.mobile_mode {
            "Rustica Web (Mobile)"
        } else {
            "Rustica Web"
        };

        let page_html = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>{title}</title>
                <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
                <style>
                    body {{
                        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
                        display: flex;
                        flex-direction: column;
                        align-items: center;
                        justify-content: center;
                        min-height: 100vh;
                        margin: 0;
                        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                        color: white;
                        padding: 20px;
                        box-sizing: border-box;
                    }}
                    h1 {{
                        font-size: {heading_size};
                        margin-bottom: 0.5em;
                        text-align: center;
                    }}
                    p {{
                        font-size: 1.2em;
                        opacity: 0.9;
                        text-align: center;
                        max-width: 600px;
                    }}
                    .search-box {{
                        margin-top: 2em;
                        padding: 1.5em 2em;
                        background: rgba(255, 255, 255, 0.2);
                        border-radius: 16px;
                        backdrop-filter: blur(10px);
                        text-align: center;
                    }}
                    .features {{
                        display: flex;
                        flex-wrap: wrap;
                        justify-content: center;
                        gap: 1em;
                        margin-top: 2em;
                        max-width: 800px;
                    }}
                    .feature {{
                        background: rgba(255, 255, 255, 0.15);
                        padding: 1em 1.5em;
                        border-radius: 12px;
                        font-size: 0.95em;
                    }}
                    @media (max-width: 768px) {{
                        h1 {{ font-size: 2em; }}
                        .features {{ flex-direction: column; }}
                        .feature {{ width: 100%; box-sizing: border-box; }}
                    }}
                </style>
            </head>
            <body>
                <h1>{title}</h1>
                <p>A modern, secure, privacy-focused web browser</p>
                <div class="search-box">
                    <p>Enter a URL or search above to get started</p>
                </div>
                <div class="features">
                    <div class="feature">🔒 Privacy-focused</div>
                    <div class="feature">⚡ Fast & Lightweight</div>
                    <div class="feature">🔒 Secure by default</div>
                </div>
            </body>
            </html>
            "#,
            title = page_title,
            heading_size = if self.config.mobile_mode { "2.5em" } else { "3em" }
        );

        webview.load_html(&page_html, None);
    }

    /// Create status bar (desktop only)
    fn create_status_bar(&self) -> Label {
        Label::builder()
            .label("Ready")
            .margin_start(6)
            .margin_end(6)
            .margin_top(3)
            .margin_bottom(3)
            .build()
    }
}

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .init();

    // Create GTK application
    let app = Application::builder()
        .application_id("org.rustica.WebBrowser")
        .build();

    // Connect activate signal
    app.connect_activate(|app| {
        let browser = BrowserApp::new();
        browser.build(app);
    });

    // Run application
    app.run();
}
