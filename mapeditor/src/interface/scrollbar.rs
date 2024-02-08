use graphics::*;
use guillotiere::euclid::num::Floor;

use crate::DrawSetting;

enum TextureState {
    Normal,
    Hover,
    Click,
}

pub struct Scrollbar {
    pub images: Vec<Image>,
    pub in_hold: bool,
    pub cur_value: usize,
    in_hover: bool,
    max_value: usize,
    scrollbar_size: usize,
    hold_pos: f32,
    start_pos: usize,
    end_pos: usize,
    length: usize,
    max_scroll_size: usize,
    min_bar_size: usize,
    default_pos: Vec3,
    pub visible: bool,
}

impl Scrollbar {
    pub fn new(draw_setting: &mut DrawSetting, pos: Vec3, max_value: usize, max_scroll_size: usize, min_bar_size: usize) -> Self {
        let mut images = Vec::with_capacity(3);

        let mut scrollbar_size = (max_scroll_size / (max_value + 1)).floor();
        if scrollbar_size < min_bar_size { scrollbar_size = min_bar_size; }

        // Top Corner of Scrollbar
        let mut image = Image::new(Some(draw_setting.resource.scrollbar.allocation), &mut draw_setting.renderer, 1);
        image.pos = Vec3::new(pos.x, pos.y, pos.z);
        image.hw = Vec2::new(10.0, 4.0);
        image.uv = Vec4::new(0.0, 0.0, 10.0, 4.0);
        images.push(image);

        // Center of Scrollbar
        let mut image = Image::new(Some(draw_setting.resource.scrollbar.allocation), &mut draw_setting.renderer, 1);
        image.pos = Vec3::new(pos.x, pos.y - scrollbar_size as f32, pos.z);
        image.hw = Vec2::new(10.0, scrollbar_size as f32);
        image.uv = Vec4::new(0.0, 5.0, 10.0, 6.0);
        images.push(image);

        // Bottom Corner of Scrollbar
        let mut image = Image::new(Some(draw_setting.resource.scrollbar.allocation), &mut draw_setting.renderer, 1);
        image.pos = Vec3::new(pos.x, pos.y - scrollbar_size as f32 - 4.0, pos.z);
        image.hw = Vec2::new(10.0, 4.0);
        image.uv = Vec4::new(0.0, 12.0, 10.0, 4.0);
        images.push(image);

        let start_pos = pos.y as usize;
        let end_pos = pos.y as usize - (max_scroll_size - scrollbar_size);
        let length = start_pos - end_pos;

        Self {
            images,
            in_hover: false,
            cur_value: 0,
            in_hold: false,
            max_value,
            scrollbar_size,
            hold_pos: 0.0,
            start_pos,
            end_pos,
            length,
            max_scroll_size,
            min_bar_size,
            default_pos: pos,
            visible: false,
        }
    }

    pub fn update_scroll_max_value(&mut self, max_value: usize) {
        if self.max_value == max_value {
            reset_scrollbar(self);
            return;
        }

        let mut scrollbar_size = (self.max_scroll_size / (max_value + 1)).floor();
        if scrollbar_size < self.min_bar_size { scrollbar_size = self.min_bar_size; }

        // Top Corner of Scrollbar
        self.images[0].pos = Vec3::new(self.default_pos.x, self.default_pos.y, self.default_pos.z);
        self.images[0].changed = true;

        // Center of Scrollbar
        self.images[1].pos = Vec3::new(self.default_pos.x, self.default_pos.y - scrollbar_size as f32, self.default_pos.z);
        self.images[1].hw = Vec2::new(10.0, scrollbar_size as f32);
        self.images[1].changed = true;

        // Bottom Corner of Scrollbar
        self.images[2].pos = Vec3::new(self.default_pos.x, self.default_pos.y - scrollbar_size as f32 - 4.0, self.default_pos.z);
        self.images[2].changed = true;

        // Reset data
        self.end_pos = self.default_pos.y as usize - (self.max_scroll_size - scrollbar_size);
        self.length = self.start_pos - self.end_pos;
        self.scrollbar_size = scrollbar_size;
        self.cur_value = 0;
        self.in_hover = false;
        self.in_hold = false;
        self.hold_pos = 0.0;
        self.max_value = max_value;
    }

