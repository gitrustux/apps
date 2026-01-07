# Application Library (rustica-applibrary) Specification

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Component**: Rustica Shell - Application Library
**Phase**: 6.5 - Desktop Applications

## Overview

Rustica Application Library is a **beautiful, user-friendly app store** for the RUSTUX OS. It provides **app browsing**, **searching**, **categories**, **installation**, **updates**, **reviews**, **screenshots**, and **seamless integration** with the package manager. It serves as the **primary interface** for discovering and installing software on RUSTUX.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Rustica Application Library                                            │
├─────────────────────────────────────────────────────────────────────────┤
│  [≡] [Discover] [Installed] [Updates] [Settings]            [□] [−] [×]  │
├─────────────────────────────────────────────────────────────────────────┤
│  Search applications...                                      [🔍]      │
├─────────────────────────────────────────────────────────────────────────┤
│  Categories                                                               │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐         │
│  │  🎮     │ │  💻     │ │  🎨     │ │  📝     │ │  🔧     │         │
│  │ Games   │ │ Dev     │ │ Graphics │ │ Office  │ │ System  │         │
│  │  234    │ │  567    │ │   89    │ │  123    │ │  456    │         │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘         │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐                                     │
│  │  🌐     │ │  🎵     │ │  📚     │                                     │
│  │ Internet│ │ Audio   │ │ Education│                                     │
│  │  345    │ │   78    │ │   91    │                                     │
│  └─────────┘ └─────────┘ └─────────┘                                     │
├─────────────────────────────────────────────────────────────────────────┤
│  Featured Applications                                                   │
│  ┌─────────────────────┐ ┌─────────────────────┐ ┌───────────────────┐ │
│  │  [Firefox Icon]     │ │  [VS Code Icon]     │ │  [GIMP Icon]      │ │
│  │                     │ │                     │ │                   │ │
│  │  Firefox Web Browser│ │  Visual Studio Code │ │  GIMP             │ │
│  │                     │ │                     │ │  Image Editor     │ │
│  │  ★★★★☆ 4.5         │ │  ★★★★★ 4.8         │ │  ★★★★☆ 4.2       │ │
│  │  Free               │ │  Free               │ │  Free             │ │
│  │  [Install]          │ │  [Install]          │ │  [Install]        │ │
│  └─────────────────────┘ └─────────────────────┘ └───────────────────┘ │
└─────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│  Firefox - Application Details                                          │
├─────────────────────────────────────────────────────────────────────────┤
│  [◀ Back]                                         [⋮] [Install] [▶]  │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌─────────┐                                                             │
│  │         │  Mozilla Firefox                                            │
│  │ [Icon]  │  Free                                                       │
│  │         │  ★★★★☆ (4,523 reviews)                                     │
│  └─────────┘                                                             │
│                                                                           │
│  Screenshots                                        [◀] 1 / 5 [▶]       │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  [Screenshot Image]                                              │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                           │
│  Description                                                              │
│  Mozilla Firefox is a free and open-source web browser developed by...   │
│  [Show more]                                                             │
│                                                                           │
│  Information                                                             │
│  Version: 120.0.1           Size: 85 MB                                 │
│  Developer: Mozilla          License: MPL-2.0                            │
│  Category: Internet          Website: mozilla.org                        │
│                                                                           │
│  Reviews                                                                 │
│  ★★★★★ "Best browser ever!" - user123  (2 days ago)                     │
│  ★★★★☆ "Great but needs..." - jane_doe  (1 week ago)                   │
│  [See all reviews]                                                       │
└─────────────────────────────────────────────────────────────────────────┘
```

## Main Application Structure

```rust
pub struct AppLibrary {
    /// Window
    window: Window,

    /// Current page
    current_page: Page,

    /// Navigation history
    history: Vec<Page>,

    /// Package manager client
    package_manager: PackageManagerClient,

    /// App registry
    registry: AppRegistry,

    /// Cache
    cache: AppCache,

    /// Settings
    settings: LibrarySettings,

    /// Download manager
    downloads: DownloadManager,
}

