# Phase 8.3: Mobile-Optimized UI Components (librustica-mobile)

## Overview

**Component**: librustica-mobile
**Purpose**: Mobile-specific UI components built on librustica foundation
**Language**: Rust
**Dependencies**: librustica, rustica-gestures, rustica-keyboard

## Goals

1. **Touch-First Design**: All components optimized for touch interaction
2. **Responsive Layouts**: Adapt to different screen sizes and orientations
3. **Gesture Support**: Integrate with gesture system for rich interactions
4. **Accessibility**: WCAG AAA compliant with mobile considerations
5. **Performance**: 60fps animations and smooth scrolling

## Component Library

### 1. BottomSheet

A modal panel that slides up from the bottom of the screen.

```rust
use librustica::{Widget, Container, Property};
use rustica_gestures::{Gesture, SwipeDirection};

/// Bottom sheet component
pub struct BottomSheet {
    content: Box<dyn Widget>,
    state: SheetState,
    height: SheetHeight,
    peek_height: Option<f64>,  // Collapsed height
    drag_handle: bool,
    dismiss_on_swipe: bool,
    on_dismiss: Option<Box<dyn Fn()>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SheetState {
    Hidden,
    Peeking,      // Showing drag handle only
    Expanded,     // Partially expanded
    FullScreen,   // Full height
}

#[derive(Debug, Clone, Copy)]
pub enum SheetHeight {
    Fixed(f64),           // Exact pixels
    Fraction(f64),        // Fraction of screen height (0.0-1.0)
    WrapContent,         // Fit content
    MatchParent,         // Full screen
}

impl BottomSheet {
    pub fn new(content: Box<dyn Widget>) -> Self {
        Self {
            content,
            state: SheetState::Hidden,
            height: SheetHeight::Fraction(0.6),
            peek_height: Some(80.0),
            drag_handle: true,
            dismiss_on_swipe: true,
            on_dismiss: None,
        }
    }

    /// Animate sheet to expanded state
    pub fn expand(&mut self) {
        self.state = SheetState::Expanded;
        self.animate_to_state(SheetState::Expanded);
    }

    /// Animate sheet to peek state
    pub fn peek(&mut self) {
        self.state = SheetState::Peeking;
        self.animate_to_state(SheetState::Peeking);
    }

    /// Dismiss (hide) the sheet
    pub fn dismiss(&mut self) {
        self.animate_to_state(SheetState::Hidden);
        if let Some(on_dismiss) = &self.on_dismiss {
            on_dismiss();
        }
    }

    /// Handle gesture events
    pub fn handle_gesture(&mut self, gesture: &Gesture) -> bool {
        match gesture {
            Gesture::Swipe { direction, .. } if *direction == SwipeDirection::Down => {
                if self.dismiss_on_swipe && self.state != SheetState::Hidden {
                    self.dismiss();
                    return true;
                }
            }

            Gesture::EdgeSwipe { edge, progress } if *edge == ScreenEdge::Bottom => {
                // Swipe up to expand
                if *progress > 0.3 {
                    self.expand();
                }
                return true;
            }

            _ => {}
        }

        false
    }

    fn animate_to_state(&mut self, target: SheetState) {
        // Animate with spring physics
        let animator = SpringAnimator::new(
            self.current_y_position(),
            self.target_y_position(target),
            stiffness: 300.0,
            damping: 25.0,
        );

        animator.start(|y| {
            self.set_y_position(y);
        });
    }

    fn target_y_position(&self, state: SheetState) -> f64 {
        match state {
            SheetState::Hidden => self.screen_height(),
            SheetState::Peeking => {
                self.screen_height() - self.peek_height.unwrap_or(80.0)
            }
            SheetState::Expanded => {
                self.screen_height() - self.actual_height()
            }
            SheetState::FullScreen => 0.0,
        }
    }
}
```

### 2. SwipeableListItem

List items that can be swiped to reveal actions.

