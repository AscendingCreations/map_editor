use graphics::*;
use cosmic_text::{Attrs, Metrics};

const MAX_VISIBLE_LIST: usize = 5;

use crate::{
    interface::{
        label::*,
        scrollbar::*,
    },
    DrawSetting
};

pub struct ListText {
    pub rect: Rect,
    pub text: Text,
    is_hover: bool,
    is_select: bool,
}

impl ListText {
    pub fn set_hover(&mut self, is_hover: bool) {
        if self.is_hover == is_hover || self.is_select {
            return;
        }
        self.is_hover = is_hover;
        if self.is_hover {
            self.rect.set_color(Color::rgba(55, 55, 55, 255));
        } else {
            self.rect.set_color(Color::rgba(35, 35, 35, 255));
        }
    }

    pub fn set_select(&mut self, is_select: bool) {
        if self.is_select == is_select {
            return;
        }
        self.is_select = is_select;
        if self.is_select {
            self.rect.set_color(Color::rgba(20, 20, 20, 255));
        } else {
            if self.is_hover {
                self.rect.set_color(Color::rgba(55, 55, 55, 255));
            } else {
                self.rect.set_color(Color::rgba(35, 35, 35, 255));
            }
        }
    }
}

pub struct SelectionBox {
    pub button: Image,
    pub rect: Vec<Rect>,
    pub text: Text,
    pub list_text: Vec<ListText>,
    pub list: Vec<String>,
    pub scrollbar: Scrollbar,
    pub is_list_visible: bool,

    is_hover: bool,
    is_click: bool,
    pub selected_index: usize,
    list_exceed: bool,

    start_index: usize,
}

impl SelectionBox {
    pub fn new(draw_setting: &mut DrawSetting, pos: Vec2, z_order: [f32; 6], width: f32, list: Vec<String>) -> Self {
        let mut rect = vec![
            Rect::new(&mut draw_setting.renderer, 0),
            Rect::new(&mut draw_setting.renderer, 0)
        ];
        // Dropdown Box
        rect[0].set_position(Vec3::new(pos.x, pos.y, z_order[0]))
            .set_size(Vec2::new(width - 21.0, 24.0))
            .set_border_width(1.0)
            .set_border_color(Color::rgba(20, 20, 20, 255))
            .set_color(Color::rgba(35, 35, 35, 255))
            .set_use_camera(true);

        // Dropdown Box Image
        let mut button = Image::new(Some(draw_setting.resource.selection_drop_button.allocation),
                &mut draw_setting.renderer, 0);
        button.pos = Vec3::new(pos.x + (width - 22.0), pos.y, z_order[0]);
        button.hw = Vec2::new(22.0, 24.0);
        button.uv = Vec4::new(0.0, 0.0, 22.0, 24.0);

        // List
        let visible_list = list.len().min(MAX_VISIBLE_LIST);
        let list_size = 4.0 + (20.0 * visible_list as f32);
        let mut list_text = Vec::new();
        let list_exceed = if list.len() > MAX_VISIBLE_LIST { true } else { false };

        let left_over = if list.len() > MAX_VISIBLE_LIST { list.len() - MAX_VISIBLE_LIST } else { 0 };
        let scrollbar = Scrollbar::new(draw_setting,
            Vec3::new(pos.x + width - 13.0, pos.y - 6.0, z_order[5]), left_over, 90, 20);

        rect[1].set_position(Vec3::new(pos.x, pos.y - (list_size - 1.0), z_order[2]))
            .set_size(Vec2::new(width, list_size))
            .set_border_width(1.0)
            .set_border_color(Color::rgba(20, 20, 20, 255))
            .set_color(Color::rgba(35, 35, 35, 255))
            .set_use_camera(true);

        for index in 0..visible_list {
            let lpos = Vec2::new(pos.x + 4.0, pos.y - 22.0 - (20.0 * index as f32));

            let mut lrect = Rect::new(&mut draw_setting.renderer, 0);
            lrect.set_position(Vec3::new(lpos.x - 2.0, lpos.y + 1.0, z_order[3]))
                .set_color(Color::rgba(35, 35, 35, 255))
                .set_use_camera(true);
            if list_exceed {
                lrect.set_size(Vec2::new(width - 17.0, 20.0));
            } else {
                lrect.set_size(Vec2::new(width - 4.0, 20.0));
            }
            
            let mut ltext = create_basic_label(draw_setting, 
                Vec3::new(lpos.x, lpos.y, z_order[4]),
                Vec2::new(width - 20.0, 20.0), Color::rgba(180, 180, 180, 255));
            ltext.set_text(&mut draw_setting.renderer, &list[index], Attrs::new());

            list_text.push(ListText { rect: lrect, text: ltext, is_hover: false, is_select: false });
        }
        
        // Selected Data Text
        let mut text = create_basic_label(draw_setting, 
                Vec3::new(pos.x + 4.0, pos.y + 1.0, z_order[1]), Vec2::new(width - 26.0, 20.0),
                Color::rgba(180, 180, 180, 255));
        text.set_text(&mut draw_setting.renderer, &list[0], Attrs::new());

        Self {
            button,
            rect,
            text,
            list_text,
            list,
            scrollbar,
            is_list_visible: false,
            is_hover: false,
            is_click: false,
            selected_index: 0,
            list_exceed,
            start_index: 0,
        }
    }