pub enum Page {
    Home,
    Category { category: Category },
    Search { query: String },
    AppDetails { app_id: String },
    Installed,
    Updates,
    Settings,
}

pub struct PackageManagerClient {
    /// D-Bus connection
    dbus: Connection,

    /// Backend type
    backend: PackageManagerBackend,
}

pub enum PackageManagerBackend {
    Rustux,    // Native rustup package manager
    Flatpak,   // Flatpak support
    Snap,      // Snap support (optional)
    AppImage,  // AppImage support (optional)
}

pub struct AppRegistry {
    /// All apps
    apps: HashMap<String, AppEntry>,

    /// Categories
    categories: Vec<Category>,

    /// Featured apps
    featured: Vec<String>,
}

pub struct AppCache {
    /// App metadata cache
    metadata: HashMap<String, CachedMetadata>,

    /// Screenshot cache
    screenshots: HashMap<String, Vec<Image>>,

    /// Icon cache
    icons: HashMap<String, Image>,

    /// Timestamps
    timestamps: HashMap<String, DateTime<Utc>>,
}

pub struct CachedMetadata {
    /// App entry
    pub entry: AppEntry,

    /// Screenshots
    pub screenshots: Vec<String>,

    /// Reviews
    pub reviews: Vec<Review>,

    /// Related apps
    pub related: Vec<String>,

    /// Cache time
    pub cached_at: DateTime<Utc>,
}

pub struct AppEntry {
    /// App ID
    pub id: String,

    /// Name
    pub name: String,

    /// Summary (short description)
    pub summary: String,

    /// Description (long description)
    pub description: String,

    /// Icon
    pub icon: String,

    /// Screenshots
    pub screenshots: Vec<String>,

    /// Category
    pub category: Category,

    /// Developer
    pub developer: String,

    /// License
    pub license: String,

    /// Website
    pub website: String,

    /// Version
    pub version: String,

    /// Installed version (if installed)
    pub installed_version: Option<String>,

    /// Size (bytes)
    pub size: u64,

    /// Download size (bytes)
    pub download_size: u64,

    /// Rating
    pub rating: Option<Rating>,

    /// Reviews
    pub reviews: Vec<Review>,

    /// Package type
    pub package_type: PackageType,

    /// Repository
    pub repository: String,

    /// Dependencies
    pub dependencies: Vec<String>,

    /// State
    pub state: AppState,
}

pub struct Rating {
    /// Average score (0-5)
    pub score: f32,

    /// Number of reviews
    pub count: usize,

    /// Distribution (1-5 stars)
    pub distribution: [usize; 5],
}

pub struct Review {
    /// Review ID
    pub id: String,

    /// Author
    pub author: String,

    /// Rating (1-5)
    pub rating: u8,

    /// Title
    pub title: Option<String>,

    /// Content
    pub content: String,

    /// Date
    pub date: DateTime<Utc>,

    /// Helpful count
    pub helpful: usize,
}

pub enum PackageType {
    Native,    // RUSTUX native package
    Flatpak,   // Flatpak package
    Snap,      // Snap package
    AppImage,  // AppImage
}

pub enum AppState {
    Available,
    Installed,
    UpdateAvailable,
    Installing(InstallationProgress),
    Failed(String),
}

pub struct InstallationProgress {
    /// Bytes downloaded
    pub downloaded: u64,

    /// Total bytes
    pub total: u64,

    /// Current operation
    pub operation: String,

    /// Overall progress (0-100)
    pub progress: u8,
}
```

## Home Page

```rust
pub struct HomePage {
    /// Featured apps
    featured: Vec<AppEntry>,

    /// Categories
    categories: Vec<Category>,

    /// Popular apps
    popular: Vec<AppEntry>,

    /// New apps
    new: Vec<AppEntry>,

    /// Recently updated
    updated: Vec<AppEntry>,
}

