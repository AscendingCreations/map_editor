use graphics::*;
use cosmic_text::{Attrs, Metrics};

use crate::{
    gfx_order::*,
    DrawSetting,
};

pub struct Checkbox {
    pub window: Vec<Rect>,
    pub text: Text,
    is_hover: bool,
}

impl Checkbox {
    pub fn new(draw_setting: &mut DrawSetting, 
                pos: Vec2,
                msg: &str,
                checkbox_size: Vec2,
                z_pos: [f32; 3]
    ) -> Self {
        let mut window = vec![
            Rect::new(&mut draw_setting.renderer, 0),
            Rect::new(&mut draw_setting.renderer, 0),
        ];
        window[0].set_size(checkbox_size)
                .set_position(Vec3::new(pos.x, pos.y, z_pos[0]))
                .set_color(Color::rgba(180, 180, 180, 0))
                .set_use_camera(true); // Button
        window[1].set_size(Vec2::new(16.0, 16.0))
                .set_position(Vec3::new(pos.x + 2.0, pos.y + ((checkbox_size.y * 0.5) - 8.0), z_pos[1]))
                .set_color(Color::rgba(100, 100, 100, 255))
                .set_use_camera(true); // Checkbox
        
        let mut text = create_label(draw_setting,
                Vec3::new(pos.x + 24.0, pos.y, z_pos[2]),
                Vec2::new(checkbox_size.x - 24.0, checkbox_size.y),
                Bounds::new(pos.x + 24.0, pos.y, pos.x + checkbox_size.x + 24.0, pos.y + checkbox_size.y),
                Color::rgba(180, 180, 180, 255));
        text.set_text(&mut draw_setting.renderer, msg, Attrs::new());
        
        Self {
            window,
            text,
            is_hover: false,
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
}

fn create_label(draw_setting: &mut DrawSetting,
    pos: Vec3,
    label_size: Vec2,
    bounds: Bounds,
    color: Color,
) -> Text {
    let mut text = Text::new(
        &mut draw_setting.renderer,
        Some(Metrics::new(16.0, 16.0).scale(draw_setting.scale as f32)),
        Vec3::new(pos.x, pos.y, pos.z), label_size, 1.0
    );
    text.set_buffer_size(&mut draw_setting.renderer, draw_setting.size.width as i32, draw_setting.size.height as i32)
            .set_bounds(Some(bounds))
            .set_default_color(color);
    text.use_camera = true;
    text.changed = true;
    text
}