use std::thread::current;
use cosmic_text::{Attrs, Metrics};
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    keyboard::*,
    window::{WindowBuilder, WindowButtons},
};
use graphics::*;
use input::Key;

pub struct Textbox {
    pub image: Rect,
    pub text: Text,
    pub data: String,
    pub is_selected: bool,
}

impl Textbox {
    pub fn new(draw_setting: &mut DrawSetting, 
                textbox_pos: Vec3, 
                textbox_size: Vec2,
                can_wrap: bool) -> Self 
{
        let mut image = Rect::new(&mut draw_setting.renderer, 0);
        image.set_size(textbox_size)
            .set_position(textbox_pos)
            .set_border_color(Color::rgba(80, 80, 80, 255))
            .set_border_width(1.0)
            .set_color(Color::rgba(80,80,80,255))
            .set_use_camera(true);
        
        let mut text = Text::new(
            &mut draw_setting.renderer,
            Some(Metrics::new(16.0, 16.0).scale(draw_setting.scale as f32)),
            Vec3::new(textbox_pos.x + 2.0, textbox_pos.y - 2.0, textbox_pos.z), textbox_size, 1.0
        );
        text.set_buffer_size(&mut draw_setting.renderer, textbox_size.x as i32, draw_setting.size.height as i32)
            .set_bounds(Some(Bounds::new(textbox_pos.x, textbox_pos.y, 
                                        textbox_pos.x + textbox_size.x, textbox_pos.y + textbox_size.y)))
            .set_default_color(Color::rgba(200, 200, 200, 255))
            .set_text(&mut draw_setting.renderer, "", Attrs::new());
        if can_wrap {
            text.set_wrap(&mut draw_setting.renderer, cosmic_text::Wrap::Word);
        }
        
        Self {
            image,
            text,
            data: String::new(),
            is_selected: false,
        }
    }

    pub fn input_text(&mut self, renderer: &mut GpuRenderer, text: String) {
        self.data.clear();
        self.data.push_str(&text);
        self.text.set_text(renderer, &self.data, Attrs::new());
    }

    pub fn enter_numeric(&mut self, renderer: &mut GpuRenderer, event: &KeyEvent, limit: usize, can_be_negative: bool) {
        if !event.state.is_pressed() || !self.is_selected {
            return;
        }
    
        if event.physical_key == PhysicalKey::Code(KeyCode::Backspace) {
            self.data.pop();
        } else if event.physical_key == PhysicalKey::Code(KeyCode::Delete) {
            self.data.clear();
        } else {
            if self.data.len() >= limit {
                return;
            }
            if let Some(char) = event.logical_key.to_text() {
                if is_numeric(char) {
                    self.data.push_str(char);
                } else if char.contains('-') && can_be_negative {
                    if self.data.len() == 0 {
                        self.data.push_str(char);
                    }
                }
            }
        }

        self.text.set_text(renderer, &self.data, Attrs::new());
    }

    pub fn enter_text(&mut self, renderer: &mut GpuRenderer, event: &KeyEvent, limit: usize) {
        if !event.state.is_pressed() || !self.is_selected {
            return;
        }
        
        if event.physical_key == PhysicalKey::Code(KeyCode::Backspace) {
            self.data.pop();
        } else if event.physical_key == PhysicalKey::Code(KeyCode::Delete) {
            self.data.clear();
        } else {
            if self.data.len() >= limit {
                return;
            }
            if is_text(event) {
                if let Some(char) = event.logical_key.to_text() {
                    self.data.push_str(char);
                }
            }
        }

        self.text.set_text(renderer, &self.data, Attrs::new());
    }

    pub fn set_select(&mut self, is_select: bool) {
        if self.is_selected == is_select {
            return;
        }
        self.is_selected = is_select;
        if self.is_selected {
            self.image.set_border_color(Color::rgba(180,180,180,255));
        } else {
            self.image.set_border_color(Color::rgba(80,80,80,255));
        }
    }
}

use crate::{game_input::*, renderer, DrawSetting};

pub fn is_numeric(char: &str) -> bool {
    char.trim().parse::<i64>().is_ok()
}

pub fn is_text(event: &KeyEvent) -> bool {
    match event.physical_key {
        PhysicalKey::Code(
            KeyCode::KeyA | KeyCode::KeyB | KeyCode::KeyC | KeyCode::KeyD
            | KeyCode::KeyE | KeyCode::KeyF | KeyCode::KeyG | KeyCode::KeyH
            | KeyCode::KeyI | KeyCode::KeyJ | KeyCode::KeyK | KeyCode::KeyL
            | KeyCode::KeyM | KeyCode::KeyN | KeyCode::KeyO | KeyCode::KeyP
            | KeyCode::KeyQ | KeyCode::KeyR | KeyCode::KeyS | KeyCode::KeyT
            | KeyCode::KeyU | KeyCode::KeyV | KeyCode::KeyW | KeyCode::KeyX
            | KeyCode::KeyY | KeyCode::KeyZ | KeyCode::Digit1 | KeyCode::Digit2
            | KeyCode::Digit3 | KeyCode::Digit4 | KeyCode::Digit5 | KeyCode::Digit6
            | KeyCode::Digit7 | KeyCode::Digit8 | KeyCode::Digit9 | KeyCode::Digit0
            | KeyCode::Comma | KeyCode::Period | KeyCode::BracketLeft | KeyCode::BracketRight
            | KeyCode::Backquote | KeyCode::Minus | KeyCode::Equal | KeyCode::Quote
            | KeyCode::Backslash | KeyCode::Semicolon | KeyCode::Slash | KeyCode::Space
        ) => true,
        _ => false,
    }
}