impl HomePage {
    pub fn render(&self, ctx: &mut RenderContext) {
        let mut y = 100.0;

        // Search bar
        y = self.draw_search_bar(ctx, y);
        y += 32.0;

        // Categories
        y = self.draw_section_header(ctx, y, "Categories");
        y = self.draw_categories(ctx, y);
        y += 32.0;

        // Featured apps
        y = self.draw_section_header(ctx, y, "Featured Applications");
        y = self.draw_featured_apps(ctx, y);
        y += 32.0;

        // Popular apps
        y = self.draw_section_header(ctx, y, "Popular Applications");
        y = self.draw_app_grid(ctx, y, &self.popular);
    }

    fn draw_search_bar(&self, ctx: &mut RenderContext, y: f32) -> f32 {
        let rect = Rect {
            x: 24.0,
            y,
            width: 950.0,
            height: 48.0,
        };

        // Background
        ctx.fill_rounded_rect(rect, 24.0, theme.colors.surface_variant);

        // Search icon
        ctx.draw_icon(
            Rect { x: rect.x + 16.0, y: rect.y + 12.0, width: 24.0, height: 24.0 },
            "search",
        );

        // Placeholder text
        ctx.draw_text(
            Rect { x: rect.x + 56.0, y: rect.y + 14.0, width: 800.0, height: 20.0 },
            "Search applications...",
            theme.typography.body,
            theme.colors.on_surface_variant,
        );

        y + rect.height + 16.0
    }

    fn draw_categories(&self, ctx: &mut RenderContext, y: f32) -> f32 {
        let mut x = 24.0;
        let card_size = 160.0;
        let spacing = 16.0;

        for category in &self.categories {
            let card_rect = Rect {
                x,
                y,
                width: card_size,
                height: card_size + 40.0,
            };

            // Draw category card
            self.draw_category_card(ctx, card_rect, category);

            x += card_size + spacing;
        }

        y + card_size + 56.0
    }

    fn draw_category_card(&self, ctx: &mut RenderContext, rect: Rect, category: &Category) {
        // Background
        ctx.fill_rounded_rect(rect, 12.0, theme.colors.surface);

        // Icon background (gradient)
        let icon_rect = Rect {
            x: rect.x + (rect.width - 64.0) / 2.0,
            y: rect.y + 16.0,
            width: 64.0,
            height: 64.0,
        };

        ctx.fill_circle(
            Point { x: icon_rect.x + 32.0, y: icon_rect.y + 32.0 },
            32.0,
            category.color,
        );

        // Icon
        ctx.draw_icon(icon_rect, &category.icon);

        // Name
        ctx.draw_text_centered(
            Rect { x: rect.x, y: rect.y + rect.height - 32.0, width: rect.width, height: 20.0 },
            &category.name,
            theme.typography.caption,
            theme.colors.on_surface,
        );

        // App count
        ctx.draw_text_centered(
            Rect { x: rect.x, y: rect.y + rect.height - 16.0, width: rect.width, height: 14.0 },
            &format!("{} apps", category.count),
            theme.typography.caption,
            theme.colors.on_surface_variant,
        );
    }

    fn draw_featured_apps(&self, ctx: &mut RenderContext, y: f32) -> f32 {
        let card_width = 300.0;
        let spacing = 16.0;
        let mut x = 24.0;

        for app in &self.featured {
            let card_rect = Rect {
                x,
                y,
                width: card_width,
                height: 280.0,
            };

            self.draw_app_card(ctx, card_rect, app);

            x += card_width + spacing;
        }

        y + 296.0
    }