```rust
pub struct SwipeableListItem {
    content: Box<dyn Widget>,
    leading_actions: Vec<SwipeAction>,
    trailing_actions: Vec<SwipeAction>,
    swipe_state: SwipeState,
    swipe_offset: f64,
    on_dismiss: Option<Box<dyn Fn()>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwipeAction {
    pub icon: Icon,
    pub label: String,
    pub background: Color,
    pub on_trigger: Box<dyn Fn()>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwipeState {
    Idle,
    Swiping { offset: f64, direction: SwipeDirection },
    ActionRevealed { action: usize, direction: SwipeDirection },
    Dismissing,
}

impl SwipeableListItem {
    pub fn new(content: Box<dyn Widget>) -> Self {
        Self {
            content,
            leading_actions: Vec::new(),
            trailing_actions: Vec::new(),
            swipe_state: SwipeState::Idle,
            swipe_offset: 0.0,
            on_dismiss: None,
        }
    }

    pub fn with_leading_action(mut self, action: SwipeAction) -> Self {
        self.leading_actions.push(action);
        self
    }

    pub fn with_trailing_action(mut self, action: SwipeAction) -> Self {
        self.trailing_actions.push(action);
        self
    }

    pub fn handle_gesture(&mut self, gesture: &Gesture) -> bool {
        match gesture {
            Gesture::Pan { delta, .. } => {
                if matches!(self.swipe_state, SwipeState::Idle) {
                    self.swipe_state = SwipeState::Swiping {
                        offset: delta.x,
                        direction: if delta.x > 0.0 {
                            SwipeDirection::Right
                        } else {
                            SwipeDirection::Left
                        },
                    };
                }

                if let SwipeState::Swiping { offset, direction } = &mut self.swipe_state {
                    *offset += delta.x;

                    // Clamp to action bounds
                    let max_offset = if *direction == SwipeDirection::Right {
                        self.leading_actions.len() as f64 * 80.0
                    } else {
                        self.trailing_actions.len() as f64 * 80.0
                    };

                    *offset = offset.clamp(-max_offset, max_offset);
                    self.swipe_offset = *offset;

                    // Check if action is revealed
                    let action_width = 80.0;
                    let action_index = ((*offset).abs() / action_width) as usize;

                    if (*offset).abs() > action_width * 0.7 {
                        if *direction == SwipeDirection::Right && !self.leading_actions.is_empty() {
                            self.swipe_state = SwipeState::ActionRevealed {
                                action: action_index.min(self.leading_actions.len() - 1),
                                direction: *direction,
                            };
                        } else if !self.trailing_actions.is_empty() {
                            self.swipe_state = SwipeState::ActionRevealed {
                                action: action_index.min(self.trailing_actions.len() - 1),
                                direction: *direction,
                            };
                        }
                    }

                    return true;
                }
            }

            Gesture::Swipe { direction, velocity, .. } => {
                // Complete swipe if velocity is high enough
                if velocity > 500.0 {
                    self.complete_swipe(*direction);
                    return true;
                }
            }

            Gesture::Tap { .. } => {
                if let SwipeState::ActionRevealed { action, direction } = self.swipe_state {
                    self.trigger_action(action, direction);
                    return true;
                }
            }

            _ => {}
        }

        false
    }

    fn complete_swipe(&mut self, direction: SwipeDirection) {
        // Snap to nearest action or reset
        let action_width = 80.0;
        let num_actions = if direction == SwipeDirection::Right {
            self.leading_actions.len()
        } else {
            self.trailing_actions.len()
        };

        if num_actions > 0 {
            let snap_offset = (num_actions as f64) * action_width * direction.sign();
            self.animate_offset_to(snap_offset);
            self.swipe_state = SwipeState::ActionRevealed {
                action: num_actions - 1,
                direction,
            };
        } else {
            self.reset();
        }
    }

    fn trigger_action(&mut self, action: usize, direction: SwipeDirection) {
        let actions = if direction == SwipeDirection::Right {
            &mut self.leading_actions
        } else {
            &mut self.trailing_actions
        };

        if let Some(action) = actions.get(action) {
            (action.on_trigger)();
        }

        self.reset();
    }

    fn reset(&mut self) {
        self.animate_offset_to(0.0);
        self.swipe_state = SwipeState::Idle;
    }

    fn animate_offset_to(&mut self, target: f64) {
        let animator = SpringAnimator::new(
            self.swipe_offset,
            target,
            stiffness: 400.0,
            damping: 30.0,
        );

        animator.start(|offset| {
            self.swipe_offset = offset;
        });
    }
}
```

