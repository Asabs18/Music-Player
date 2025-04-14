//! User interface menu module
//!
//! Handles the interactive control panel for the application, including:
//! - Play/pause button
//! - Menu layout and rendering
//! - Mouse interaction handling
//!
//! The menu provides visual feedback and translates user input into playback commands.

use crate::song::Song;
use crate::ui::button::Button;
use crate::ui::color::*;
use nannou::prelude::*;
use std::fs;
use std::io;
use std::path::Path;

/// Represents the interactive control menu.
///
/// This struct handles the UI for interacting with the music visualizer,
/// allowing the user to select songs, control playback, and navigate back
/// from the playback screen.
pub struct Menu {
    is_playing: bool,
    menu_rect: Rect,
    buttons: Vec<Button>,
    was_mouse_pressed: bool,
    /// The currently selected song, if any.
    pub song: Song,
    song_buttons_created: bool,
}

impl Menu {
    // ============================================================================
    // Constructors and Public Methods
    // ============================================================================

    /// Creates a new `Menu` instance.
    ///
    /// The menu is initialized with default playback buttons (play and back) created via
    /// [`default_buttons()`] and no song selected.
    ///
    /// # Arguments
    ///
    /// * `menu_rect` - A [`Rect`] that defines the area where the menu will be rendered.
    ///
    /// # Returns
    ///
    /// A new `Menu` instance.
    pub fn new(menu_rect: Rect) -> Self {
        Self {
            is_playing: false,
            song: Song::empty(),
            menu_rect,
            buttons: Self::default_buttons(menu_rect),
            was_mouse_pressed: false,
            song_buttons_created: false,
        }
    }

    /// Updates the menu state based on user interaction.
    ///
    /// This method processes input from the application to update button visibility,
    /// create song selection buttons if needed, update the play button label, and process mouse
    /// click events on visible buttons.
    ///
    /// # Arguments
    ///
    /// * `app` - A reference to the nannou [`App`] which provides access to input states.
    pub fn update(&mut self, app: &App) {
        let mouse = app.mouse.position();
        let is_mouse_pressed = app.mouse.buttons.pressed().next().is_some();

        // 🔍 Press 'D' to print debug information about the song's audio configuration.
        if app.keys.down.contains(&Key::D) {
            println!("\n🧪 [DEBUG] Dumping supported audio configs...\n");
            self.song.debug_info();
        }

        // Update button visibility based on the current screen.
        self.update_button_visibility();

        // Create song selection buttons if needed.
        if self.song.is_empty() && !self.song_buttons_created {
            self.create_song_buttons();
            self.song_buttons_created = true;
        }

        // Update the label for the play button.
        self.update_play_button_label();

        // Process mouse click events for visible buttons.
        self.process_mouse_click_events(mouse, is_mouse_pressed);

        self.was_mouse_pressed = is_mouse_pressed;
    }

    /// Draws the menu interface.
    ///
    /// This method clears the background with a dark gray color and renders either the
    /// song selection controls or the playback controls depending on whether a song is loaded.
    ///
    /// # Arguments
    ///
    /// * `draw` - A reference to the nannou [`Draw`] context for rendering the menu.
    pub fn draw(&self, draw: &Draw) {
        draw.rect()
            .xy(self.menu_rect.xy())
            .wh(self.menu_rect.wh())
            .color(*DARK_GRAY_F32);

        if self.song.is_empty() {
            self.draw_song_select_controls(draw);
        } else {
            self.draw_playback_controls(draw);
        }
    }

    /// Returns whether a song is currently playing.
    ///
    /// # Returns
    ///
    /// `true` if a song is playing, and `false` otherwise.
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    // ============================================================================
    // Private Helper Methods
    // ============================================================================