    fn draw_app_card(&self, ctx: &mut RenderContext, rect: Rect, app: &AppEntry) {
        // Background
        ctx.fill_rounded_rect(rect, 12.0, theme.colors.surface);

        // Shadow
        ctx.draw_shadow(rect, theme.shadows.md);

        // Icon
        let icon_rect = Rect {
            x: rect.x + (rect.width - 96.0) / 2.0,
            y: rect.y + 24.0,
            width: 96.0,
            height: 96.0,
        };

        // Try to load cached icon
        if let Ok(icon) = self.load_icon(&app.icon) {
            ctx.draw_image_rounded(icon_rect, &icon, 12.0);
        }

        // Name
        ctx.draw_text_centered(
            Rect { x: rect.x, y: rect.y + 130.0, width: rect.width, height: 24.0 },
            &app.name,
            theme.typography.h4,
            theme.colors.on_surface,
        );

        // Summary
        ctx.draw_text_wrapped(
            Rect { x: rect.x + 16.0, y: rect.y + 160.0, width: rect.width - 32.0, height: 40.0 },
            &app.summary,
            theme.typography.sm,
            theme.colors.on_surface_variant,
        );

        // Rating
        if let Some(ref rating) = app.rating {
            let rating_y = rect.y + 210.0;
            self.draw_rating(ctx, Rect { x: rect.x + 16.0, y: rating_y, width: rect.width - 32.0, height: 20.0 }, rating);
        }

        // Install button
        let button_rect = Rect {
            x: rect.x + (rect.width - 120.0) / 2.0,
            y: rect.y + rect.height - 48.0,
            width: 120.0,
            height: 36.0,
        };

        let button_text = match app.state {
            AppState::Available => "Install",
            AppState::Installed => "Open",
            AppState::UpdateAvailable => "Update",
            _ => "Installing...",
        };

        ctx.draw_button(button_rect, button_text);
    }

    fn draw_rating(&self, ctx: &mut RenderContext, rect: Rect, rating: &Rating) {
        // Draw stars
        let star_size = 16.0;
        let mut x = rect.x;

        for i in 0..5 {
            let star_rect = Rect { x, y: rect.y, width: star_size, height: star_size };

            if i < rating.score as usize {
                ctx.draw_icon(star_rect, "star-filled");
            } else {
                ctx.draw_icon(star_rect, "star-outline");
            }

            x += star_size + 4.0;
        }

        // Rating text
        let rating_text = format!("{} ({})", rating.score, rating.count);
        ctx.draw_text(
            Rect { x: rect.x + 100.0, y: rect.y + 2.0, width: 100.0, height: 16.0 },
            &rating_text,
            theme.typography.caption,
            theme.colors.on_surface_variant,
        );
    }
}
```

## App Details Page

```rust
pub struct AppDetailsPage {
    /// App entry
    app: AppEntry,

    /// Current screenshot index
    screenshot_index: usize,

    /// Expanded description
    description_expanded: bool,

    /// Reviews page
    reviews_page: usize,

    /// Installing
    installing: bool,
}

impl AppDetailsPage {
    pub fn render(&self, ctx: &mut RenderContext) {
        let mut y = 60.0;

        // Back button and install button
        y = self.draw_header(ctx, y);
        y += 24.0;

        // App info (icon, name, rating)
        y = self.draw_app_info(ctx, y);
        y += 24.0;

        // Screenshots carousel
        y = self.draw_screenshots(ctx, y);
        y += 24.0;

        // Description
        y = self.draw_description(ctx, y);
        y += 24.0;

        // Information
        y = self.draw_information(ctx, y);
        y += 24.0;

        // Reviews preview
        self.draw_reviews(ctx, y);
    }

    fn draw_header(&self, ctx: &mut RenderContext, y: f32) -> f32 {
        // Back button
        ctx.draw_button(
            Rect { x: 24.0, y, width: 80.0, height: 36.0 },
            "◀ Back",
        );

        // Install/Open button
        let button_rect = Rect {
            x: ctx.width() - 140.0,
            y,
            width: 120.0,
            height: 36.0,
        };

        let (button_text, color) = match self.app.state {
            AppState::Available => ("Install", theme.colors.primary),
            AppState::Installed => ("Open", theme.colors.primary),
            AppState::UpdateAvailable => ("Update", theme.colors.primary),
            AppState::Installing(_) => ("Installing...", theme.colors.surface_variant),
            AppState::Failed(_) => ("Retry", theme.colors.error),
        };

        ctx.draw_button_colored(button_rect, button_text, color);

        y + 48.0
    }