### 3. RefreshableList

List that pulls down to refresh content.

```rust
pub struct RefreshableList<W: Widget> {
    items: Vec<W>,
    state: RefreshState,
    pull_distance: f64,
    refresh_threshold: f64,
    on_refresh: Option<Box<dyn Fn() -> Box<dyn Future<Output = ()>> + 'static>>,
    refreshing_indicator: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshState {
    Idle,
    Pulling { distance: f64 },
    ReadyToRefresh,  // Pull distance exceeded threshold
    Refreshing,      // Currently refreshing
    Success,         // Show success animation
    Error,           // Show error state
}

impl<W: Widget> RefreshableList<W> {
    pub fn new(items: Vec<W>) -> Self {
        Self {
            items,
            state: RefreshState::Idle,
            pull_distance: 0.0,
            refresh_threshold: 80.0,
            on_refresh: None,
            refreshing_indicator: false,
        }
    }

    pub fn on_refresh<F>(mut self, callback: F) -> Self
    where
        F: Fn() -> Box<dyn Future<Output = ()>> + 'static
    {
        self.on_refresh = Some(Box::new(callback));
        self
    }

    pub fn handle_gesture(&mut self, gesture: &Gesture) -> bool {
        match gesture {
            Gesture::Pan { delta, current_position } => {
                // Only allow pull from top
                if current_position.y < 100.0 && self.state != RefreshState::Refreshing {
                    if self.state == RefreshState::Idle {
                        self.state = RefreshState::Pulling { distance: 0.0 };
                    }

                    if let RefreshState::Pulling { distance } = &mut self.state {
                        *distance += delta.y;
                        self.pull_distance = (*distance).min(200.0);  // Max pull 200px

                        // Check if ready to refresh
                        if self.pull_distance >= self.refresh_threshold {
                            self.state = RefreshState::ReadyToRefresh;
                        }

                        return true;
                    }
                }
            }

            Gesture::Swipe { direction, .. } if *direction == SwipeDirection::Down => {
                if matches!(
                    self.state,
                    RefreshState::Pulling { .. } | RefreshState::ReadyToRefresh
                ) {
                    if self.pull_distance >= self.refresh_threshold {
                        self.start_refresh();
                    } else {
                        self.reset();
                    }
                    return true;
                }
            }

            _ => {}
        }

        false
    }

    fn start_refresh(&mut self) {
        self.state = RefreshState::Refreshing;
        self.pull_distance = self.refresh_threshold;

        if let Some(on_refresh) = &self.on_refresh {
            let future = on_refresh();
            let sender = self.refresh_complete_sender.clone();

            // Execute refresh and send result when done
            async move {
                future.await;
                sender.send(()).ok();
            };
        }
    }

    fn handle_refresh_complete(&mut self) {
        self.state = RefreshState::Success;

        // Show success briefly then reset
        let timer = Timer::new(Duration::from_millis(1000));
        timer.on_complete(move || {
            self.reset();
        });
    }

    fn reset(&mut self) {
        self.animate_offset_to(0.0);
        self.state = RefreshState::Idle;
        self.pull_distance = 0.0;
    }

    fn animate_offset_to(&mut self, target: f64) {
        let animator = SpringAnimator::new(
            self.pull_distance,
            target,
            stiffness: 500.0,
            damping: 35.0,
        );

        animator.start(|offset| {
            self.pull_distance = offset;
        });
    }
}
```

