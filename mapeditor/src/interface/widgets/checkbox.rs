use graphics::*;
use cosmic_text::{Attrs, Metrics};

use crate::{
    collection::*,
    interface::label::*,
    DrawSetting,
};

pub struct Checkbox {
    pub window: Vec<Rect>,
    pub text: Text,
    is_hover: bool,
    pub is_select: bool,
}

impl Checkbox {
    pub fn new(systems: &mut DrawSetting, 
                pos: Vec2,
                msg: &str,
                checkbox_size: Vec2,
                z_pos: [f32; 3],
                default_value: bool,
    ) -> Self {
        let mut window = vec![
            Rect::new(&mut systems.renderer, 0),
            Rect::new(&mut systems.renderer, 0),
        ];
        window[0].set_size(checkbox_size)
                .set_position(Vec3::new(pos.x, pos.y, z_pos[0]))
                .set_color(Color::rgba(180, 180, 180, 0))
                .set_use_camera(true); // Button
        window[1].set_size(Vec2::new(16.0, 16.0))
                .set_position(Vec3::new(pos.x + 2.0, pos.y + ((checkbox_size.y * 0.5) - 8.0), z_pos[1]))
                .set_use_camera(true); // Checkbox
        if default_value {
            window[1].set_color(Color::rgba(200, 200, 200, 255))
                    .set_border_width(2.0)
                    .set_border_color(Color::rgba(100, 100, 100, 255));
        } else {
            window[1].set_color(Color::rgba(100, 100, 100, 255))
                    .set_border_width(0.0);
        }
        
        let mut text = create_label(systems,
                Vec3::new(pos.x + 24.0, pos.y, z_pos[2]),
                Vec2::new(checkbox_size.x - 24.0, checkbox_size.y),
                Bounds::new(pos.x + 24.0, pos.y, pos.x + checkbox_size.x + 24.0, pos.y + checkbox_size.y),
                Color::rgba(180, 180, 180, 255));
        text.set_text(&mut systems.renderer, msg, Attrs::new());
        
        Self {
            window,
            text,
            is_hover: false,
            is_select: default_value,
        }
    }
    
    pub fn set_hover(&mut self, is_hover: bool) {
        if self.is_hover == is_hover {
            return;
        }

        self.is_hover = is_hover;
        if self.is_hover {
            self.window[0].set_color(Color::rgba(180, 180, 180, 255));
            self.text.set_default_color(Color::rgba(40, 40, 40, 255));
        } else {
            self.window[0].set_color(Color::rgba(180, 180, 180, 0));
            self.text.set_default_color(Color::rgba(180, 180, 180, 255));
        }
    }

    pub fn set_select(&mut self, is_select: bool) {
        if self.is_select == is_select {
            return;
        }

        self.is_select = is_select;
        if self.is_select {
            self.window[1].set_color(Color::rgba(200, 200, 200, 255))
                        .set_border_width(2.0)
                        .set_border_color(Color::rgba(100, 100, 100, 255));
        } else {
            self.window[1].set_color(Color::rgba(100, 100, 100, 255))
                        .set_border_width(0.0);
        }
    }
}