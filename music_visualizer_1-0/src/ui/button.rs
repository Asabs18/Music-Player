use nannou::prelude::*;

/// A reusable UI button
pub struct Button {
    pub label: String,
    pub tag: String,
    pub rect: Rect,
    pub is_visible: bool,
}

impl Button {
    /// Creates a new Button
    pub fn new(label: &str, tag: &str, rect: Rect) -> Self {
        Self {
            label: label.to_string(),
            tag: tag.to_string(),
            rect,
            is_visible: true,
        }
    }

    /// Returns true if the mouse is inside the button and the button is visible
    pub fn contains(&self, point: Point2) -> bool {
        self.is_visible && self.rect.contains(point)
    }

    /// Draws the button if visible
    pub fn draw(
        &self,
        draw: &Draw,
        background: Rgb<f32>,
        text_color: Rgb<f32>,
        border: Option<Rgb<f32>>,
    ) {
        if !self.is_visible {
            return;
        }

        draw.rect()
            .xy(self.rect.xy())
            .wh(self.rect.wh())
            .color(background);

        if let Some(border_color) = border {
            draw.rect()
                .xy(self.rect.xy())
                .wh(self.rect.wh())
                .no_fill()
                .stroke(border_color)
                .stroke_weight(2.0);
        }

        draw.text(&self.label)
            .xy(self.rect.xy())
            .color(text_color)
            .font_size(20)
            .align_text_middle_y()
            .center_justify()
            .width(self.rect.w() - 20.0);
    }

    pub fn set_label(&mut self, label: &str) {
        self.label = label.to_string();
    }
}