    pub fn update_list(&mut self, renderer: &mut GpuRenderer, start_pos: usize) {
        if self.start_index == start_pos || !self.list_exceed {
            return;
        }
        self.start_index = start_pos;
        for index in 0..MAX_VISIBLE_LIST {
            let list_index = index + self.start_index;
            if list_index < self.list.len() {
                if self.selected_index == list_index {
                    self.list_text[index].set_select(true);
                } else {
                    self.list_text[index].set_select(false);
                }
                self.list_text[index].text.set_text(renderer,
                    &self.list[list_index], Attrs::new());
            }
        }
    }

    pub fn show_list(&mut self, renderer: &mut GpuRenderer) {
        self.is_list_visible = true;
        self.rect[1].changed = true;
        
        self.start_index = 0;
        for index in 0..MAX_VISIBLE_LIST {
            let list_index = index + self.start_index;
            if list_index < self.list.len() {
                if self.selected_index == list_index {
                    self.list_text[index].set_select(true);
                } else {
                    self.list_text[index].set_select(false);
                }
                self.list_text[index].text.set_text(renderer,
                    &self.list[list_index], Attrs::new());
            }
        }

        self.list_text.iter_mut().for_each(|list_text| {
            list_text.text.changed = true;
            list_text.rect.changed = true;
        });
        if self.list_exceed {
            self.scrollbar.show();
            reset_scrollbar(&mut self.scrollbar);
        }
    }

    pub fn hide_list(&mut self) {
        self.is_list_visible = false;
    }

    pub fn set_hover(&mut self, is_hover: bool) {
        if self.is_hover == is_hover {
            return;
        }
        self.is_hover = is_hover;
        if self.is_hover {
            self.rect[0].set_color(Color::rgba(55, 55, 55, 255));
            self.button.uv.y = 24.0;
            self.button.changed = true;
        } else {
            self.rect[0].set_color(Color::rgba(35, 35, 35, 255));
            self.button.uv.y = 0.0;
            self.button.changed = true;
        }
    }

    pub fn set_click(&mut self, is_click: bool) {
        if self.is_click == is_click {
            return;
        }
        self.is_click = is_click;
        if self.is_click {
            self.rect[0].set_color(Color::rgba(20, 20, 20, 255));
            self.button.uv.y = 48.0;
            self.button.changed = true;
        } else {
            if self.is_hover {
                self.rect[0].set_color(Color::rgba(55, 55, 55, 255));
                self.button.uv.y = 24.0;
                self.button.changed = true;
            } else {
                self.rect[0].set_color(Color::rgba(35, 35, 35, 255));
                self.button.uv.y = 0.0;
                self.button.changed = true;
            }
        }
    }

    pub fn click_list(&mut self, mouse_pos: Vec2) -> Option<usize> {
        let mut found_button = None;
        for (index, list_text) in self.list_text.iter().enumerate() {
            if (mouse_pos.x) >= list_text.rect.position.x
                && (mouse_pos.x) <= list_text.rect.position.x + list_text.rect.size.x
                && (mouse_pos.y) >= list_text.rect.position.y
                && (mouse_pos.y) <= list_text.rect.position.y + list_text.rect.size.y {
                found_button = Some(index);
            }
        }
        found_button
    }

    pub fn hover_list(&mut self, mouse_pos: Vec2) {
        // We check if buttons are within the mouse position
        self.list_text.iter_mut().for_each(|list_text| {
            if (mouse_pos.x) >= list_text.rect.position.x
                && (mouse_pos.x) <= list_text.rect.position.x + list_text.rect.size.x
                && (mouse_pos.y) >= list_text.rect.position.y
                && (mouse_pos.y) <= list_text.rect.position.y + list_text.rect.size.y {
                list_text.set_hover(true);
            } else {
                list_text.set_hover(false);
            }
        });
    }

    pub fn switch_list(&mut self, renderer: &mut GpuRenderer, index: usize) {
        let list_index = index + self.start_index;
        if list_index == self.selected_index {
            return;
        }
        self.selected_index = list_index;
        self.text.set_text(renderer, &self.list[self.selected_index], Attrs::new());
    }
}