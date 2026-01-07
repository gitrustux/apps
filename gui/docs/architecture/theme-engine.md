# Theme Engine Specification

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Component**: Rustica Shell - Theme Engine
**Phase**: 7.2 - Integration & Polish

## Overview

The Theme Engine provides a **comprehensive theming system** for Rustica Shell with **Material Design 3-inspired color system**, **dark/light modes**, **accent color customization**, **dynamic theming**, **wallpaper-based colors**, and **real-time theme switching**. It serves as the **central theming authority** for all applications.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         Theme Engine                                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌───────────────┐  │
│  │ Color       │  │ Typography  │  │ Spacing     │  │ Animation      │  │
│  │ System      │  │ System      │  │ System      │  │ System         │  │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └───────┬───────┘  │
│         │                │                │                  │          │
│         ▼                ▼                ▼                  ▼          │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                      Theme Definition                            │  │
│  │  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐   │  │
│  │  │ Palette    │ │ Fonts      │ │ Sizes      │ │ Easings    │   │  │
│  │  └────────────┘ └────────────┘ └────────────┘ └────────────┘   │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                    │                                 │
│                                    ▼                                 │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                    Theme Provider                                │  │
│  │  - Loads themes                                                  │  │
│  │  - Caches resources                                              │  │
│  │  - Broadcasts changes                                            │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                    │                                 │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
        ┌───────────────────────────┴───────────────────────────┐
        │                                                       │
┌───────┴───────┐                                       ┌───────┴───────┐
│ Applications  │                                       │  Shell        │
│ (librustica)  │                                       │  Components   │
└───────────────┘                                       └───────────────┘
```

## Color System

```rust
pub struct ColorScheme {
    /// Primary color (brand color)
    pub primary: Color,

    /// On-primary color (text/icons on primary)
    pub on_primary: Color,

    /// Primary container color
    pub primary_container: Color,

    /// On-primary container color
    pub on_primary_container: Color,

    /// Secondary color
    pub secondary: Color,

    /// On-secondary color
    pub on_secondary: Color,

    /// Secondary container color
    pub secondary_container: Color,

    /// On-secondary container color
    pub on_secondary_container: Color,

    /// Tertiary color
    pub tertiary: Color,

    /// On-tertiary color
    pub on_tertiary: Color,

    /// Tertiary container color
    pub tertiary_container: Color,

    /// On-tertiary container color
    pub on_tertiary_container: Color,

    /// Error color
    pub error: Color,

    /// On-error color
    pub on_error: Color,

    /// Error container color
    pub error_container: Color,

    /// On-error container color
    pub on_error_container: Color,

    /// Background color
    pub background: Color,

    /// On-background color
    pub on_background: Color,

    /// Surface color
    pub surface: Color,

    /// On-surface color
    pub on_surface: Color,

    /// Surface variant color
    pub surface_variant: Color,

    /// On-surface-variant color
    pub on_surface_variant: Color,

    /// Outline color
    pub outline: Color,

    /// Outline variant color
    pub outline_variant: Color,

    /// Scrim color (for overlays)
    pub scrim: Color,

    /// Shadow color
    pub shadow: Color,

    /// Inverse surface color
    pub inverse_surface: Color,

    /// Inverse on-surface color
    pub inverse_on_surface: Color,

    /// Inverse primary color
    pub inverse_primary: Color,
}

