use nannou::prelude::*;
use once_cell::sync::Lazy;

/// Pure black
pub static BLACK_F32: Lazy<Rgb<f32>> = Lazy::new(|| rgb(0.0, 0.0, 0.0));

/// Pure white
pub static WHITE_F32: Lazy<Rgb<f32>> = Lazy::new(|| rgb(1.0, 1.0, 1.0));

/// Pure red
pub static RED_F32: Lazy<Rgb<f32>> = Lazy::new(|| rgb(1.0, 0.0, 0.0));

/// Pure green
pub static GREEN_F32: Lazy<Rgb<f32>> = Lazy::new(|| rgb(0.0, 1.0, 0.0));

/// Pure blue
pub static BLUE_F32: Lazy<Rgb<f32>> = Lazy::new(|| rgb(0.0, 0.0, 1.0));

/// Dark gray background
pub static DARK_GRAY_F32: Lazy<Rgb<f32>> = Lazy::new(|| rgb(0.1, 0.1, 0.1));

/// Muted blue-gray for buttons
pub static SLATE_F32: Lazy<Rgb<f32>> = Lazy::new(|| rgb(0.3, 0.3, 0.5));

/// Light blue border
pub static LIGHT_BLUE_F32: Lazy<Rgb<f32>> = Lazy::new(|| rgb(0.8, 0.8, 1.0));