### 4. SwipeableViewPager

Horizontal paging with swipe navigation.

```rust
pub struct SwipeableViewPager<W: Widget> {
    pages: Vec<W>,
    current_page: usize,
    page_offset: f64,
    state: PagerState,
    page_indicator: bool,
    on_page_changed: Option<Box<dyn Fn(usize)>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PagerState {
    Idle,
    Dragging { offset: f64 },
    Settling,  // Animating to page
    Looping,   // Wrap around (infinite scroll)
}

impl<W: Widget> SwipeableViewPager<W> {
    pub fn new(pages: Vec<W>) -> Self {
        Self {
            pages,
            current_page: 0,
            page_offset: 0.0,
            state: PagerState::Idle,
            page_indicator: true,
            on_page_changed: None,
        }
    }

    pub fn handle_gesture(&mut self, gesture: &Gesture) -> bool {
        match gesture {
            Gesture::Pan { delta, .. } => {
                if self.state == PagerState::Idle {
                    self.state = PagerState::Dragging { offset: 0.0 };
                }

                if let PagerState::Dragging { offset } = &mut self.state {
                    *offset += delta.x;
                    self.page_offset = *offset;
                    return true;
                }
            }

            Gesture::Swipe { direction, .. } => {
                match direction {
                    SwipeDirection::Left => {
                        // Next page
                        if self.current_page < self.pages.len() - 1 {
                            self.go_to_page(self.current_page + 1);
                        } else {
                            self.bounce_back();
                        }
                        return true;
                    }
                    SwipeDirection::Right => {
                        // Previous page
                        if self.current_page > 0 {
                            self.go_to_page(self.current_page - 1);
                        } else {
                            self.bounce_back();
                        }
                        return true;
                    }
                    _ => {}
                }
            }

            _ => {}
        }

        false
    }

    pub fn go_to_page(&mut self, page: usize) {
        if page >= self.pages.len() {
            return;
        }

        let target_offset = (page as f64 - self.current_page as f64) * self.page_width();

        self.state = PagerState::Settling;

        let animator = SpringAnimator::new(
            self.page_offset,
            target_offset,
            stiffness: 400.0,
            damping: 30.0,
        );

        let old_page = self.current_page;
        animator.start(move |offset| {
            self.page_offset = offset;

            // Update current page when settled
            if offset == target_offset {
                self.current_page = page;
                self.state = PagerState::Idle;

                if let Some(on_page_changed) = &self.on_page_changed {
                    on_page_changed(page);
                }
            }
        });
    }

    fn bounce_back(&mut self) {
        // Animate back to current page
        self.state = PagerState::Settling;

        let animator = SpringAnimator::new(
            self.page_offset,
            0.0,
            stiffness: 400.0,
            damping: 30.0,
        );

        animator.start(|offset| {
            self.page_offset = offset;
            if offset == 0.0 {
                self.state = PagerState::Idle;
            }
        });
    }

    fn page_width(&self) -> f64 {
        // Width of visible page area
        400.0  // Should use actual screen width
    }
}
```

### 5. Floating Action Button (FAB)

Circular button that floats above content, typically for primary actions.