#[derive(Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    pub fn to_rgba_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
    }

    pub fn to_f32(&self) -> (f32, f32, f32, f32) {
        (
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        )
    }

    pub fn with_alpha(&self, alpha: u8) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a: alpha,
        }
    }

    pub fn lighten(&self, amount: f32) -> Self {
        // Convert to HSL, lighten, convert back
        let (h, s, l) = self.to_hsl();
        Self::from_hsl(h, s, (l + amount).min(1.0))
    }

    pub fn darken(&self, amount: f32) -> Self {
        let (h, s, l) = self.to_hsl();
        Self::from_hsl(h, s, (l - amount).max(0.0))
    }

    fn to_hsl(&self) -> (f32, f32, f32) {
        let (r, g, b) = (self.r as f32 / 255.0, self.g as f32 / 255.0, self.b as f32 / 255.0);

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let l = (max + min) / 2.0;

        let h = if delta == 0.0 {
            0.0
        } else if max == r {
            60.0 * (((g - b) / delta).rem_euclid(6.0))
        } else if max == g {
            60.0 * (((b - r) / delta + 2.0).rem_euclid(6.0))
        } else {
            60.0 * (((r - g) / delta + 4.0).rem_euclid(6.0))
        };

        let s = if delta == 0.0 {
            0.0
        } else {
            delta / (1.0 - (2.0 * l - 1.0).abs())
        };

        (h, s, l)
    }

    fn from_hsl(h: f32, s: f32, l: f32) -> Self {
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0).rem_euclid(2.0) - 1.0).abs());
        let m = l - c / 2.0;

        let (r, g, b) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        Self {
            r: ((r + m) * 255.0) as u8,
            g: ((g + m) * 255.0) as u8,
            b: ((b + m) * 255.0) as u8,
            a: 255,
        }
    }
}
```

## Material Design 3 Color Generation

```rust
pub struct MaterialColorGenerator {
    /// Source color (accent color)
    source: Color,
}

impl MaterialColorGenerator {
    pub fn new(source: Color) -> Self {
        Self { source }
    }

    /// Generate complete color scheme from source color
    pub fn generate(&self, dark: bool) -> ColorScheme {
        // Generate key colors from source
        let primary = self.generate_primary();
        let secondary = self.generate_secondary();
        let tertiary = self.generate_tertiary();
        let error = self.generate_error();
        let neutral = self.generate_neutral();

        // Generate tonal palettes
        let primary_palette = self.generate_tonal_palette(primary);
        let secondary_palette = self.generate_tonal_palette(secondary);
        let tertiary_palette = self.generate_tonal_palette(tertiary);
        let neutral_palette = self.generate_tonal_palette(neutral);
        let error_palette = self.generate_tonal_palette(error);

        if dark {
            self.generate_dark_scheme(
                primary_palette,
                secondary_palette,
                tertiary_palette,
                neutral_palette,
                error_palette,
            )
        } else {
            self.generate_light_scheme(
                primary_palette,
                secondary_palette,
                tertiary_palette,
                neutral_palette,
                error_palette,
            )
        }
    }

    fn generate_primary(&self) -> Color {
        self.source
    }

    fn generate_secondary(&self) -> Color {
        // Generate secondary from source with hue shift
        let (h, s, l) = self.source.to_hsl();
        Color::from_hsl((h + 60.0) % 360.0, s * 0.7, l)
    }

    fn generate_tertiary(&self) -> Color {
        // Generate tertiary from source with hue shift
        let (h, s, l) = self.source.to_hsl();
        Color::from_hsl((h + 120.0) % 360.0, s * 0.6, l)
    }

    fn generate_error(&self) -> Color {
        // Error color (typically red)
        Color::rgb(196, 40, 40)
    }

    fn generate_neutral(&self) -> Color {
        // Generate neutral from source (desaturated)
        let (h, _s, l) = self.source.to_hsl();
        Color::from_hsl(h, 0.02, l)
    }

    fn generate_tonal_palette(&self, base: Color) -> TonalPalette {
        // Generate 13 tones (0-100 in steps of 10)
        let mut tones = [Color::rgb(0, 0, 0); 13];

        for (i, tone) in tones.iter_mut().enumerate() {
            let tone_value = (i * 10) as f32;

            // Calculate chroma and tone based on base color
            let (h, s, _l) = base.to_hsl();

            // Adjust lightness for tone
            let lightness = if tone_value < 50.0 {
                (tone_value / 50.0) * 0.5
            } else {
                0.5 + ((tone_value - 50.0) / 50.0) * 0.5
            };

            // Adjust chroma (darker tones have less chroma)
            let chroma = s * (1.0 - (tone_value - 50.0).abs() / 100.0);

            *tone = Color::from_hsl(h, chroma, lightness);
        }

        TonalPalette { tones }
    }

