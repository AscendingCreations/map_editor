use graphics::*;
use cosmic_text::{Attrs, Metrics};

use crate::{
    DrawSetting,
    interface::label::*,
    gfx_order::*,
};

pub struct TabText {
    pub text: Text,
    pub button: Image,
    pub is_selected: bool,
    pub is_hover: bool,
    pub is_visible: bool,
}

impl TabText {
    pub fn new(draw_setting: &mut DrawSetting, pos: Vec2) -> Self {
        let mut button = Image::new(Some(draw_setting.resource.tab_option.allocation), &mut draw_setting.renderer, 1);

        // Setup the interface position, height, width, color and texture coordinate
        button.pos = Vec3::new(pos.x, pos.y, ORDER_TAB_BUTTON);
        button.hw = Vec2::new(194.0, 20.0);
        button.uv = Vec4::new(0.0, 0.0, 194.0, 20.0);

        let text = create_basic_label(draw_setting,
            Vec3::new(pos.x + 24.0, pos.y - 1.0, ORDER_TAB_LABEL),
            Vec2::new(165.0, 20.0),
            Color::rgba(120, 120, 120, 255));

        Self {
            text,
            button,
            is_selected: false,
            is_hover: false,
            is_visible: false,
        }
    }

    pub fn init(&mut self, renderer: &mut GpuRenderer, msg: &str, width: f32) {
        self.text.set_text(renderer, msg, Attrs::new());
        self.text.changed = true;
        // Change width
        self.button.hw.x = width;
        self.button.uv.z = width;
        self.button.changed = true;
        self.is_visible = true;
    }

    pub fn update(&mut self, renderer: &mut GpuRenderer, msg: &str, is_select: bool) {
        if !self.is_visible {
            return;
        }
        
        self.text.set_text(renderer, msg, Attrs::new());
        self.text.changed = true;

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

    pub fn close(&mut self, renderer: &mut GpuRenderer) {
        if !self.is_visible {
            return;
        }
        self.text.set_text(renderer, "", Attrs::new());
        self.button.uv.y = 0.0;
        self.button.changed = true;
        self.is_hover = false;
        self.is_selected = false;
        self.is_visible = false;
    }

    pub fn set_select(&mut self, is_select: bool) {
        if !self.is_visible {
            return;
        }
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
        if !self.is_visible {
            return;
        }
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