    pub fn in_scrollbar(&mut self, mouse_pos: Vec2) -> bool {
        mouse_pos.x >= self.images[0].pos.x &&
            mouse_pos.x <= self.images[0].pos.x + self.images[0].hw.x &&
            mouse_pos.y >= self.images[2].pos.y &&
            mouse_pos.y <= self.images[0].pos.y + 4.0
    }

    pub fn hold_scrollbar(&mut self, pos_y: f32) {
        if !self.visible {
            return;
        }
        if !self.in_hold {
            self.in_hold = true;
            self.hold_pos = (self.images[0].pos.y + 4.0) - pos_y;
            set_texture_state(&mut self.images, TextureState::Click);
        }
    }

    pub fn release_scrollbar(&mut self) {
        if self.in_hold {
            self.in_hold = false;
            if self.in_hover {
                set_texture_state(&mut self.images, TextureState::Hover);
            } else {
                set_texture_state(&mut self.images, TextureState::Normal);
            }
        }
    }

    pub fn set_hover(&mut self, mouse_pos: Vec2) {
        if !self.visible {
            return;
        }
        self.in_hover = mouse_pos.x >= self.images[0].pos.x &&
                mouse_pos.x <= self.images[0].pos.x + self.images[0].hw.x &&
                mouse_pos.y >= self.images[2].pos.y &&
                mouse_pos.y <= self.images[0].pos.y + 4.0;
        
        if !self.in_hold {
            if self.in_hover {
                set_texture_state(&mut self.images, TextureState::Hover);
            } else {
                set_texture_state(&mut self.images, TextureState::Normal);
            }
        }
    }

    pub fn move_scrollbar(&mut self, pos_y: f32, forced: bool) {
        if !forced {
            if !self.in_hold || !self.visible {
                return;
            }
        }

        let mut y = pos_y + self.hold_pos;
        y = y.clamp(self.end_pos as f32, self.start_pos as f32);

        self.images[0].pos.y = y;
        self.images[0].changed = true;
        self.images[1].pos.y = y - self.scrollbar_size as f32;
        self.images[1].changed = true;
        self.images[2].pos.y = y - self.scrollbar_size as f32 - 4.0;
        self.images[2].changed = true;

        // Calculate the current value
        self.cur_value = (((self.start_pos as f32 - y) / self.length as f32) * self.max_value as f32).floor() as usize;
    }

    pub fn show(&mut self) {
        self.visible = true;
        self.images.iter_mut().for_each(|image| {
            image.changed = true;
        });
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }
}

pub fn reset_scrollbar(scrollbar: &mut Scrollbar) {
    scrollbar.move_scrollbar(scrollbar.start_pos as f32, true);
    scrollbar.set_hover(Vec2::new(0.0, 0.0));
}

fn set_texture_state(image: &mut Vec<Image>, state: TextureState) {
    match state {
        TextureState::Normal => {
            image[0].uv.y = 0.0; // Top
            image[0].changed = true;
            image[1].uv.y = 5.0; // Center
            image[1].changed = true;
            image[2].uv.y = 12.0; // Bottom
            image[2].changed = true;
        },
        TextureState::Hover => {
            image[0].uv.y = 16.0; // Top
            image[0].changed = true;
            image[1].uv.y = 21.0; // Center
            image[1].changed = true;
            image[2].uv.y = 28.0; // Bottom
            image[2].changed = true;
        },
        TextureState::Click => {
            image[0].uv.y = 32.0; // Top
            image[0].changed = true;
            image[1].uv.y = 37.0; // Center
            image[1].changed = true;
            image[2].uv.y = 44.0; // Bottom
            image[2].changed = true;
        },
    }
}