    fn generate_light_scheme(
        &self,
        primary: TonalPalette,
        secondary: TonalPalette,
        tertiary: TonalPalette,
        neutral: TonalPalette,
        error: TonalPalette,
    ) -> ColorScheme {
        ColorScheme {
            primary: primary.tones[6],          // 40
            on_primary: neutral.tones[10],       // 100
            primary_container: primary.tones[9], // 90
            on_primary_container: primary.tones[0], // 0

            secondary: secondary.tones[6],
            on_secondary: neutral.tones[10],
            secondary_container: secondary.tones[9],
            on_secondary_container: secondary.tones[0],

            tertiary: tertiary.tones[6],
            on_tertiary: neutral.tones[10],
            tertiary_container: tertiary.tones[9],
            on_tertiary_container: tertiary.tones[0],

            error: error.tones[6],
            on_error: neutral.tones[10],
            error_container: error.tones[9],
            on_error_container: error.tones[0],

            background: neutral.tones[10],      // 100
            on_background: neutral.tones[2],     // 10
            surface: neutral.tones[10],          // 100
            on_surface: neutral.tones[2],        // 10
            surface_variant: neutral.tones[9],   // 90
            on_surface_variant: neutral.tones[3], // 30
            outline: neutral.tones[6],           // 40
            outline_variant: secondary.tones[7], // 50

            scrim: Color::rgba(0, 0, 0, 180),
            shadow: Color::rgba(0, 0, 0, 40),
            inverse_surface: neutral.tones[2],
            inverse_on_surface: neutral.tones[10],
            inverse_primary: primary.tones[8],
        }
    }

    fn generate_dark_scheme(
        &self,
        primary: TonalPalette,
        secondary: TonalPalette,
        tertiary: TonalPalette,
        neutral: TonalPalette,
        error: TonalPalette,
    ) -> ColorScheme {
        ColorScheme {
            primary: primary.tones[8],          // 80
            on_primary: primary.tones[0],        // 0
            primary_container: primary.tones[3], // 30
            on_primary_container: primary.tones[10], // 100

            secondary: secondary.tones[8],
            on_secondary: secondary.tones[0],
            secondary_container: secondary.tones[3],
            on_secondary_container: secondary.tones[10],

            tertiary: tertiary.tones[8],
            on_tertiary: tertiary.tones[0],
            tertiary_container: tertiary.tones[3],
            on_tertiary_container: tertiary.tones[10],

            error: error.tones[8],
            on_error: error.tones[0],
            error_container: error.tones[3],
            on_error_container: error.tones[10],

            background: neutral.tones[2],       // 10
            on_background: neutral.tones[11],    // 100
            surface: neutral.tones[2],           // 10
            on_surface: neutral.tones[11],       // 100
            surface_variant: neutral.tones[3],   // 30
            on_surface_variant: neutral.tones[9], // 90
            outline: neutral.tones[7],           // 50
            outline_variant: neutral.tones[6],   // 40

            scrim: Color::rgba(0, 0, 0, 180),
            shadow: Color::rgba(0, 0, 0, 60),
            inverse_surface: neutral.tones[11],
            inverse_on_surface: neutral.tones[2],
            inverse_primary: primary.tones[4],
        }
    }
}

pub struct TonalPalette {
    pub tones: [Color; 13], // 0, 10, 20, ..., 120
}
```

## Wallpaper-Based Theme Generation

```rust
pub struct WallpaperThemeGenerator {
    /// Color extractor
    extractor: ColorExtractor,

    /// Material color generator
    material: MaterialColorGenerator,
}

impl WallpaperThemeGenerator {
    /// Generate theme from wallpaper image
    pub fn generate_from_wallpaper(&self, wallpaper: &Image, dark: bool) -> ColorScheme {
        // Extract dominant colors from wallpaper
        let colors = self.extractor.extract_colors(wallpaper, 5);

        // Use dominant color as source
        let source = colors[0];

        // Generate color scheme
        let generator = MaterialColorGenerator::new(source);
        generator.generate(dark)
    }

    /// Generate theme with user-selected accent
    pub fn generate_with_accent(&self, wallpaper: &Image, accent: Color, dark: bool) -> ColorScheme {
        // Extract neutral tones from wallpaper
        let colors = self.extractor.extract_colors(wallpaper, 5);
        let neutral_source = colors[0]; // Most dominant for neutrals

        // Generate base scheme with accent
        let generator = MaterialColorGenerator::new(accent);
        let mut scheme = generator.generate(dark);

        // Override neutral colors with wallpaper-based colors
        let neutral_palette = self.material.generate_tonal_palette(neutral_source);

        if dark {
            scheme.background = neutral_palette.tones[2];
            scheme.on_background = neutral_palette.tones[11];
            scheme.surface = neutral_palette.tones[2];
            scheme.on_surface = neutral_palette.tones[11];
        } else {
            scheme.background = neutral_palette.tones[10];
            scheme.on_background = neutral_palette.tones[2];
            scheme.surface = neutral_palette.tones[10];
            scheme.on_surface = neutral_palette.tones[2];
        }

        scheme
    }
}

