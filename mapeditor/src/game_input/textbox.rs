use input::Key;
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, WindowButtons},
};

use crate::game_input::*;

pub fn is_numeric(char: &str) -> bool {
    char.trim().parse::<i64>().is_ok()
}

pub fn enter_numeric(text: &mut String,
                    event: &KeyEvent,
                    limit: usize,
                    can_be_negative: bool,)
{
    if !event.state.is_pressed() {
        return;
    }

    if event.physical_key == KeyCode::Backspace {
        text.pop();
    } else {
        if text.len() >= limit {
            return;
        }
        if let Some(char) = event.logical_key.to_text() {
            if is_numeric(char) {
                text.push_str(char);
            } else if char.contains('-') && can_be_negative {
                if text.len() == 0 {
                    text.push_str(char);
                }
            }
        }
    }
}