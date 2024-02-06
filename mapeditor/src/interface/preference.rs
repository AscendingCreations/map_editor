pub mod keybind;

use graphics::*;
use cosmic_text::{Attrs, Metrics};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::BufReader;
use std::path::Path;

use winit::{
    event::*,
    keyboard::*,
};

pub use keybind::*;

use crate::{
    collection::ZOOM_LEVEL,
    gfx_order::*,
    interface::{
        button::*,
        scrollbar::*,
        checkbox::*,
    },
    DrawSetting,
};

use super::button;

pub const PREF_TAB_GENERAL: usize = 0;
pub const PREF_TAB_KEYBIND: usize = 1;

pub struct MenuButton {
    pub image: Rect,
    pub text: Text,
    is_selected: bool,
}

impl MenuButton {
    pub fn new(draw_setting: &mut DrawSetting, pos: Vec2, msg: &str) -> Self {
        let mut image = Rect::new(&mut draw_setting.renderer, 0);
        image.set_position(Vec3::new(pos.x, pos.y, ORDER_PREFERENCE_MENU_BUTTON))
            .set_size(Vec2::new(118.0, 20.0))
            .set_color(Color::rgba(50, 50, 50, 0))
            .set_use_camera(true);

        let mut text = create_label(draw_setting, 
                Vec3::new(pos.x + 2.0, pos.y, ORDER_PREFERENCE_MENU_BUTTON_TEXT),
                Vec2::new(118.0, 20.0),
                Bounds::new(pos.x, pos.y, pos.x + 120.0, pos.y + 20.0),
                Color::rgba(20, 20, 20, 255));
        text.set_text(&mut draw_setting.renderer, msg, Attrs::new());

        Self {
            image,
            text,
            is_selected: false,
        }
    }

    pub fn set_select(&mut self, is_selected: bool) {
        if self.is_selected == is_selected {
            return;
        }

        self.is_selected = is_selected;
        if self.is_selected {
            self.image.set_color(Color::rgba(50, 50, 50, 255));
            self.text.set_default_color(Color::rgba(180, 180, 180, 255));
        } else {
            self.image.set_color(Color::rgba(50, 50, 50, 0));
            self.text.set_default_color(Color::rgba(50, 50, 50, 255));
        }
    }
}

pub struct KeyList {
    pub text: Text,
    pub key_string: Text,
    pub key_button: Rect,
    is_hover: bool,
}

impl KeyList {
    pub fn new(draw_setting: &mut DrawSetting, pos: Vec2, msg: &str, keystr: &str) -> Self {
        let label_size = Vec2::new(100.0, 20.0);
        let mut text = create_label(draw_setting, Vec3::new(pos.x, pos.y, ORDER_PREFERENCE_KEYLIST_TEXT), label_size,
                    Bounds::new(pos.x, pos.y, pos.x + label_size.x, pos.y + label_size.y),
                    Color::rgba(180, 180, 180, 255));
        text.set_text(&mut draw_setting.renderer, msg, Attrs::new());

        let key_pos = Vec3::new(pos.x + 100.0, pos.y, ORDER_PREFERENCE_KEYLIST_TEXT);
        let key_label_size = Vec2::new(200.0, 20.0);
        let mut key_string = create_label(draw_setting, key_pos, key_label_size, 
                    Bounds::new(key_pos.x, key_pos.y, key_pos.x + key_label_size.x, key_pos.y + key_label_size.y),
                    Color::rgba(180, 180, 180, 255));
        key_string.set_text(&mut draw_setting.renderer, keystr, Attrs::new());

        let mut key_button = Rect::new(&mut draw_setting.renderer, 0);
        key_button.set_size(key_label_size)
            .set_position(Vec3::new(key_pos.x - 3.0, key_pos.y, ORDER_PREFERENCE_KEYLIST_BUTTON))
            .set_color(Color::rgba(50, 50, 50, 255))
            .set_use_camera(true);
        
        Self {
            text,
            key_string,
            key_button,
            is_hover: false,
        }
    }