pub struct ColorExtractor {
    /// Quantization quality
    quality: u8,
}

impl ColorExtractor {
    /// Extract dominant colors from image
    pub fn extract_colors(&self, image: &Image, count: usize) -> Vec<Color> {
        // Sample pixels from image
        let samples = self.sample_pixels(image, 1000);

        // Quantize colors
        let quantized = self.quantize_colors(&samples, count * 5);

        // Cluster by color similarity
        let clusters = self.cluster_colors(&quantized, count);

        // Sort by frequency
        let mut colors: Vec<_> = clusters.into_iter()
            .map(|(color, _)| color)
            .collect();

        colors.sort_by(|a, b| {
            let brightness_a = self.brightness(a);
            let brightness_b = self.brightness(b);
            brightness_b.partial_cmp(&brightness_a).unwrap()
        });

        colors
    }

    fn sample_pixels(&self, image: &Image, max_samples: usize) -> Vec<Color> {
        let mut samples = Vec::new();
        let width = image.width();
        let height = image.height();

        // Sample at regular intervals
        let step = ((width * height) / max_samples as u32).max(1);

        for y in (0..height).step_by(step as usize) {
            for x in (0..width).step_by(step as usize) {
                samples.push(image.get_pixel(x, y));
            }
        }

        samples
    }

    fn quantize_colors(&self, colors: &[Color], max_colors: usize) -> Vec<Color> {
        // Simple k-means clustering
        let mut centroids: Vec<Color> = colors.iter()
            .take(max_colors)
            .cloned()
            .collect();

        for _ in 0..10 {
            // Assign each color to nearest centroid
            let mut clusters: Vec<Vec<Color>> = vec![Vec::new(); centroids.len()];

            for color in colors {
                let nearest = self.find_nearest_centroid(color, &centroids);
                clusters[nearest].push(*color);
            }

            // Update centroids
            for (cluster, centroid) in clusters.iter().zip(centroids.iter_mut()) {
                if !cluster.is_empty() {
                    *centroid = self.average_color(cluster);
                }
            }
        }

        centroids
    }

    fn cluster_colors(&self, colors: &[Color], count: usize) -> Vec<(Color, usize)> {
        // Group similar colors
        let mut clusters = Vec::new();

        for color in colors {
            let found = clusters.iter_mut()
                .find(|(cluster_color, _)| self.color_distance(color, cluster_color) < 30.0);

            if let Some((_, ref mut count)) = found {
                *count += 1;
            } else if clusters.len() < count {
                clusters.push((*color, 1));
            }
        }

        clusters
    }

    fn find_nearest_centroid(&self, color: &Color, centroids: &[Color]) -> usize {
        centroids.iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                self.color_distance(color, a)
                    .partial_cmp(&self.color_distance(color, b))
                    .unwrap()
            })
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    fn color_distance(&self, a: &Color, b: &Color) -> f32 {
        let (r1, g1, b1) = (a.r as f32, a.g as f32, a.b as f32);
        let (r2, g2, b2) = (b.r as f32, b.g as f32, b.b as f32);

        ((r1 - r2).powi(2) + (g1 - g2).powi(2) + (b1 - b2).powi(2)).sqrt()
    }

    fn brightness(&self, color: &Color) -> f32 {
        let (r, g, b) = (color.r as f32, color.g as f32, color.b as f32);
        (r * 0.299 + g * 0.587 + b * 0.114) / 255.0
    }

    fn average_color(&self, colors: &[Color]) -> Color {
        let r = colors.iter().map(|c| c.r as u32).sum::<u32>() / colors.len() as u32;
        let g = colors.iter().map(|c| c.g as u32).sum::<u32>() / colors.len() as u32;
        let b = colors.iter().map(|c| c.b as u32).sum::<u32>() / colors.len() as u32;

        Color {
            r: r as u8,
            g: g as u8,
            b: b as u8,
            a: 255,
        }
    }
}
```

## Typography System

```rust
pub struct TypographySystem {
    /// Font families
    pub families: FontFamilies,