```rust
pub struct FloatingActionButton {
    icon: Icon,
    size: FabSize,
    position: FabPosition,
    background: Color,
    on_click: Box<dyn Fn()>,
    extended: bool,
    label: Option<String>,
    state: FabState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FabSize {
    Mini,      // 40×40px
    Standard,  // 56×56px
    Extended,  // Varies by content
    Large,     // 64×64px
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FabPosition {
    End,       // Bottom-right (default)
    EndTop,    // Top-right
    Start,     // Bottom-left
    StartTop,  // Top-left
    Center,    // Center of screen
    CenterBottom,
    CenterTop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FabState {
    Resting,
    Pressed,
    Expanding,  // Transitioning to extended
    Collapsing, // Transitioning to standard
}

impl FloatingActionButton {
    pub fn new(icon: Icon) -> Self {
        Self {
            icon,
            size: FabSize::Standard,
            position: FabPosition::End,
            background: Color::from_hex("#6200EE"),  // Primary color
            on_click: Box::new(|| {}),
            extended: false,
            label: None,
            state: FabState::Resting,
        }
    }

    pub fn on_click<F>(mut self, callback: F) -> Self
    where
        F: Fn() + 'static
    {
        self.on_click = Box::new(callback);
        self
    }

    pub fn extend(mut self, label: String) -> Self {
        self.extended = true;
        self.label = Some(label);
        self.size = FabSize::Extended;
        self
    }

    pub fn handle_gesture(&mut self, gesture: &Gesture) -> bool {
        match gesture {
            Gesture::Tap { position, .. } => {
                if self.contains_point(*position) {
                    self.state = FabState::Pressed;

                    // Ripple effect
                    self.show_ripple(*position);

                    // Execute callback
                    (self.on_click)();

                    return true;
                }
            }

            Gesture::LongPress { position, .. } => {
                if self.contains_point(*position) {
                    // Show extended menu or options
                    return true;
                }
            }

            _ => {}
        }

        false
    }

    fn show_ripple(&self, position: Point<f64, Logical>) {
        // Create expanding circular ripple animation
        let ripple = RippleEffect {
            center: position,
            radius: 0.0,
            max_radius: self.size.diameter() / 2.0,
            opacity: 0.3,
        };

        ripple.animate(|radius, opacity| {
            // Update ripple visual
        });
    }

    fn contains_point(&self, point: Point<f64, Logical>) -> bool {
        let bounds = self.bounds();
        bounds.contains(point)
    }

    fn bounds(&self) -> Rect<f64, Logical> {
        let position = self.position.coordinates();
        let size = self.size.dimensions();

        Rect::new(position, Size::new(size.0, size.1))
    }
}

impl FabSize {
    fn dimensions(&self) -> (f64, f64) {
        match self {
            FabSize::Mini => (40.0, 40.0),
            FabSize::Standard => (56.0, 56.0),
            FabSize::Extended => {
                // Width varies by label, height is 48×2 = 96px
                (200.0, 48.0)
            }
            FabSize::Large => (64.0, 64.0),
        }
    }

    fn diameter(&self) -> f64 {
        self.dimensions().0
    }
}

impl FabPosition {
    fn coordinates(&self) -> Point<f64, Logical> {
        let screen_size = (1920.0, 1080.0);  // Should use actual screen
        let fab_size = 56.0;  // Standard

        // 16px padding from edge
        let padding = 16.0;

        match self {
            FabPosition::End => Point::new(
                screen_size.0 - fab_size - padding,
                screen_size.1 - fab_size - padding - 64.0,  // Above nav bar
            ),
            FabPosition::EndTop => Point::new(
                screen_size.0 - fab_size - padding,
                padding + 64.0,  // Below status bar
            ),
            FabPosition::Start => Point::new(
                padding,
                screen_size.1 - fab_size - padding - 64.0,
            ),
            FabPosition::StartTop => Point::new(
                padding,
                padding + 64.0,
            ),
            FabPosition::Center => Point::new(
                screen_size.0 / 2.0 - fab_size / 2.0,
                screen_size.1 / 2.0 - fab_size / 2.0,
            ),
            FabPosition::CenterBottom => Point::new(
                screen_size.0 / 2.0 - fab_size / 2.0,
                screen_size.1 - fab_size - padding - 64.0,
            ),
            FabPosition::CenterTop => Point::new(
                screen_size.0 / 2.0 - fab_size / 2.0,
                padding + 64.0,
            ),
        }
    }
}
```

