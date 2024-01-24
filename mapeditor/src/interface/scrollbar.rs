use graphics::*;
use guillotiere::euclid::num::Floor;

use crate::resource::*;

const MAX_SCROLL_SIZE: usize = 377;
const MIN_SIZE: usize = 20;

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
}

impl Scrollbar {
    pub fn new(resource: &TextureAllocation, renderer: &mut GpuRenderer, pos: Vec2, max_value: usize) -> Self {
        let mut images = Vec::with_capacity(3);

        let mut scrollbar_size = (MAX_SCROLL_SIZE / (max_value + 1)).floor();
        if scrollbar_size < MIN_SIZE { scrollbar_size = MIN_SIZE; }

        // Top Corner of Scrollbar
        let mut image = Image::new(Some(resource.scrollbar.allocation), renderer, 1);
        image.pos = Vec3::new(pos.x, pos.y, 3.0);
        image.hw = Vec2::new(10.0, 4.0);
        image.uv = Vec4::new(0.0, 0.0, 10.0, 4.0);
        image.color = Color::rgba(255, 255, 255, 255);
        images.push(image);

        // Center of Scrollbar
        let mut image = Image::new(Some(resource.scrollbar.allocation), renderer, 1);
        image.pos = Vec3::new(pos.x, pos.y - scrollbar_size as f32, 3.0);
        image.hw = Vec2::new(10.0, scrollbar_size as f32);
        image.uv = Vec4::new(0.0, 5.0, 10.0, 6.0);
        image.color = Color::rgba(255, 255, 255, 255);
        images.push(image);

        // Bottom Corner of Scrollbar
        let mut image = Image::new(Some(resource.scrollbar.allocation), renderer, 1);
        image.pos = Vec3::new(pos.x, pos.y - scrollbar_size as f32 - 4.0, 2.0);
        image.hw = Vec2::new(10.0, 4.0);
        image.uv = Vec4::new(0.0, 12.0, 10.0, 4.0);
        image.color = Color::rgba(255, 255, 255, 255);
        images.push(image);

        let start_pos = pos.y as usize;
        let end_pos = pos.y as usize - (MAX_SCROLL_SIZE - scrollbar_size);
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
        }
    }

    pub fn in_scrollbar(&mut self, mouse_pos: Vec2) -> bool {
        mouse_pos.x >= self.images[0].pos.x &&
            mouse_pos.x <= self.images[0].pos.x + self.images[0].hw.x &&
            mouse_pos.y >= self.images[2].pos.y &&
            mouse_pos.y <= self.images[0].pos.y + 4.0
    }

    pub fn hold_scrollbar(&mut self, pos_y: f32) {
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

    pub fn move_scrollbar(&mut self, pos_y: f32) {
        if !self.in_hold {
            return;
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