    /// Font sizes (in points)
    pub sizes: FontSizes,

    /// Font weights
    pub weights: FontWeights,

    /// Line heights
    pub line_heights: LineHeights,
}

pub struct FontFamilies {
    /// Display font (headings)
    pub display: String,

    /// Body font
    pub body: String,

    /// Monospace font
    pub mono: String,
}

pub struct FontSizes {
    /// Display large
    pub display_large: f32,   // 57pt

    /// Display medium
    pub display_medium: f32,  // 45pt

    /// Display small
    pub display_small: f32,   // 36pt

    /// Headline large
    pub headline_large: f32,  // 32pt

    /// Headline medium
    pub headline_medium: f32, // 28pt

    /// Headline small
    pub headline_small: f32,  // 24pt

    /// Title large
    pub title_large: f32,     // 22pt

    /// Title medium
    pub title_medium: f32,    // 16pt

    /// Title small
    pub title_small: f32,     // 14pt

    /// Body large
    pub body_large: f32,      // 16pt

    /// Body medium
    pub body_medium: f32,     // 14pt

    /// Body small
    pub body_small: f32,      // 12pt

    /// Label large
    pub label_large: f32,     // 14pt

    /// Label medium
    pub label_medium: f32,    // 12pt

    /// Label small
    pub label_small: f32,     // 11pt
}

pub struct FontWeights {
    pub thin: u16,      // 100
    pub extralight: u16, // 200
    pub light: u16,     // 300
    pub regular: u16,   // 400
    pub medium: u16,    // 500
    pub semibold: u16,  // 600
    pub bold: u16,      // 700
    pub extrabold: u16, // 800
    pub black: u16,     // 900
}

pub struct LineHeights {
    pub display: f32,   // 64pt
    pub headline: f32,  // 40pt
    pub title: f32,     // 28pt
    pub body: f32,      // 20pt
    pub label: f32,     // 16pt
}

impl TypographySystem {
    pub fn default() -> Self {
        Self {
            families: FontFamilies {
                display: "Roboto".into(),
                body: "Roboto".into(),
                mono: "JetBrains Mono".into(),
            },
            sizes: FontSizes {
                display_large: 57.0,
                display_medium: 45.0,
                display_small: 36.0,
                headline_large: 32.0,
                headline_medium: 28.0,
                headline_small: 24.0,
                title_large: 22.0,
                title_medium: 16.0,
                title_small: 14.0,
                body_large: 16.0,
                body_medium: 14.0,
                body_small: 12.0,
                label_large: 14.0,
                label_medium: 12.0,
                label_small: 11.0,
            },
            weights: FontWeights {
                thin: 100,
                extralight: 200,
                light: 300,
                regular: 400,
                medium: 500,
                semibold: 600,
                bold: 700,
                extrabold: 800,
                black: 900,
            },
            line_heights: LineHeights {
                display: 64.0,
                headline: 40.0,
                title: 28.0,
                body: 20.0,
                label: 16.0,
            },
        }
    }
}
```

## Spacing System

```rust
pub struct SpacingSystem {
    /// Base spacing unit (4px grid)
    pub base: f32,

    /// Spacing scale
    pub scale: SpacingScale,
}

pub struct SpacingScale {
    /// 0px
    pub none: f32,

    /// 4px
    pub xs: f32,

    /// 8px
    pub sm: f32,

    /// 12px
    pub md: f32,

    /// 16px
    pub lg: f32,

    /// 20px
    pub xl: f32,

    /// 24px
    pub xxl: f32,

    /// 32px
    pub xxxl: f32,
}

impl SpacingSystem {
    pub fn new(base: f32) -> Self {
        Self {
            base,
            scale: SpacingScale {
                none: 0.0,
                xs: base,
                sm: base * 2.0,
                md: base * 3.0,
                lg: base * 4.0,
                xl: base * 5.0,
                xxl: base * 6.0,
                xxxl: base * 8.0,
            },
        }
    }

    pub fn default() -> Self {
        Self::new(4.0)
    }
}
```

## Animation System

```rust
pub struct AnimationSystem {
    /// Durations
    pub durations: AnimationDurations,

    /// Easing functions
    pub easings: AnimationEasings,
}

pub struct AnimationDurations {
    /// 50ms - Micro-interaction
    pub instant: Duration,