### 6. Snackbar

Transient messages with optional actions.

```rust
pub struct Snackbar {
    message: String,
    action: Option<SnackbarAction>,
    duration: Duration,
    state: SnackbarState,
    position: SnackbarPosition,
}

#[derive(Debug, Clone)]
pub struct SnackbarAction {
    pub label: String,
    pub on_click: Box<dyn Fn()>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnackbarState {
    Hidden,
    Showing,
    Dismissing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnackbarPosition {
    Bottom,
    Top,
}

impl Snackbar {
    pub fn new(message: String) -> Self {
        Self {
            message,
            action: None,
            duration: Duration::from_millis(4000),
            state: SnackbarState::Hidden,
            position: SnackbarPosition::Bottom,
        }
    }

    pub fn with_action(mut self, label: String, on_click: Box<dyn Fn()>) -> Self {
        self.action = Some(SnackbarAction { label, on_click });
        self
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn show(&mut self) {
        self.state = SnackbarState::Showing;

        // Auto-dismiss after duration
        let timer = Timer::new(self.duration);
        timer.on_complete(move || {
            self.dismiss();
        });
    }

    pub fn dismiss(&mut self) {
        self.state = SnackbarState::Dismissing;

        // Animate out
        let animator = TweenAnimator::new(
            1.0,  // opacity
            0.0,
            duration: Duration::from_millis(300),
            easing: Easing::EaseInOut,
        );

        animator.start(|opacity| {
            // Update opacity
        });

        // Mark as hidden after animation
        let timer = Timer::new(Duration::from_millis(300));
        timer.on_complete(move || {
            self.state = SnackbarState::Hidden;
        });
    }
}

/// Snackbar manager for queueing multiple snackbars
pub struct SnackbarManager {
    queue: Vec<Snackbar>,
    current: Option<Snackbar>,
    max_in_queue: usize,
}

impl SnackbarManager {
    pub fn new() -> Self {
        Self {
            queue: Vec::new(),
            current: None,
            max_in_queue: 3,
        }
    }

    pub fn show(&mut self, snackbar: Snackbar) {
        if self.current.is_none() {
            self.show_now(snackbar);
        } else {
            // Queue for later
            if self.queue.len() < self.max_in_queue {
                self.queue.push(snackbar);
            }
        }
    }

    fn show_now(&mut self, mut snackbar: Snackbar) {
        snackbar.show();
        self.current = Some(snackbar);
    }

    pub fn on_current_dismissed(&mut self) {
        self.current = None;

        // Show next in queue
        if let Some(next) = self.queue.drain(0..1).next() {
            self.show_now(next);
        }
    }
}
```

## Responsive Layout System

