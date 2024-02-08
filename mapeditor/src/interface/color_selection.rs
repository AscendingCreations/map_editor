use graphics::*;
use cosmic_text::{Attrs, Metrics};

use crate::{
    gfx_order::*, 
    interface::{
        label::*,
        textbox::*,
        button::*,
    }, 
    textbox, 
    DrawSetting
};

pub struct ColorEditor {
    pub is_open: bool,
    pub window: Rect,
    pub label: Vec<Text>,
    pub textbox: Vec<Textbox>,
    pub button: Button,
    pub data: [u8; 4]
}

impl ColorEditor {
    pub fn new(draw_setting: &mut DrawSetting, pos: Vec2, z_order: [f32; 3], data: [u8; 4], can_edit_alpha: bool) -> Self {
        let window_size = if can_edit_alpha { Vec2::new(100.0, 153.0) } else { Vec2::new(100.0, 128.0) };
        let window_pos = Vec3::new(pos.x, pos.y - window_size.y, z_order[0]);
        let mut window = Rect::new(&mut draw_setting.renderer, 0);
        window.set_size(window_size)
                .set_position(window_pos)
                .set_color(Color::rgba(70, 70, 70, 255))
                .set_radius(3.0)
                .set_border_width(2.0)
                .set_border_color(Color::rgba(20, 20, 20, 255))
                .set_use_camera(true);
        
        let content_pos = Vec2::new(window_pos.x + 10.0, (window_pos.y + window_size.y) - 9.0);
        let mut label = vec![];
        let mut textbox = vec![];

        label.push(create_basic_label(draw_setting, 
                Vec3::new(content_pos.x, content_pos.y - 25.0, ORDER_COLOREDIT_TEXTBOX_TEXT), 
                Vec2::new(20.0, 20.0), Color::rgba(180, 180, 180, 255)));
        label[0].set_text(&mut draw_setting.renderer, "R", Attrs::new());
        label.push(create_basic_label(draw_setting, 
                Vec3::new(content_pos.x, content_pos.y - 50.0, ORDER_COLOREDIT_TEXTBOX_TEXT), 
                Vec2::new(20.0, 20.0), Color::rgba(180, 180, 180, 255)));
        label[1].set_text(&mut draw_setting.renderer, "G", Attrs::new());
        label.push(create_basic_label(draw_setting, 
                Vec3::new(content_pos.x, content_pos.y - 75.0, ORDER_COLOREDIT_TEXTBOX_TEXT), 
                Vec2::new(20.0, 20.0), Color::rgba(180, 180, 180, 255)));
        label[2].set_text(&mut draw_setting.renderer, "B", Attrs::new());

        textbox.push(Textbox::new(draw_setting, Vec3::new(content_pos.x + 20.0, content_pos.y - 25.0, ORDER_COLOREDIT_TEXTBOX),
                    Vec2::new(60.0, 24.0), false));
        textbox[0].input_text(&mut draw_setting.renderer, data[0].to_string());
        textbox.push(Textbox::new(draw_setting, Vec3::new(content_pos.x + 20.0, content_pos.y - 50.0, ORDER_COLOREDIT_TEXTBOX),
                    Vec2::new(60.0, 24.0), false));
        textbox[1].input_text(&mut draw_setting.renderer, data[1].to_string());
        textbox.push(Textbox::new(draw_setting, Vec3::new(content_pos.x + 20.0, content_pos.y - 75.0, ORDER_COLOREDIT_TEXTBOX),
                    Vec2::new(60.0, 24.0), false));
        textbox[2].input_text(&mut draw_setting.renderer, data[2].to_string());

        if can_edit_alpha {
            label.push(create_basic_label(draw_setting, 
                    Vec3::new(content_pos.x, content_pos.y - 100.0, ORDER_COLOREDIT_TEXTBOX_TEXT), 
                    Vec2::new(20.0, 20.0), Color::rgba(180, 180, 180, 255)));
            label[3].set_text(&mut draw_setting.renderer, "A", Attrs::new());

            textbox.push(Textbox::new(draw_setting, Vec3::new(content_pos.x + 20.0, content_pos.y - 100.0, ORDER_COLOREDIT_TEXTBOX),
                    Vec2::new(60.0, 24.0), false));
            textbox[3].input_text(&mut draw_setting.renderer, data[3].to_string());
        }

        let button = Button::new(draw_setting, draw_setting.resource.preference_button.allocation, "Apply",
                Vec2::new(window_pos.x + 10.0, window_pos.y + 10.0), Vec2::new(80.0, 22.0), 
                [ORDER_COLOREDIT_BUTTON, ORDER_COLOREDIT_BUTTON_LABEL], 2.0);

        Self {
            is_open: false,
            window,
            data,
            label,
            textbox,
            button,
        }
    }

    pub fn open(&mut self) {
        if self.is_open {
            return;
        }
        self.is_open = true;
        self.window.changed = true;
        self.label.iter_mut().for_each(|label| {
            label.changed = true;
        });
        self.button.image.changed = true;
        self.button.text.changed = true;
        self.textbox.iter_mut().for_each(|textbox| {
            textbox.image.changed = true;
            textbox.text.changed = true;
            textbox.set_select(false);
        });
    }

    pub fn close(&mut self) {
        if !self.is_open {
            return;
        }
        self.is_open = false;
    }
}

pub struct ColorSelection {
    pub image: Rect,
    pub text: Text,
    is_hover: bool,

    pub color_editor: ColorEditor,
}

impl ColorSelection {
    pub fn new(draw_setting: &mut DrawSetting, pos: Vec3, size: Vec2, color: [u8; 4], msg: Option<&str>, can_edit_alpha: bool) -> Self {
        let mut image = Rect::new(&mut draw_setting.renderer, 0);
        image.set_size(size).set_position(Vec3::new(pos.x, pos.y, pos.z))
                .set_color(Color::rgba(color[0], color[1], color[2], color[3]))
                .set_radius(3.0)
                .set_border_width(2.0)
                .set_border_color(Color::rgba(20, 20, 20, 255))
                .set_use_camera(true);
        
        let text_pos = Vec2::new(pos.x + size.x + 10.0, pos.y);
        let mut text = create_basic_label(draw_setting, 
                Vec3::new(text_pos.x, text_pos.y, pos.z), 
                Vec2::new(100.0, 20.0),
                Color::rgba(180, 180, 180, 255));
        
        if msg.is_some() {
            text.set_text(&mut draw_setting.renderer, msg.unwrap(), Attrs::new());
            text.set_bounds(Some(Bounds::new(text_pos.x, text_pos.y, text_pos.x + text.measure().x, text_pos.y + 20.0)));
        };
        
        let color_editor = ColorEditor::new(draw_setting,
                            Vec2::new(pos.x, pos.y),
                            [ORDER_COLOREDIT_WINDOW, ORDER_COLOREDIT_TEXTBOX, ORDER_COLOREDIT_TEXTBOX_TEXT], 
                            color.clone(), can_edit_alpha);

        Self {
            image,
            text,
            is_hover: false,
            color_editor,
        }
    }

    pub fn set_hover(&mut self, is_hover: bool) {
        if self.is_hover == is_hover {
            return;
        }

        self.is_hover = is_hover;
        if self.is_hover {
            self.image.set_border_color(Color::rgba(200, 200, 200, 255));
        } else {
            self.image.set_border_color(Color::rgba(20, 20, 20, 255));
        }
    }

    pub fn open_color_editor(&mut self) {
        self.color_editor.open();
        self.image.changed = true;
        self.text.changed = true;
    }

    pub fn close_color_editor(&mut self) {
        self.color_editor.close();
    }
}