    fn draw_app_info(&self, ctx: &mut RenderContext, y: f32) -> f32 {
        // Icon
        let icon_rect = Rect { x: 24.0, y, width: 128.0, height: 128.0 };

        if let Ok(icon) = self.load_icon(&self.app.icon) {
            ctx.draw_image_rounded(icon_rect, &icon, 16.0);
        }

        // Name
        ctx.draw_text(
            Rect { x: 170.0, y, width: 600.0, height: 32.0 },
            &self.app.name,
            theme.typography.h2,
            theme.colors.on_surface,
        );

        // License
        ctx.draw_text(
            Rect { x: 170.0, y: y + 36.0, width: 200.0, height: 20.0 },
            &self.app.license,
            theme.typography.body,
            theme.colors.on_surface_variant,
        );

        // Rating
        if let Some(ref rating) = self.app.rating {
            self.draw_rating(
                ctx,
                Rect { x: 170.0, y: y + 60.0, width: 300.0, height: 24.0 },
                rating,
            );
        }

        y + 144.0
    }

    fn draw_screenshots(&self, ctx: &mut RenderContext, y: f32) -> f32 {
        // Section header
        ctx.draw_text(
            Rect { x: 24.0, y, width: 200.0, height: 24.0 },
            "Screenshots",
            theme.typography.h4,
            theme.colors.on_surface,
        );

        let screenshot_y = y + 32.0;

        // Main screenshot
        let main_rect = Rect {
            x: 24.0,
            y: screenshot_y,
            width: 950.0,
            height: 500.0,
        };

        if let Some(screenshot) = self.app.screenshots.get(self.screenshot_index) {
            if let Ok(image) = self.load_screenshot(screenshot) {
                ctx.draw_image_rounded(main_rect, &image, 8.0);
            }
        }

        // Navigation arrows
        if self.app.screenshots.len() > 1 {
            // Left arrow
            ctx.draw_button(
                Rect { x: main_rect.x + 16.0, y: main_rect.y + main_rect.height - 60.0, width: 40.0, height: 40.0 },
                "◀",
            );

            // Right arrow
            ctx.draw_button(
                Rect { x: main_rect.x + main_rect.width - 56.0, y: main_rect.y + main_rect.height - 60.0, width: 40.0, height: 40.0 },
                "▶",
            );

            // Indicator
            let indicator = format!("{}/{}", self.screenshot_index + 1, self.app.screenshots.len());
            ctx.draw_text_centered(
                Rect { x: main_rect.x, y: main_rect.y + main_rect.height - 50.0, width: main_rect.width, height: 20.0 },
                &indicator,
                theme.typography.caption,
                theme.colors.on_surface,
            );
        }

        screenshot_y + 516.0
    }

    fn draw_description(&self, ctx: &mut RenderContext, y: f32) -> f32 {
        // Section header
        ctx.draw_text(
            Rect { x: 24.0, y, width: 200.0, height: 24.0 },
            "Description",
            theme.typography.h4,
            theme.colors.on_surface,
        );

        let text_y = y + 32.0;

        // Description text
        let description_rect = Rect {
            x: 24.0,
            y: text_y,
            width: 950.0,
            height: if self.description_expanded { 500.0 } else { 100.0 },
        };

        ctx.draw_text_wrapped(
            description_rect,
            &self.app.description,
            theme.typography.body,
            theme.colors.on_surface,
        );

        // Show more/less button
        if !self.description_expanded {
            ctx.draw_button(
                Rect { x: 24.0, y: text_y + 108.0, width: 100.0, height: 36.0 },
                "Show more",
            );
        }

        text_y + description_rect.height + 16.0
    }