    /// Creates the default playback buttons (play and back).
    ///
    /// This helper function builds and returns a vector containing the play and back buttons.
    ///
    /// # Arguments
    ///
    /// * `menu_rect` - The rectangle in which to position the buttons.
    ///
    /// # Returns
    ///
    /// A `Vec<Button>` containing the play button and the back button.
    fn default_buttons(menu_rect: Rect) -> Vec<Button> {
        vec![
            Button::new(
                "PLAY",
                "play_button",
                Rect::from_x_y_w_h(
                    menu_rect.x(),
                    menu_rect.y() + menu_rect.h() * 0.3,
                    menu_rect.w() * 0.8,
                    50.0,
                ),
            ),
            Button::new(
                "BACK",
                "back_button",
                Rect::from_x_y_w_h(
                    menu_rect.x(),
                    menu_rect.y() - menu_rect.h() * 0.3,
                    menu_rect.w() * 0.8,
                    50.0,
                ),
            ),
        ]
    }

    /// Updates the visibility of buttons based on the current screen mode.
    ///
    /// If no song is selected (song selection screen), only song buttons (tags starting with `"song_"`)
    /// are made visible, and playback buttons (`"play_button"` and `"back_button"`) are hidden.
    /// When a song is loaded (playback screen), the reverse occurs.
    fn update_button_visibility(&mut self) {
        if self.song.is_empty() {
            // Song selection screen: hide playback buttons, show song selection buttons.
            for button in &mut self.buttons {
                if button.tag == "play_button" || button.tag == "back_button" {
                    button.is_visible = false;
                } else if button.tag.starts_with("song_") {
                    button.is_visible = true;
                }
            }
        } else {
            // Playback screen: show playback buttons, hide song selection buttons.
            for button in &mut self.buttons {
                if button.tag == "play_button" || button.tag == "back_button" {
                    button.is_visible = true;
                } else if button.tag.starts_with("song_") {
                    button.is_visible = false;
                }
            }
        }
    }

    /// Updates the play button's label based on the current playback state.
    ///
    /// If a song is playing, the button label is set to `"PAUSE"`. Otherwise, it is set to `"PLAY"`.
    fn update_play_button_label(&mut self) {
        let playing = self.is_playing;
        if let Some(play_button) = self.get_button_mut("play_button") {
            play_button.set_label(if playing { "PAUSE" } else { "PLAY" });
        }
    }

    /// Processes mouse click events for visible buttons.
    ///
    /// This method checks for a new mouse press and, if a button is pressed, handles it by:
    /// - Toggling the play state when the play button is pressed.
    /// - Resetting the song and playback state for the back button.
    /// - Loading a new song when a song selection button is pressed.
    ///
    /// # Arguments
    ///
    /// * `mouse` - The current mouse position.
    /// * `is_mouse_pressed` - A boolean indicating whether a mouse button is pressed.
    fn process_mouse_click_events(&mut self, mouse: Vec2, is_mouse_pressed: bool) {
        if is_mouse_pressed && !self.was_mouse_pressed {
            for button in &self.buttons {
                if button.is_visible && button.contains(mouse) {
                    match button.tag.as_str() {
                        "play_button" => {
                            self.is_playing = !self.is_playing;
                        }
                        "back_button" => {
                            self.song = Song::empty();
                            self.is_playing = false;
                            self.song_buttons_created = false;
                        }
                        _ if button.tag.starts_with("song_") => {
                            self.song =
                                Song::from_file(Song::get_file_from_title(&button.label).as_str());
                            // Remove song selection buttons once a song is chosen.
                            self.buttons
                                .retain(|b| b.tag == "play_button" || b.tag == "back_button");
                            self.song_buttons_created = false;
                        }
                        _ => {}
                    }
                    break;
                }
            }
        }
    }

    /// Draws the playback controls, including the play/pause and back buttons, as well as the
    /// currently playing song's title.
    ///
    /// # Arguments
    ///
    /// * `draw` - A reference to the nannou [`Draw`] context used for rendering.
    fn draw_playback_controls(&self, draw: &Draw) {
        if let Some(play_button) = self.get_button("play_button") {
            let button_color = if self.is_playing {
                *GREEN_F32
            } else {
                *RED_F32
            };

            play_button.draw(draw, button_color, *BLACK_F32, None);
        }

        if let Some(back_button) = self.get_button("back_button") {
            back_button.draw(draw, *BLUE_F32, *BLACK_F32, None);
        }

        if !self.song.is_empty() {
            draw.text(&format!("Now Playing: {}", self.song.title))
                .xy(pt2(self.menu_rect.x(), self.menu_rect.top() - 60.0))
                .color(*WHITE_F32)
                .font_size(20);
        }
    }

