use graphics::*;
use cosmic_text::{Attrs, Metrics};

use crate::DrawSetting;

pub fn center_text(text: &mut Text) {
    let size = text.measure();
    let bound = text.bounds.unwrap_or_default();
    let textbox_size = bound.right - bound.left;
    text.pos.x = bound.left + ((textbox_size * 0.5) - (size.x * 0.5));
    text.changed = true;
}

pub fn create_label(draw_setting: &mut DrawSetting,
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

pub fn create_basic_label(draw_setting: &mut DrawSetting,
                pos: Vec3,
                label_size: Vec2,
                color: Color,
) -> Text {
    let mut text = Text::new(
        &mut draw_setting.renderer,
        Some(Metrics::new(16.0, 16.0).scale(draw_setting.scale as f32)),
        Vec3::new(pos.x, pos.y, pos.z), label_size, 1.0
    );
    text.set_buffer_size(&mut draw_setting.renderer, draw_setting.size.width as i32, draw_setting.size.height as i32)
        .set_bounds(Some(Bounds::new(pos.x, pos.y, pos.x + label_size.x, pos.y + label_size.y)))
        .set_default_color(color);
    text.use_camera = true;
    text.changed = true;
    text
}