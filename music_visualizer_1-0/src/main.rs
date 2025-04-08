//! # Nannou + CPAL Audio Visualizer
//!
//! This module provides the main application structure for an audio visualizer
//! that combines Nannou for visualization and CPAL for audio playback.
//!
//! The application follows a Model-View-Controller (MVC) pattern with:
//! - Controller handling the business logic
//! - View managing the visual display
//! - Model (this file) maintaining application state

//Lesson summary - 3/27/25:
// - Focus on visuals keep playback controls simple
// - work on music library code later only load one song at a time work on song processing
// - rename song and edit song.rs to be stronger and a better model
// - use idvf file types to load .wav files

/// Module containing the controller logic for managing application state
mod controller;
/// Module containing the menu UI and interaction logic
mod menu;
/// Module handling audio playback and song management
mod song;
/// Module responsible for visual rendering
mod view;

mod ui;

use controller::Controller;

/// Main entry point for the application
///
/// Initializes and runs the Nannou application with:
/// - `model` for initialization
/// - `update` for the main loop
/// - `view` for rendering
/// - A simple window for display
fn main() {
    nannou::app(model).update(update).simple_window(view).run();
}

/// The main application state container
///
/// Holds the controller which manages all other components:
/// - Audio playback
/// - UI interactions
/// - Visual rendering
struct Model {
    /// The main controller coordinating all application components
    controller: Controller,
}

/// Initializes the application model
///
/// # Arguments
/// * `app` - Reference to the Nannou application
///
/// # Returns
/// Fully initialized `Model` containing all application state
fn model(app: &nannou::App) -> Model {
    let window = app.main_window();
    let win_rect = window.rect();

    Model {
        controller: Controller::new(win_rect),
    }
}

/// Main application update loop
///
/// Called once per frame to update application state.
///
/// # Arguments
/// * `app` - Reference to the Nannou application
/// * `model` - Mutable reference to the application model
/// * `_update` - Nannou's update metadata (unused in this implementation)
fn update(app: &nannou::App, model: &mut Model, _update: nannou::prelude::Update) {
    model.controller.update(app);
}

/// Main rendering function
///
/// Called once per frame to render the current application state.
///
/// # Arguments
/// * `app` - Reference to the Nannou application
/// * `model` - Reference to the application model
/// * `frame` - The target frame for rendering
fn view(app: &nannou::App, model: &Model, frame: nannou::Frame) {
    model.controller.view(app, frame);
}
