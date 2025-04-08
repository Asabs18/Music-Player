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

/// Represents the interactive control menu
pub struct Menu {
    is_playing: bool,
    menu_rect: Rect,
    buttons: Vec<Button>,
    was_mouse_pressed: bool,
    pub song: Song,
    song_buttons_created: bool,
}

impl Menu {
    pub fn new(menu_rect: Rect) -> Self {
        Self {
            is_playing: false,
            song: Song::empty(),
            menu_rect,
            buttons: vec![
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
            ],
            was_mouse_pressed: false,
            song_buttons_created: false,
        }
    }

    pub fn update(&mut self, app: &App) {
        let mouse = app.mouse.position();
        let is_mouse_pressed = app.mouse.buttons.pressed().next().is_some();

        // 🔍 Press 'D' to print debug info
        if app.keys.down.contains(&Key::D) {
            println!("\n🧪 [DEBUG] Dumping supported audio configs...\n");
            self.song.debug_info();
        }

        if self.song.is_empty() && !self.song_buttons_created {
            self.create_song_buttons();
            self.song_buttons_created = true;
        }

        let is_playing = self.is_playing;

        if let Some(play_button) = self.get_button_mut("play_button") {
            play_button.set_label(if is_playing { "PAUSE" } else { "PLAY" });
        }

        if is_mouse_pressed && !self.was_mouse_pressed {
            for button in &self.buttons {
                if button.contains(mouse) {
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

        self.was_mouse_pressed = is_mouse_pressed;
    }

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

    fn draw_song_select_controls(&self, draw: &Draw) {
        draw.text("SELECT A SONG")
            .xy(pt2(self.menu_rect.x(), self.menu_rect.top() - 30.0))
            .color(*WHITE_F32)
            .font_size(24);

        for button in &self.buttons {
            if button.tag.starts_with("song_") {
                button.draw(draw, *SLATE_F32, *WHITE_F32, Some(*LIGHT_BLUE_F32));
            }
        }
    }

    fn get_button_mut(&mut self, tag: &str) -> Option<&mut Button> {
        self.buttons
            .iter_mut()
            .find(|b| b.tag == tag && b.is_visible)
    }

    fn get_button(&self, tag: &str) -> Option<&Button> {
        self.buttons.iter().find(|b| b.tag == tag && b.is_visible)
    }

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

    fn create_song_buttons(&mut self) {
        match self.get_song_names("music_library") {
            Ok(song_names) => {
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

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }
}