    pub fn set_hover(&mut self, is_hover: bool) {
        if self.is_hover == is_hover {
            return;
        }

        self.is_hover = is_hover;
        if self.is_hover {
            self.key_button.set_color(Color::rgba(180, 180, 180, 255));
            self.key_string.set_default_color(Color::rgba(40, 40, 40, 255));
        } else {
            self.key_button.set_color(Color::rgba(50, 50, 50, 255));
            self.key_string.set_default_color(Color::rgba(180, 180, 180, 255));
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeybindData {
    pub key_code: Vec<Key>,
    pub key_code_modifier: Vec<[bool; 3]>,
}

impl KeybindData {
    pub fn default() -> Self {
        let mut key_code = Vec::new();
        let mut key_code_modifier = Vec::new();

        for key in 0..EditorKey::Count as usize {
            let keycode = match key {
                1 => Key::Character(SmolStr::new("s")), // Save
                2 => Key::Character(SmolStr::new("z")), // Undo
                3 => Key::Character(SmolStr::new("y")), // Redo
                4 => Key::Character(SmolStr::new("d")), // Draw
                5 => Key::Character(SmolStr::new("e")), // Erase
                6 => Key::Character(SmolStr::new("f")), // Fill
                7 => Key::Character(SmolStr::new("y")), // Eyetool
                _ => Key::Character(SmolStr::new("o")), // Load
            };
            let keycodemodifier = match key {
                1 => [true, false, false], // Save
                2 => [true, false, false], // Undo
                3 => [true, false, false], // Redo
                4 => [false, false, false], // Draw
                5 => [false, false, false], // Erase
                6 => [false, false, false], // Fill
                7 => [false, false, false], // Eyetool
                _ => [true, false, false], // Load
            };
            key_code.push(keycode);
            key_code_modifier.push(keycodemodifier);
        }

        Self {
            key_code,
            key_code_modifier,
        }
    }

    pub fn save_config(&self) -> Result<(), AscendingError> {
        let name = "./config.json".to_string();

        match OpenOptions::new().truncate(true).write(true).open(&name) {
            Ok(file) => {
                if let Err(e) = serde_json::to_writer_pretty(&file, self) {
                    Err(AscendingError::Other(OtherError::new(&format!("Serdes File Error Err {:?}", e))))
                } else {
                    Ok(())
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
            Err(e) => Err(AscendingError::Other(OtherError::new(&format!("Failed to open {}, Err {:?}", name, e)))),
        }
    }
}

pub fn create_config(data: &KeybindData) -> Result<(), AscendingError> {
    let name = "./config.json".to_string();

    match OpenOptions::new().write(true).create_new(true).open(&name) {
        Ok(file) => {
            if let Err(e) = serde_json::to_writer_pretty(&file, &data) {
                Err(AscendingError::Other(OtherError::new(&format!("Serdes File Error Err {:?}", e))))
            } else {
                Ok(())
            }
        }
        Err(ref e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
        Err(e) => Err(AscendingError::Other(OtherError::new(&format!("Failed to open {}, Err {:?}", name, e)))),
    }
}

pub fn load_config() -> Result<KeybindData, AscendingError> {
    if !is_config_exist() {
        let data = KeybindData::default();
        match create_config(&KeybindData::default()) {
            Ok(()) => return Ok(data),
            Err(e) => return Err(e),
        }
    }

    let name = "./config.json".to_string();
    match OpenOptions::new().read(true).open(&name) {
        Ok(file) => {
            let reader = BufReader::new(file);

            match serde_json::from_reader(reader) {
                Ok(data) => Ok(data),
                Err(e) => {
                    println!("Error {:?}", e);
                    Ok(KeybindData::default())
                }
            }
        }
        Err(e) => Err(AscendingError::Other(OtherError::new(&format!("Failed to open {}, Err {:?}", name, e)))),
    }
}

pub fn is_config_exist() -> bool {
    let name = "./config.json".to_string();
    Path::new(&name).exists()
}

pub enum SettingData {
    None,
    Checkbox(Checkbox)
}

pub struct Preference {
    pub is_open: bool,
    pub bg: Rect,
    pub window: [Rect; 4],
    pub buttons: [Button; 3],
    pub menu_button: Vec<MenuButton>,
    pub scrollbar: Scrollbar,
    reset_button: bool,
    pub selected_menu: usize,
    pub config_data: KeybindData,
    // General
    pub setting_data: Vec<SettingData>,
    // Keybind
    pub key_list: Vec<KeyList>,
    pub keywindow: KeybindWindow,
}

impl Preference {
    pub fn new(draw_setting: &mut DrawSetting) -> Self {
        // This image is for the transparent shadow that will render behind the preference
        let mut bg = Rect::new(&mut draw_setting.renderer, 0);
        bg.set_position(Vec3::new(0.0, 0.0, ORDER_PREFERENCE_SHADOW))
            .set_size(Vec2::new(draw_setting.size.width, draw_setting.size.height))
            .set_color(Color::rgba(0, 0, 0, 200))
            .set_use_camera(true);

        // This will contain all rect that will shape the preference window design
        let window_size = Vec2::new(500.0, 350.0);
        let window_pos = Vec2::new(((draw_setting.size.width / ZOOM_LEVEL) * 0.5) - (window_size.x * 0.5),
                ((draw_setting.size.height / ZOOM_LEVEL) * 0.5) - (window_size.y * 0.5)).floor();
        let mut window =
                [Rect::new(&mut draw_setting.renderer, 0), // Window
                Rect::new(&mut draw_setting.renderer, 0), // Menu BG
                Rect::new(&mut draw_setting.renderer, 0), // Content
                Rect::new(&mut draw_setting.renderer, 0)]; // Scrollbar BG
        window[0].set_size(window_size)
            .set_position(Vec3::new(window_pos.x, window_pos.y, ORDER_PREFERENCE_WINDOW))
            .set_radius(3.0)
            .set_border_color(Color::rgba(10, 10, 10, 255))
            .set_border_width(2.0)
            .set_color(Color::rgba(50,50,50,255))
            .set_use_camera(true);
        window[1].set_size(Vec2::new(120.0, window_size.y - 65.0))
            .set_position(Vec3::new(window_pos.x + 20.0, window_pos.y + 45.0, ORDER_PREFERENCE_MENU))
            .set_color(Color::rgba(100,100,100,255))
            .set_use_camera(true);
        window[2].set_size(Vec2::new(window_size.x - 170.0, window_size.y - 65.0))
            .set_position(Vec3::new(window_pos.x + 150.0, window_pos.y + 45.0, ORDER_PREFERENCE_MENU))
            .set_color(Color::rgba(70,70,70,255))
            .set_use_camera(true);
        let pos = Vec2::new(window[2].position.x + window[2].size.x - 10.0, window[2].position.y + 2.0);
        window[3].set_size(Vec2::new(8.0, window_size.y - 69.0))
            .set_position(Vec3::new(pos.x, pos.y, ORDER_PREFERENCE_MENU))
            .set_color(Color::rgba(50,50,50,255))
            .set_use_camera(true);

        // Buttons
        let button_x = window_pos.x + window_size.x - 20.0;
        let buttons = [
            Button::new(draw_setting, draw_setting.resource.preference_button.allocation, "Cancel",
                        Vec2::new(button_x - 80.0, window_pos.y + 15.0), Vec2::new(80.0, 22.0),
                        [ORDER_PREFERENCE_BUTTON, ORDER_PREFERENCE_BUTTON_TEXT], 2.0),
            Button::new(draw_setting, draw_setting.resource.preference_button.allocation, "Reset",
                        Vec2::new(button_x - 165.0, window_pos.y + 15.0), Vec2::new(80.0, 22.0),
                        [ORDER_PREFERENCE_BUTTON, ORDER_PREFERENCE_BUTTON_TEXT], 2.0),
            Button::new(draw_setting, draw_setting.resource.preference_button.allocation, "Save",
                        Vec2::new(button_x - 250.0, window_pos.y + 15.0), Vec2::new(80.0, 22.0),
                        [ORDER_PREFERENCE_BUTTON, ORDER_PREFERENCE_BUTTON_TEXT], 2.0),
        ];

        // Menu Buttons
        let button_y = window[1].position.y + (window_size.y - 65.0);
        let mut menu_button = vec![
            MenuButton::new(draw_setting,
                    Vec2::new(window_pos.x + 21.0, button_y - 21.0), "General"),
            MenuButton::new(draw_setting,
                    Vec2::new(window_pos.x + 21.0, button_y - 42.0), "Keybinds"),
        ];
        menu_button[0].set_select(true);

        // Scrollbar
        let scrollbar = Scrollbar::new(draw_setting,
                            Vec3::new(window[2].position.x + window[2].size.x - 11.0, 
                                            window[2].position.y + window[2].size.y - 5.0, ORDER_PREFERENCE_SCROLLBAR),
                            0, window[2].size.y as usize - 10, 20);
        
        // Keybind Window
        let keywindow = KeybindWindow::new(draw_setting);

        // Config data
        let config_data = match load_config() {
            Ok(data) => data,
            Err(_) => KeybindData::default(),
        };
        
        Self {
            is_open: false,
            bg,
            window,
            buttons,
            reset_button: false,
            menu_button,
            scrollbar,
            selected_menu: 0,
            setting_data: Vec::new(),
            key_list: Vec::new(),
            keywindow,
            config_data,
        }
    }

    pub fn open(&mut self) {
        self.is_open = true;
        self.bg.changed = true;
        self.window.iter_mut().for_each(|window| {
            window.changed = true;
        });
        self.buttons.iter_mut().for_each(|button| {
            button.image.changed = true;
            button.text.changed = true;
        });
        self.menu_button.iter_mut().for_each(|button| {
            button.image.changed = true;
            button.text.changed = true;
        });
        self.menu_button[self.selected_menu].set_select(false);
        self.menu_button[0].set_select(true);
        self.selected_menu = 0;
        self.scrollbar.show();
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.scrollbar.hide();
    }

    pub fn hover_buttons(&mut self, mouse_pos: Vec2) {
        if self.keywindow.is_open {
            return;
        }
        // We check if buttons are within the mouse position
        self.buttons.iter_mut().for_each(|button| {
            if (mouse_pos.x) >= button.image.pos.x
                && (mouse_pos.x) <= button.image.pos.x + button.image.hw.x
                && (mouse_pos.y) >= button.image.pos.y
                && (mouse_pos.y) <= button.image.pos.y + button.image.hw.y {
                button.set_hover(true);
            } else {
                button.set_hover(false);
            }
        });
        
        match self.selected_menu {
            PREF_TAB_KEYBIND => {
                self.key_list.iter_mut().for_each(|keylist| {
                    if (mouse_pos.x) >= keylist.key_button.position.x
                        && (mouse_pos.x) <= keylist.key_button.position.x + keylist.key_button.size.x
                        && (mouse_pos.y) >= keylist.key_button.position.y
                        && (mouse_pos.y) <= keylist.key_button.position.y + keylist.key_button.size.y {
                        keylist.set_hover(true);
                    } else {
                        keylist.set_hover(false);
                    }
                });
            },
            PREF_TAB_GENERAL => {
                self.setting_data.iter_mut().for_each(|setting| {
                    match setting {
                        SettingData::Checkbox(checkbox) => {
                            if (mouse_pos.x) >= checkbox.window[0].position.x
                                && (mouse_pos.x) <= checkbox.window[0].position.x + checkbox.window[0].size.x
                                && (mouse_pos.y) >= checkbox.window[0].position.y
                                && (mouse_pos.y) <= checkbox.window[0].position.y + checkbox.window[0].size.y {
                                checkbox.set_hover(true);
                            } else {
                                checkbox.set_hover(false);
                            }
                        }
                        _ => {},
                    }
                });
            },
            _ => {},
        }
    }

    // This function should be called when the mouse button is not being pressed
    // This check if a button has been clicked, if yes, it will reset their click status
    pub fn release_click(&mut self) {
        if !self.reset_button {
            return;
        }
        
        self.buttons.iter_mut().for_each(|button| {
            button.set_click(false);
        });
    }

    // This function check which buttons are within the click position and return the button index
    pub fn click_buttons(&mut self, mouse_pos: Vec2) -> Option<usize> {
        if self.keywindow.is_open {
            return None;
        }
        let mut found_button = None;
        for (index, button) in self.buttons.iter().enumerate() {
            if (mouse_pos.x) >= button.image.pos.x
                && (mouse_pos.x) <= button.image.pos.x + button.image.hw.x
                && (mouse_pos.y) >= button.image.pos.y
                && (mouse_pos.y) <= button.image.pos.y + button.image.hw.y {
                found_button = Some(index);
            }
        }
        if let Some(index) = found_button {
            self.buttons[index].set_click(true);
            self.reset_button = true; // This remind us that a button has been clicked and needed to be reset
        }
        found_button
    }

    pub fn select_menu_button(&mut self, mouse_pos: Vec2) -> bool {
        if self.keywindow.is_open {
            return false;
        }
        let mut found_button = None;
        for (index, button) in self.menu_button.iter().enumerate() {
            if (mouse_pos.x) >= button.image.position.x
                && (mouse_pos.x) <= button.image.position.x + button.image.size.x
                && (mouse_pos.y) >= button.image.position.y
                && (mouse_pos.y) <= button.image.position.y + button.image.size.y {
                found_button = Some(index);
            }
        }
        if let Some(index) = found_button {
            if self.selected_menu == index {
                return false;
            }
            self.menu_button[self.selected_menu].set_select(false);
            self.menu_button[index].set_select(true);
            self.selected_menu = index;
            return true;
        }
        false
    }

    pub fn select_keylist(&mut self, mouse_pos: Vec2) -> Option<usize> {
        if self.keywindow.is_open {
            return None;
        }
        let mut found_button = None;
        for (index, keylist) in self.key_list.iter().enumerate() {
            if (mouse_pos.x) >= keylist.key_button.position.x
                && (mouse_pos.x) <= keylist.key_button.position.x + keylist.key_button.size.x
                && (mouse_pos.y) >= keylist.key_button.position.y
                && (mouse_pos.y) <= keylist.key_button.position.y + keylist.key_button.size.y {
                found_button = Some(index);
            }
        }
        found_button
    }

    pub fn update_scroll(&mut self, _cur_value: usize) -> bool {
        // Scrollbar is not being used on any tabs, but it will be kept for future expansion
        false
    }

    pub fn update_list(&mut self) {
        // Scrollbar is not being used on any tabs, but it will be kept for future expansion
    }

    pub fn update_key_list(&mut self, draw_setting: &mut DrawSetting, key_index: usize) {
        let pos = Vec2::new(self.window[2].position.x + 10.0,
                                (self.window[2].position.y + self.window[2].size.y) - 30.0);
        let key_text = 
                    get_key_name(self.config_data.key_code[key_index].clone(), 
                                self.config_data.key_code_modifier[key_index].clone());
        self.key_list[key_index] = KeyList::new(draw_setting,
            Vec2::new(pos.x, pos.y - (key_index as f32 * 21.0)),
            EditorKey::as_str(key_index), &key_text);
    }

    pub fn reset_preference(&mut self, draw_setting: &mut DrawSetting) {
        // Reset data
        self.config_data = KeybindData::default();
        open_preference_tab(self, draw_setting);
    }
}

pub fn open_preference_tab(preference: &mut Preference, draw_setting: &mut DrawSetting) {
    let _key_pos = Vec2::new(preference.window[2].position.x,
                                preference.window[2].position.y - preference.window[2].size.y);
    match preference.selected_menu {
        PREF_TAB_KEYBIND => {
            preference.scrollbar.update_scroll_max_value(0);
            preference.key_list = Vec::with_capacity(EditorKey::Count as usize);
            for key in 0..EditorKey::Count as usize {
                let pos = Vec2::new(preference.window[2].position.x + 10.0,
                                        (preference.window[2].position.y + preference.window[2].size.y) - 30.0);
                let key_text = 
                    get_key_name(preference.config_data.key_code[key].clone(), 
                                preference.config_data.key_code_modifier[key].clone());
                let keylist = KeyList::new(draw_setting, 
                    Vec2::new(pos.x, pos.y - (key as f32 * 21.0)),
                                EditorKey::as_str(key), &key_text);
                preference.key_list.push(keylist);
            }
            preference.setting_data = vec![];
        } // Keybind
        PREF_TAB_GENERAL => {
            preference.scrollbar.update_scroll_max_value(0);
            preference.key_list = vec![];

            let pos: Vec2 = Vec2::new(preference.window[2].position.x + 10.0,
                (preference.window[2].position.y + preference.window[2].size.y) - 30.0);
            preference.setting_data = vec![
                SettingData::Checkbox(Checkbox::new(draw_setting, Vec2::new(pos.x, pos.y), "Hide FPS?", Vec2::new(preference.window[2].size.x - 30.0, 20.0),
                        [ORDER_PREFERENCE_SETTING_IMG1, ORDER_PREFERENCE_SETTING_IMG2, ORDER_PREFERENCE_SETTING_TEXT])),
                SettingData::Checkbox(Checkbox::new(draw_setting, Vec2::new(pos.x, pos.y - 21.0), "Hide Something?", Vec2::new(preference.window[2].size.x - 30.0, 20.0),
                        [ORDER_PREFERENCE_SETTING_IMG1, ORDER_PREFERENCE_SETTING_IMG2, ORDER_PREFERENCE_SETTING_TEXT])),
                SettingData::Checkbox(Checkbox::new(draw_setting, Vec2::new(pos.x, pos.y - 42.0), "Checkbox Test", Vec2::new(preference.window[2].size.x - 30.0, 20.0),
                        [ORDER_PREFERENCE_SETTING_IMG1, ORDER_PREFERENCE_SETTING_IMG2, ORDER_PREFERENCE_SETTING_TEXT])),
            ];
        } // General: Default
        _ => {}
    }
}

fn create_label(draw_setting: &mut DrawSetting,
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