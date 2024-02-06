use graphics::*;
use crate::interface::MAX_TOOL;

#[derive(PartialEq, Eq)]
pub enum ButtonState {
    Normal,
    Selected,
}

pub struct ToolButton {
    pub index: usize,
    pub image: Image,
    pub state: ButtonState,
    pub in_hover: bool,
    pub in_click: bool,
}

impl ToolButton {
    pub fn set_state(&mut self, state: ButtonState) {
        if self.state != state {
            self.state = state;
            match self.state {
                ButtonState::Normal => { self.image.uv.y = 0.0; }
                ButtonState::Selected => { self.image.uv.y = self.image.hw.y * 2.0; },
            }
            self.image.changed = true;
        }
    }

    pub fn set_hover(&mut self, hover: bool) {
        if self.in_hover != hover {
            self.in_hover = hover;
            if self.state == ButtonState::Normal {
                if self.in_hover {
                    self.image.uv.y = self.image.hw.y;
                } else {
                    self.image.uv.y = 0.0;
                }
                self.image.changed = true;
            }
        }
    }

    pub fn set_click(&mut self, click: bool) {
        if self.in_click != click {
            self.in_click = click;
            if self.state == ButtonState::Normal {
                if self.in_click {
                    self.image.uv.y = self.image.hw.y * 2.0;
                } else {
                    self.image.uv.y = self.image.hw.y;
                }
                self.image.changed = true;
            }
        }
    }
}