    fn draw_information(&self, ctx: &mut RenderContext, y: f32) -> f32 {
        // Section header
        ctx.draw_text(
            Rect { x: 24.0, y, width: 200.0, height: 24.0 },
            "Information",
            theme.typography.h4,
            theme.colors.on_surface,
        );

        let info_y = y + 32.0;

        // Info grid
        let mut x = 24.0;
        let mut current_y = info_y;
        let row_height = 32.0;

        // Version
        ctx.draw_text(Rect { x, y: current_y, width: 100.0, height: 20.0 }, "Version:", theme.typography.body, theme.colors.on_surface_variant);
        ctx.draw_text(Rect { x: x + 120.0, y: current_y, width: 200.0, height: 20.0 }, &self.app.version, theme.typography.body, theme.colors.on_surface);
        current_y += row_height;

        // Size
        let size = format_size(self.app.size);
        ctx.draw_text(Rect { x, y: current_y, width: 100.0, height: 20.0 }, "Size:", theme.typography.body, theme.colors.on_surface_variant);
        ctx.draw_text(Rect { x: x + 120.0, y: current_y, width: 200.0, height: 20.0 }, &size, theme.typography.body, theme.colors.on_surface);
        current_y += row_height;

        // Developer
        ctx.draw_text(Rect { x, y: current_y, width: 100.0, height: 20.0 }, "Developer:", theme.typography.body, theme.colors.on_surface_variant);
        ctx.draw_text(Rect { x: x + 120.0, y: current_y, width: 200.0, height: 20.0 }, &self.app.developer, theme.typography.body, theme.colors.on_surface);
        current_y += row_height;

        // Category
        ctx.draw_text(Rect { x, y: current_y, width: 100.0, height: 20.0 }, "Category:", theme.typography.body, theme.colors.on_surface_variant);
        ctx.draw_text(Rect { x: x + 120.0, y: current_y, width: 200.0, height: 20.0 }, &self.app.category.name, theme.typography.body, theme.colors.on_surface);
        current_y += row_height;

        // Website
        ctx.draw_text(Rect { x, y: current_y, width: 100.0, height: 20.0 }, "Website:", theme.typography.body, theme.colors.on_surface_variant);
        ctx.draw_text(Rect { x: x + 120.0, y: current_y, width: 300.0, height: 20.0 }, &self.app.website, theme.typography.body, theme.colors.primary);

        current_y + 16.0
    }

    fn draw_reviews(&self, ctx: &mut RenderContext, y: f32) {
        // Section header
        ctx.draw_text(
            Rect { x: 24.0, y, width: 200.0, height: 24.0 },
            "Reviews",
            theme.typography.h4,
            theme.colors.on_surface,
        );

        let mut review_y = y + 32.0;

        // Show first 3 reviews
        for review in self.app.reviews.iter().take(3) {
            review_y = self.draw_review(ctx, review_y, review);
            review_y += 16.0;
        }

        // "See all reviews" button
        if !self.app.reviews.is_empty() {
            ctx.draw_button(
                Rect { x: 24.0, y: review_y, width: 140.0, height: 36.0 },
                &format!("See all {} reviews", self.app.reviews.len()),
            );
        }
    }

    fn draw_review(&self, ctx: &mut RenderContext, y: f32, review: &Review) -> f32 {
        let card_rect = Rect {
            x: 24.0,
            y,
            width: 950.0,
            height: 80.0,
        };

        // Background
        ctx.fill_rounded_rect(card_rect, 8.0, theme.colors.surface);

        // Rating stars
        let mut star_x = card_rect.x + 16.0;
        for _ in 0..review.rating {
            ctx.draw_icon(
                Rect { x: star_x, y: card_rect.y + 12.0, width: 16.0, height: 16.0 },
                "star-filled",
            );
            star_x += 18.0;
        }

        // Review text
        if let Some(ref title) = review.title {
            ctx.draw_text(
                Rect { x: card_rect.x + 16.0, y: card_rect.y + 36.0, width: 600.0, height: 20.0 },
                title,
                theme.typography.h4,
                theme.colors.on_surface,
            );
        }

        ctx.draw_text_wrapped(
            Rect { x: card_rect.x + 16.0, y: card_rect.y + 56.0, width: 600.0, height: 20.0 },
            &review.content,
            theme.typography.sm,
            theme.colors.on_surface_variant,
        );

        // Author and date
        let date_str = format!("{} - {}", review.author, format_date_ago(review.date));
        ctx.draw_text(
            Rect { x: card_rect.x + card_rect.width - 200.0, y: card_rect.y + 12.0, width: 180.0, height: 16.0 },
            &date_str,
            theme.typography.caption,
            theme.colors.on_surface_variant,
        );

        y + 88.0
    }
}
```

## Package Manager Integration

```rust
impl AppLibrary {
    pub fn install_app(&mut self, app_id: String) -> Result<(), Error> {
        // Get app entry
        let app = self.registry.get_app(&app_id)?;

        // Update state
        self.registry.update_state(
            &app_id,
            AppState::Installing(InstallationProgress {
                downloaded: 0,
                total: app.download_size,
                operation: "Starting...".into(),
                progress: 0,
            }),
        );

        // Start installation
        match app.package_type {
            PackageType::Native => {
                self.package_manager.install_native(&app_id)?;
            }

            PackageType::Flatpak => {
                self.package_manager.install_flatpak(&app_id)?;
            }

            _ => {
                return Err(Error::UnsupportedPackageType);
            }
        }

        Ok(())
    }

