use cosmic_text::{Attrs, Metrics};
use wgpu::core::device::resource;
use graphics::*;

use crate::{
    resource::*,
    interface::{
        label::*,
        scrollbar::*,
    },
    collection::*,
    DrawSetting,
};

const MAX_VISIBLE_LIST: u32 = 18;

pub struct SelectButton {
    pub image: Image,
    pub in_hover: bool,
    pub is_selected: bool,
}

impl SelectButton {
    pub fn set_hover(&mut self, in_hover: bool) {
        if self.in_hover != in_hover {
            self.in_hover = in_hover;
            if !self.is_selected {
                if in_hover {
                    self.image.uv.y = self.image.hw.y;
                } else {
                    self.image.uv.y = 0.0;
                }
                self.image.changed = true;
            }
        }
    }

    pub fn set_select(&mut self, is_select: bool) {
        if self.is_selected != is_select {
            self.is_selected = is_select;
            if is_select {
                self.image.uv.y = self.image.hw.y * 2.0;
            } else {
                self.image.uv.y = 0.0;
            }
            self.image.changed = true;
        }
    }
}

pub struct TilesetList {
    pub visible: bool,
    pub bg: Vec<Rect>,
    pub selection_buttons: Vec<SelectButton>,
    pub texts: Vec<Text>,
    start_view_index: usize,
    pub selected_tileset: usize,
    view_index: Option<usize>,
    pub scrollbar: Scrollbar,
}

impl TilesetList {
    pub fn new(systems: &mut DrawSetting) -> Self {
        let mut bg = vec![
            Rect::new(&mut systems.renderer, 0), Rect::new(&mut systems.renderer, 0)
        ];
        bg[0].set_size(Vec2::new(200.0, 400.0))
            .set_position(Vec3::new(11.0, 369.0, ORDER_TILESETLIST))
            .set_color(Color::rgba(50,50,50,255))
            .set_use_camera(true);
        bg[1].set_size(Vec2::new(8.0, 377.0))
            .set_position(Vec3::new(200.0, 381.0, ORDER_TILESETLIST_SCROLL_BG))
            .set_color(Color::rgba(30, 30, 30, 255))
            .set_use_camera(true);
        
        // Tileset List and Button
        // This limit the amount of item on the list if tileset count is lower than the visible count
        // Note: If the tileset count is more than the visible count, we will limit the items with the visible count
        let max_view = std::cmp::min(MAX_TILESHEET, MAX_VISIBLE_LIST) as usize;
        let mut texts = Vec::with_capacity(max_view);
        let mut selection_buttons = Vec::with_capacity(max_view);
        for index in 0..max_view {
            // Create the selectable buttons
            let mut button = SelectButton {
                image: Image::new(Some(systems.resource.tileset_list_select.allocation), &mut systems.renderer, 0),
                in_hover: false,
                is_selected: false,
            };
            button.image.pos = Vec3::new(bg[0].position.x + 3.0, bg[0].position.y + 369.0 - (21.0 * index as f32), ORDER_TILESETLIST_BUTTON);
            button.image.hw = Vec2::new(183.0, 20.0);
            button.image.uv = Vec4::new(0.0, 0.0, 183.0, 20.0);
            selection_buttons.push(button);

            // Create the text
            let mut text = create_basic_label(systems,
                        Vec3::new(bg[0].position.x + 7.0, bg[0].position.y + 369.0 - (21.0 * index as f32), ORDER_TILESETLIST_LABEL),
                        Vec2::new(100.0, 20.0),
                        Color::rgba(180, 180, 180, 255));
            text.set_text(&mut systems.renderer, &systems.resource.tilesheet[index].name, Attrs::new());
            texts.push(text);
        };

        // Scrollbar
        let scrollbar_value = MAX_TILESHEET.max(MAX_VISIBLE_LIST) - MAX_VISIBLE_LIST;
        let scrollbar = Scrollbar::new(systems,
            Vec3::new(bg[0].position.x + 188.0, bg[0].position.y + 389.0, ORDER_TILESETLIST_SCROLLBAR), scrollbar_value as usize, 377, 20);

        // We set the default selected tileset
        selection_buttons[0].set_select(true);

        Self {
            visible: false,
            bg,
            selection_buttons,
            texts,
            start_view_index: 0, // We will use this to adjust the visible item on the list
            selected_tileset: 0,
            view_index: Some(0),
            scrollbar,
        }
    }

    pub fn select_list(&mut self, mouse_pos: Vec2) -> bool {
        if !self.visible {
            return false;
        }
        if let Some(index) = self.selection_buttons.iter().position(|button| {
            mouse_pos.x >= button.image.pos.x && 
            mouse_pos.x <= button.image.pos.x + button.image.hw.x && 
            mouse_pos.y >= button.image.pos.y && 
            mouse_pos.y <= button.image.pos.y + button.image.hw.y
        }) {
            let tileset_index = self.start_view_index + index;
            if self.selected_tileset != tileset_index {
                if let Some(view_index) = self.view_index {
                    self.selection_buttons[view_index].set_select(false);
                }
                self.selection_buttons[index].set_select(true);
                self.selected_tileset = tileset_index;
                self.view_index = Some(index);
                return true;
            }
        }
        false
    }

    // We use this function to update the list when the start view index has been adjusted
    pub fn update_list(&mut self, renderer: &mut GpuRenderer, resource: &TextureAllocation) {
        if !self.visible {
            return;
        }
        self.view_index = None;
        let max_view = std::cmp::min(MAX_TILESHEET, MAX_VISIBLE_LIST) as usize;
        for index in 0..max_view {
            let tileset_index = index + self.start_view_index;
            if self.selected_tileset == tileset_index {
                self.selection_buttons[index].set_select(true);
                self.view_index = Some(index);
            } else {
                self.selection_buttons[index].set_select(false);
            }
            self.texts[index].set_text(renderer, &resource.tilesheet[tileset_index].name, Attrs::new());
        }
    }

    pub fn update_scroll(&mut self, scroll_index: usize) -> bool {
        if !self.visible {
            return false;
        }
        if self.start_view_index != scroll_index {
            self.start_view_index = scroll_index as usize;
            return true;
        }
        false
    }

    pub fn hover_selection(&mut self, mouse_pos: Vec2) {
        if !self.visible {
            return;
        }
        // We check if buttons are within the mouse position
        for index in 0..self.selection_buttons.len() {
            if (mouse_pos.x) >= self.selection_buttons[index].image.pos.x
                && (mouse_pos.x) <= self.selection_buttons[index].image.pos.x + self.selection_buttons[index].image.hw.x
                && (mouse_pos.y) >= self.selection_buttons[index].image.pos.y
                && (mouse_pos.y) <= self.selection_buttons[index].image.pos.y + self.selection_buttons[index].image.hw.y {
                self.selection_buttons[index].set_hover(true);
            } else {
                self.selection_buttons[index].set_hover(false);
            }
        }
    }

    pub fn show(&mut self) {
        if self.visible {
            return;
        }
        self.visible = true;
        self.bg[0].changed = true;
        self.bg[1].changed = true;
        self.scrollbar.show();
        self.selection_buttons.iter_mut().for_each(|button| {
            button.image.changed = true;
        });
        self.texts.iter_mut().for_each(|text| {
            text.changed = true;
        });
        self.scrollbar.images.iter_mut().for_each(|image| {
            image.changed = true;
        });
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.scrollbar.hide();
    }
}