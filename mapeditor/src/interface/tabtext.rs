use graphics::*;
use cosmic_text::{Attrs, Metrics};
use winit::dpi::PhysicalSize;
use crate::resource::*;
use crate::interface::create_label;

pub struct TabText {
    pub text: Text,
    pub button: Image,
    pub is_selected: bool,
    pub is_hover: bool,
}

impl TabText {
    pub fn new(resource: &TextureAllocation, renderer: &mut GpuRenderer, size: &PhysicalSize<f32>, scale: f64, msg: &str, pos: Vec2) -> Self {
        let mut button = Image::new(Some(resource.tab_option.allocation), renderer, 1);

        // Setup the interface position, height, width, color and texture coordinate
        button.pos = Vec3::new(pos.x, pos.y, 10.0);
        button.hw = Vec2::new(194.0, 20.0);
        button.uv = Vec4::new(0.0, 0.0, 194.0, 20.0);
        button.color = Color::rgba(255, 255, 255, 255);

        let mut text = create_label(renderer, size, scale,
            Vec3::new(pos.x + 24.0, pos.y - 1.0, 1.9),
            Vec2::new(165.0, 20.0),
            Color::rgba(120, 120, 120, 255));
        text.set_text(renderer, msg, Attrs::new());

        Self {
            text,
            button,
            is_selected: false,
            is_hover: false,
        }
    }

    pub fn set_select(&mut self, is_select: bool) {
        if self.is_selected != is_select {
            self.is_selected = is_select;

            if is_select {
                self.button.uv.y = 40.0;
            } else {
                self.button.uv.y = 0.0;
            }
            self.button.changed = true;
        }
    }

    pub fn set_hover(&mut self, is_hover: bool) {
        if self.is_hover != is_hover {
            self.is_hover = is_hover;

            if !self.is_selected {
                if is_hover {
                    self.button.uv.y = 20.0;
                } else {
                    self.button.uv.y = 0.0;
                }
                self.button.changed = true;
            }
        }
    }
}