    /// Draws the song selection controls.
    ///
    /// This function renders a title ("SELECT A SONG") and all visible song selection buttons.
    ///
    /// # Arguments
    ///
    /// * `draw` - A reference to the nannou [`Draw`] context for rendering.
    fn draw_song_select_controls(&self, draw: &Draw) {
        draw.text("SELECT A SONG")
            .xy(pt2(self.menu_rect.x(), self.menu_rect.top() - 30.0))
            .color(*WHITE_F32)
            .font_size(24);

        // Render only the song selection buttons that are marked visible.
        for button in &self.buttons {
            if button.tag.starts_with("song_") && button.is_visible {
                button.draw(draw, *SLATE_F32, *WHITE_F32, Some(*LIGHT_BLUE_F32));
            }
        }
    }

    /// Retrieves a mutable reference to a button by its tag.
    ///
    /// Only buttons marked as visible are considered.
    ///
    /// # Arguments
    ///
    /// * `tag` - The identifier tag for the button.
    ///
    /// # Returns
    ///
    /// An `Option` containing a mutable reference to the matching button if found.
    fn get_button_mut(&mut self, tag: &str) -> Option<&mut Button> {
        self.buttons
            .iter_mut()
            .find(|b| b.tag == tag && b.is_visible)
    }

    /// Retrieves an immutable reference to a button by its tag.
    ///
    /// Only buttons marked as visible are considered.
    ///
    /// # Arguments
    ///
    /// * `tag` - The identifier tag for the button.
    ///
    /// # Returns
    ///
    /// An `Option` containing an immutable reference to the matching button if found.
    fn get_button(&self, tag: &str) -> Option<&Button> {
        self.buttons.iter().find(|b| b.tag == tag && b.is_visible)
    }

    /// Retrieves a list of song names from the specified directory.
    ///
    /// The file names are converted to song titles using [`Song::get_title_from_file`].
    ///
    /// # Arguments
    ///
    /// * `dir_path` - The path to the directory containing the song files.
    ///
    /// # Returns
    ///
    /// An [`io::Result`] containing a vector of song names, or an error if the directory cannot be read.
    fn get_song_names(&self, dir_path: &str) -> io::Result<Vec<String>> {
        let path = Path::new(dir_path);
        let mut file_names = Vec::new();

        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let file_name = entry.file_name();
                if let Some(name) = file_name.to_str() {
                    file_names.push(name.to_owned());
                }
            }
        }

        for name in &mut file_names {
            *name = Song::get_title_from_file(name);
        }

        Ok(file_names)
    }

    /// Creates song selection buttons dynamically by scanning the music library.
    ///
    /// Buttons are created for each song file found in the directory and are appended to the menu's
    /// button list, replacing any previously created song selection buttons.
    fn create_song_buttons(&mut self) {
        match self.get_song_names("music_library") {
            Ok(song_names) => {
                // Retain only the playback buttons; song selection buttons will be recreated.
                self.buttons
                    .retain(|b| b.tag == "play_button" || b.tag == "back_button");

                let button_width = self.menu_rect.w() * 0.7;
                let button_height = 50.0;
                let vertical_spacing = 60.0;
                let start_y = self.menu_rect.top() - 80.0;

                for (index, name) in song_names.iter().enumerate() {
                    let tag = format!("song_{}", index);
                    let button_rect = Rect::from_x_y_w_h(
                        self.menu_rect.x(),
                        start_y - (vertical_spacing * index as f32),
                        button_width,
                        button_height,
                    );
                    self.buttons.push(Button::new(name, &tag, button_rect));
                }
            }
            Err(e) => {
                eprintln!("Failed to retrieve song names: {}", e);
            }
        }
    }
}