    /// 100ms - Quick feedback
    pub quick: Duration,

    /// 200ms - Standard
    pub standard: Duration,

    /// 300ms - Deliberate
    pub deliberate: Duration,

    /// 400ms - Slow
    pub slow: Duration,
}

pub struct AnimationEasings {
    /// Linear
    pub linear: EasingFunction,

    /// Ease in
    pub ease_in: EasingFunction,

    /// Ease out
    pub ease_out: EasingFunction,

    /// Ease in-out
    pub ease_in_out: EasingFunction,

    /// Standard (material design)
    pub standard: EasingFunction,

    /// Emphasized (material design)
    pub emphasized: EasingFunction,

    /// Decelerated (material design)
    pub decelerated: EasingFunction,

    /// Accelerated (material design)
    pub accelerated: EasingFunction,
}

pub enum EasingFunction {
    Linear,
    CubicBezier(f32, f32, f32, f32),
}

impl AnimationSystem {
    pub fn default() -> Self {
        Self {
            durations: AnimationDurations {
                instant: Duration::from_millis(50),
                quick: Duration::from_millis(100),
                standard: Duration::from_millis(200),
                deliberate: Duration::from_millis(300),
                slow: Duration::from_millis(400),
            },
            easings: AnimationEasings {
                linear: EasingFunction::Linear,
                ease_in: EasingFunction::CubicBezier(0.42, 0.0, 1.0, 1.0),
                ease_out: EasingFunction::CubicBezier(0.0, 0.0, 0.58, 1.0),
                ease_in_out: EasingFunction::CubicBezier(0.42, 0.0, 0.58, 1.0),
                standard: EasingFunction::CubicBezier(0.2, 0.0, 0.0, 1.0),
                emphasized: EasingFunction::CubicBezier(0.0, 0.0, 0.0, 1.0),
                decelerated: EasingFunction::CubicBezier(0.0, 0.0, 0.0, 1.0),
                accelerated: EasingFunction::CubicBezier(0.3, 0.0, 0.8, 0.15),
            },
        }
    }

    pub fn apply_easing(&self, easing: &EasingFunction, t: f32) -> f32 {
        match easing {
            EasingFunction::Linear => t,

            EasingFunction::CubicBezier(x1, y1, x2, y2) => {
                // Solve cubic bezier for t
                self.cubic_bezier(t, *x1, *y1, *x2, *y2)
            }
        }
    }

    fn cubic_bezier(&self, t: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
        // Simplified cubic bezier solver
        let u = 1.0 - t;
        let tt = t * t;
        let uu = u * u;
        let uuu = uu * u;
        let ttt = tt * t;

        // Y-value
        3.0 * uu * t * y1 + 3.0 * u * tt * y2 + ttt
    }
}
```

## Shadow System

```rust
pub struct ShadowSystem {
    /// Shadow elevations
    pub shadows: Shadows,
}

pub struct Shadows {
    /// No shadow
    pub none: Shadow,

    /// 1dp
    pub xs: Shadow,

    /// 2dp
    pub sm: Shadow,

    /// 3dp
    pub md: Shadow,

    /// 8dp
    pub lg: Shadow,

    /// 12dp
    pub xl: Shadow,

    /// 16dp
    pub xxl: Shadow,
}

pub struct Shadow {
    /// X offset
    pub x: f32,

    /// Y offset
    pub y: f32,

    /// Blur radius
    pub blur: f32,

    /// Spread radius
    pub spread: f32,

    /// Color
    pub color: Color,
}

impl ShadowSystem {
    pub fn new(scheme: &ColorScheme) -> Self {
        let shadow_color = if scheme.background.r < 128 {
            Color::rgba(0, 0, 0, 50)
        } else {
            Color::rgba(0, 0, 0, 20)
        };

        Self {
            shadows: Shadows {
                none: Shadow { x: 0.0, y: 0.0, blur: 0.0, spread: 0.0, color: Color::rgba(0, 0, 0, 0) },
                xs: Shadow { x: 0.0, y: 1.0, blur: 2.0, spread: 0.0, color: shadow_color },
                sm: Shadow { x: 0.0, y: 2.0, blur: 4.0, spread: 0.0, color: shadow_color },
                md: Shadow { x: 0.0, y: 4.0, blur: 8.0, spread: 0.0, color: shadow_color },
                lg: Shadow { x: 0.0, y: 8.0, blur: 16.0, spread: 0.0, color: shadow_color },
                xl: Shadow { x: 0.0, y: 12.0, blur: 24.0, spread: 0.0, color: shadow_color },
                xxl: Shadow { x: 0.0, y: 16.0, blur: 32.0, spread: 0.0, color: shadow_color },
            },
        }
    }
}
```

## Complete Theme

```rust
pub struct Theme {
    /// Name
    pub name: String,