```rust
pub trait ResponsiveLayout {
    /// Adjust layout based on screen size and orientation
    fn adapt(&mut self, screen_info: &ScreenInfo);

    /// Return preferred size for given constraints
    fn preferred_size(&self, constraints: SizeConstraints) -> Size;
}

pub struct ScreenInfo {
    pub width: f64,
    pub height: f64,
    pub density: f64,  // DPI
    pub orientation: Orientation,
    pub form_factor: FormFactor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Portrait,
    Landscape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormFactor {
    Phone,       // < 600dp width
    Tablet,      // 600-840dp width
    Desktop,     // > 840dp width
}

impl FormFactor {
    pub fn from_width(width_dp: f64) -> Self {
        if width_dp < 600.0 {
            FormFactor::Phone
        } else if width_dp < 840.0 {
            FormFactor::Tablet
        } else {
            FormFactor::Desktop
        }
    }
}

/// Adaptive container that changes layout based on form factor
pub struct AdaptiveContainer<W: Widget> {
    phone_layout: Box<dyn Fn() -> W>,
    tablet_layout: Box<dyn Fn() -> W>,
    desktop_layout: Box<dyn Fn() -> W>,
    current: Option<W>,
}

impl<W: Widget> AdaptiveContainer<W> {
    pub fn new() -> Self {
        Self {
            phone_layout: Box::new(|| unreachable!()),
            tablet_layout: Box::new(|| unreachable!()),
            desktop_layout: Box::new(|| unreachable!()),
            current: None,
        }
    }

    pub fn phone_layout<F>(mut self, layout: F) -> Self
    where
        F: Fn() -> W + 'static
    {
        self.phone_layout = Box::new(layout);
        self
    }

    pub fn tablet_layout<F>(mut self, layout: F) -> Self
    where
        F: Fn() -> W + 'static
    {
        self.tablet_layout = Box::new(layout);
        self
    }

    pub fn desktop_layout<F>(mut self, layout: F) -> Self
    where
        F: Fn() -> W + 'static
    {
        self.desktop_layout = Box::new(layout);
        self
    }

    pub fn adapt(&mut self, screen_info: &ScreenInfo) {
        let layout_fn = match screen_info.form_factor {
            FormFactor::Phone => &self.phone_layout,
            FormFactor::Tablet => &self.tablet_layout,
            FormFactor::Desktop => &self.desktop_layout,
        };

        self.current = Some(layout_fn());
    }
}
```

## Animation System

```rust
pub struct SpringAnimator {
    from: f64,
    to: f64,
    stiffness: f64,
    damping: f64,
    mass: f64,
    on_update: Box<dyn Fn(f64)>,
}

impl SpringAnimator {
    pub fn new(
        from: f64,
        to: f64,
        stiffness: f64,
        damping: f64,
    ) -> Self {
        Self {
            from,
            to,
            stiffness,
            damping,
            mass: 1.0,
            on_update: Box::new(|_| {}),
        }
    }

    pub fn on_update<F>(mut self, callback: F) -> Self
    where
        F: Fn(f64) + 'static
    {
        self.on_update = Box::new(callback);
        self
    }

    pub fn start(self) {
        // Simulate spring physics
        let mut position = self.from;
        let mut velocity = 0.0;
        let dt = 1.0 / 60.0;  // 60fps

        let animation = move || {
            let displacement = position - self.to;
            let spring_force = -self.stiffness * displacement;
            let damping_force = -self.damping * velocity;
            let acceleration = (spring_force + damping_force) / self.mass;

            velocity += acceleration * dt;
            position += velocity * dt;

            (self.on_update)(position);

            // Continue animation until settled
            if velocity.abs() > 0.1 || (position - self.to).abs() > 0.1 {
                request_animation_frame(animation);
            }
        };

        request_animation_frame(animation);
    }
}

pub struct TweenAnimator {
    from: f64,
    to: f64,
    duration: Duration,
    easing: Easing,
    on_update: Box<dyn Fn(f64)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    // ... more easing functions
}

impl TweenAnimator {
    pub fn new(
        from: f64,
        to: f64,
        duration: Duration,
        easing: Easing,
    ) -> Self {
        Self {
            from,
            to,
            duration,
            easing,
            on_update: Box::new(|_| {}),
        }
    }

    pub fn on_update<F>(mut self, callback: F) -> Self
    where
        F: Fn(f64) + 'static
    {
        self.on_update = Box::new(callback);
        self
    }

    pub fn start(self) {
        let start_time = Instant::now();

        let animation = move || {
            let elapsed = start_time.elapsed();
            let progress = (elapsed.as_secs_f64() / self.duration.as_secs_f64()).min(1.0);

            let eased_progress = self.easing.apply(progress);
            let value = self.from + (self.to - self.from) * eased_progress;

            (self.on_update)(value);

            if progress < 1.0 {
                request_animation_frame(animation);
            }
        };

        request_animation_frame(animation);
    }
}

impl Easing {
    fn apply(self, t: f64) -> f64 {
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - 2.0 * (1.0 - t) * (1.0 - t)
                }
            }
            Easing::EaseInQuad => t * t * t,
            Easing::EaseOutQuad => 1.0 - (1.0 - t).powi(3),
            Easing::EaseInOutQuad => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - 4.0 * (1.0 - t).powi(3)
                }
            }
        }
    }
}
```

