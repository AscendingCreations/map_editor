use graphics::*;
use cosmic_text::{Attrs, Metrics};

use crate::{
    gfx_order::*,
    interface::*,
    DrawSetting,
};

pub struct Button {
    pub image: Image,
    pub text: Text,
    in_hover: bool,
    in_click: bool,
    button_size: Vec2,
    adjust_text_y: f32,
}

impl Button {
    pub fn new(draw_setting: &mut DrawSetting,
                texture: usize,
                message: &str,
                pos: Vec2,
                button_size: Vec2,
                z_order: [f32; 2],
                adjust_text_y: f32,
    ) -> Self {
        let mut image = Image::new(Some(texture), &mut draw_setting.renderer, 1);
        image.pos = Vec3::new(pos.x, pos.y, z_order[0]);
        image.hw = button_size;
        image.uv = Vec4::new(0.0, 0.0, button_size.x, button_size.y);

        let adjust_x = (button_size.x * 0.5).floor() - (button_size.x * 0.5).floor();
        let mut text = create_label(draw_setting,
            Vec3::new(pos.x + adjust_x, pos.y + adjust_text_y, z_order[1]), 
            Vec2::new(button_size.x, 20.0),
            Bounds::new(pos.x, pos.y + adjust_text_y, pos.x + button_size.x, pos.y + button_size.y),
            Color::rgba(120, 120, 120, 255));
        text.set_text(&mut draw_setting.renderer, message, Attrs::new());
        // Adjust text x position
        let message_size = text.measure();
        text.pos.x =  pos.x + ((button_size.x * 0.5).floor() - (message_size.x * 0.5)).floor();
        text.changed = true;

        Self {
            image,
            text,
            in_hover: false,
            in_click: false,
            button_size,
            adjust_text_y,
        }
    }

    pub fn set_hover(&mut self, in_hover: bool) {
        if self.in_hover == in_hover {
            return;
        }
        self.in_hover = in_hover;
        if !self.in_click {
            if self.in_hover {
                self.image.uv.y = self.button_size.y;
            } else {
                self.image.uv.y = 0.0;
            }
            self.image.changed = true;
        }
    }

    pub fn set_click(&mut self, in_click: bool) {
        if self.in_click == in_click {
            return;
        }
        self.in_click = in_click;
        if self.in_click {
            self.image.uv.y = self.button_size.y * 2.0;
            self.text.pos.y = self.image.pos.y + (self.adjust_text_y - 2.0);
        } else {
            if !self.in_hover {
                self.image.uv.y = 0.0;
            } else {
                self.image.uv.y = self.button_size.y;
            }
            self.text.pos.y = self.image.pos.y + self.adjust_text_y;
        }
        self.image.changed = true;
        self.text.changed = true;
    }
}