    /// Color scheme
    pub colors: ColorScheme,

    /// Typography
    pub typography: TypographySystem,

    /// Spacing
    pub spacing: SpacingSystem,

    /// Animation
    pub animation: AnimationSystem,

    /// Shadows
    pub shadows: ShadowSystem,

    /// Border radius
    pub border_radius: BorderRadius,
}

pub struct BorderRadius {
    /// None
    pub none: f32,

    /// Extra small (4px)
    pub xs: f32,

    /// Small (8px)
    pub sm: f32,

    /// Medium (12px)
    pub md: f32,

    /// Large (16px)
    pub lg: f32,

    /// Extra large (24px)
    pub xl: f32,

    /// Full (50%)
    pub full: f32,
}

impl Theme {
    /// Generate theme from accent color
    pub fn from_accent(accent: Color, dark: bool) -> Self {
        let generator = MaterialColorGenerator::new(accent);
        let colors = generator.generate(dark);

        Self {
            name: format!("{} ({})", if dark { "Dark" } else { "Light" }, accent.to_hex()),
            colors,
            typography: TypographySystem::default(),
            spacing: SpacingSystem::default(),
            animation: AnimationSystem::default(),
            shadows: ShadowSystem::new(&colors),
            border_radius: BorderRadius::default(),
        }
    }

    /// Generate theme from wallpaper
    pub fn from_wallpaper(wallpaper: &Image, accent: Color, dark: bool) -> Self {
        let generator = WallpaperThemeGenerator {
            extractor: ColorExtractor { quality: 90 },
            material: MaterialColorGenerator::new(accent),
        };

        let colors = generator.generate_with_accent(wallpaper, accent, dark);

        Self {
            name: format!("Wallpaper ({})", if dark { "Dark" } else { "Light" }),
            colors,
            typography: TypographySystem::default(),
            spacing: SpacingSystem::default(),
            animation: AnimationSystem::default(),
            shadows: ShadowSystem::new(&colors),
            border_radius: BorderRadius::default(),
        }
    }

    /// Save theme to file
    pub fn save(&self, path: &Path) -> Result<(), Error> {
        let theme_data = serde_json::to_string_pretty(self)?;
        std::fs::write(path, theme_data)?;
        Ok(())
    }

    /// Load theme from file
    pub fn load(path: &Path) -> Result<Self, Error> {
        let theme_data = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&theme_data)?)
    }
}
```

## File Structure

```
/var/www/rustux.com/prod/apps/gui/
├── rustica-theme/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── color.rs
│       ├── material.rs
│       ├── wallpaper.rs
│       ├── typography.rs
│       ├── spacing.rs
│       ├── animation.rs
│       ├── shadow.rs
│       └── theme.rs
└── themes/
    ├── rustica-dark.toml
    ├── rustica-light.toml
    └── custom/
        └── user-theme.toml
```

## Dependencies

```toml
[package]
name = "rustica-theme"
version = "1.0.0"
edition = "2021"

[dependencies]
# Image processing
image = "0.24"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Logging
tracing = "0.1"

# XDG
dirs = "5.0"
```

## Success Criteria

- [ ] Color system works
- [ ] Material Design 3 generation works
- [ ] Wallpaper-based generation works
- [ ] Typography system works
- [ ] Spacing system works
- [ ] Animation system works
- [ ] Shadow system works
- [ ] Theme serialization works
- [ ] Real-time theme switching works
- [ ] WCAG AA contrast ratios met

## Sign-Off

**Architect**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅

---

## Timeline

- Week 1: Color system + Material generation
- Week 2: Wallpaper-based generation
- Week 3: Typography + spacing systems
- Week 4: Animation + shadow systems
- Week 5: Theme provider + caching
- Week 6: D-Bus integration + live reload

**Total**: 6 weeks