## Configuration

```toml
# /etc/rustica/mobile-components.conf
[general]
# Default animation duration (ms)
default_animation_duration = 300

# Spring physics defaults
spring_stiffness = 400.0
spring_damping = 30.0

[bottom_sheet]
# Enable peek state
enable_peek = true

# Peek height (pixels)
peek_height = 80

# Enable swipe-to-dismiss
swipe_to_dismiss = true

[swipeable_list]
# Action width (pixels)
action_width = 80

# Swipe threshold to trigger action (fraction)
swipe_threshold = 0.7

[refreshable_list]
# Pull threshold to trigger refresh (pixels)
refresh_threshold = 80

# Maximum pull distance (pixels)
max_pull_distance = 200

[view_pager]
# Page snap sensitivity (0.0-1.0)
snap_sensitivity = 0.3

# Enable infinite looping
enable_loop = false

[fab]
# Default FAB size
default_size = "standard"

# Enable ripple effect
ripple_effect = true

[snackbar]
# Default duration (ms)
default_duration = 4000

# Maximum queued snackbars
max_queue = 3
```

## Accessibility

All components support:

1. **Minimum Touch Targets**: 44×44px minimum (WCAG AAA)
2. **Semantic Labels**: Proper AT-SPI role descriptions
3. **Focus Indicators**: Clear visual focus indication
4. **Keyboard Navigation**: Full keyboard support
5. **Screen Reader**: Announce state changes
6. **Reduced Motion**: Respect prefers-reduced-motion
7. **Color Contrast**: WCAG AAA compliant (7:1 ratio)

## Dependencies

```toml
[dependencies]
librustica = { path = "../librustica" }
rustica-gestures = { path = "../rustica-gestures" }
rustica-keyboard = { path = "../rustica-keyboard" }
```

## Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bottom_sheet_expand() {
        let content = Box::new(Container::new());
        let mut sheet = BottomSheet::new(content);
        assert!(matches!(sheet.state, SheetState::Hidden));

        sheet.expand();
        assert!(matches!(sheet.state, SheetState::Expanded));
    }

    #[test]
    fn test_fab_touch_target_size() {
        let fab = FloatingActionButton::new(Icon::Add);
        let size = fab.size.dimensions();

        // Minimum touch target is 44×44px
        assert!(size.0 >= 44.0);
        assert!(size.1 >= 44.0);
    }

    #[test]
    fn test_spring_animation() {
        let animator = SpringAnimator::new(0.0, 100.0, 400.0, 30.0);

        let mut final_value = None;
        animator.on_update(|v| final_value = Some(v));

        animator.start();

        // After settling, should reach target
        assert_eq!(final_value, Some(100.0));
    }

    #[test]
    fn test_responsive_layout() {
        let phone_info = ScreenInfo {
            width: 360.0,
            height: 640.0,
            density: 2.0,
            orientation: Orientation::Portrait,
            form_factor: FormFactor::Phone,
        };

        let mut container = AdaptiveContainer::new()
            .phone_layout(|| Container::new())
            .tablet_layout(|| Container::new())
            .desktop_layout(|| Container::new());

        container.adapt(&phone_info);
        assert!(container.current.is_some());
    }
}
```

## Future Enhancements

1. **More Components**: Carousel, Stepper, Timeline, ChipGroup
2. **Haptic Feedback**: Vibration on gesture completion
3. **Staggered Animations**: Animate multiple items sequentially
4. **Shared Element Transitions**: Animate elements between screens
5. **Gesture Customization**: Per-app gesture overrides
6. **Component Themes**: Material Design 3 component variants
