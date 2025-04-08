//! The application controller module
//!
//! Manages the coordination between all application components:
//! - View (visual display)
//! - Menu (user interface)
//! - Song (audio playback)
//!
//! Handles layout, updates, and rendering of the complete application.

use crate::{menu::Menu, view::View};
use nannou::prelude::*;

/// Main application controller that orchestrates all components
///
/// Responsible for:
/// - Managing the layout of UI elements
/// - Coordinating updates between components
/// - Handling the overall rendering pipeline
pub struct Controller {
    /// Handles the main visualization display
    view: View,
    /// Manages the user interface and controls
    menu: Menu,
    /// Stores the main window dimensions
    window_rect: Rect,
}

impl Controller {
    /// Creates a new Controller with proper layout initialization
    ///
    /// # Arguments
    /// * `win_rect` - The dimensions of the application window
    ///
    /// # Layout
    /// - Menu takes up 200px on the right side
    /// - View occupies remaining space on the left
    /// - A divider line separates the two sections
    pub fn new(win_rect: Rect) -> Self {
        let menu_width = 200.0;
        let menu_rect = Rect::from_x_y_w_h(
            win_rect.right() - menu_width / 2.0,
            win_rect.y(),
            menu_width,
            win_rect.h(),
        );

        let view_rect = Rect::from_x_y_w_h(
            win_rect.left() + (win_rect.w() - menu_width) / 2.0,
            win_rect.y(),
            win_rect.w() - menu_width,
            win_rect.h(),
        );

        Controller {
            view: View::new(view_rect),
            menu: Menu::new(menu_rect),
            window_rect: win_rect,
        }
    }

    /// Updates all application components
    ///
    /// Called once per frame to:
    /// 1. Update menu state based on user input
    /// 2. Update song playback based on menu state
    /// 3. Update view based on playback state
    ///
    /// # Arguments
    /// * `app` - Reference to the Nannou application for input handling
    pub fn update(&mut self, app: &App) {
        self.menu.update(app);
        self.menu.song.update(self.menu.is_playing());
        self.view.update(self.menu.song.is_playing());
    }

    /// Renders all application components
    ///
    /// Draws the complete application interface including:
    /// - Background
    /// - View visualization
    /// - Menu controls
    /// - Divider line
    ///
    /// # Arguments
    /// * `app` - Reference to the Nannou application
    /// * `frame` - The target frame for rendering
    pub fn view(&self, app: &App, frame: Frame) {
        let draw = app.draw();

        // Clear the background
        draw.background().color(BLACK);

        // Draw view and menu
        self.view.draw(&draw); //TODO: Rename (maybe canvas)
        self.menu.draw(&draw);

        // Draw divider line between view and menu
        draw.line()
            .start(pt2(
                self.window_rect.right() - 200.0,
                self.window_rect.top(),
            ))
            .end(pt2(
                self.window_rect.right() - 200.0,
                self.window_rect.bottom(),
            ))
            .color(BLACK)
            .weight(1.0);

        draw.to_frame(app, &frame).unwrap();
    }
}