    pub fn uninstall_app(&mut self, app_id: String) -> Result<(), Error> {
        self.package_manager.uninstall(&app_id)?;
        self.registry.update_state(&app_id, AppState::Available);
        Ok(())
    }

    pub fn update_app(&mut self, app_id: String) -> Result<(), Error> {
        self.package_manager.update(&app_id)?;
        Ok(())
    }

    pub fn launch_app(&self, app_id: String) -> Result<(), Error> {
        // Get app entry
        let app = self.registry.get_app(&app_id)?;

        // Launch using desktop file
        let desktop_file = format!(
            "/usr/share/applications/{}.desktop",
            app_id.replace("/", "-")
        );

        std::process::Command::new("gtk-launch")
            .arg(&app_id)
            .spawn()?;

        Ok(())
    }
}
```

## File Structure

```
/var/www/rustux.com/prod/apps/gui/
├── rustica-applibrary/
│   ├── Cargo.toml
│   ├── resources/
│   │   └── icons/
│   └── src/
│       ├── main.rs
│       ├── app.rs
│       ├── pages/
│       │   ├── mod.rs
│       │   ├── home.rs
│       │   ├── details.rs
│       │   ├── installed.rs
│       │   ├── updates.rs
│       │   ├── search.rs
│       │   └── settings.rs
│       ├── registry.rs
│       ├── cache.rs
│       ├── package_manager.rs
│       ├── download.rs
│       └── widgets/
│           ├── mod.rs
│           ├── app_card.rs
│           ├── category_card.rs
│           └── review_card.rs
```

## Dependencies

```toml
[package]
name = "rustica-applibrary"
version = "1.0.0"
edition = "2021"

[dependencies]
# GUI framework
librustica = { path = "../../../libs/librustica" }

# Package manager
rustica-pm = { path = "../../package-manager/rustica-pm" }

# Image handling
image = "0.24"

# HTTP (for downloading apps, screenshots)
reqwest = { version = "0.11", features = ["json"] }

# Async runtime
tokio = { version = "1.0", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# XDG
dirs = "5.0"

# Date/time
chrono = "0.4"
```

## Performance Requirements

| Metric | Target | Measurement |
|--------|--------|-------------|
| Page load | <500ms | Home page display |
| Search | <200ms | Query to results |
| App details | <300ms | Click to display |
| Install start | <1s | Click to progress |
| Cache refresh | <5s | Background update |
| Memory | <150MB | Base usage |

## Success Criteria

- [ ] Home page displays correctly
- [ ] Search works
- [ ] Categories filter correctly
- [ ] App details page works
- [ ] Installation works
- [ ] Updates work
- [ ] Screenshots display
- [ ] Reviews display
- [ ] Caching works
- [ ] Full accessibility
- [ ] Performance targets met

## Sign-Off

**Architect**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅

---

## Timeline

- Week 1: Basic app structure + home page
- Week 2: App details page + navigation
- Week 3: Package manager integration
- Week 4: Installation + progress tracking
- Week 5: Search + categories
- Week 6: Cache + screenshots
- Week 7: Updates + installed page
- Week 8: Reviews + polish

**Total**: 8 weeks
