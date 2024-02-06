use graphics::*;
use cosmic_text::{Attrs, Metrics};

use crate::collection::ZOOM_LEVEL;

use crate::{
    gfx_order::*,
    interface::{
        button::*,
        scrollbar::*,
    },
    DrawSetting,
};

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

pub struct Preference {
    pub is_open: bool,
    pub bg: Rect,
    pub window: [Rect; 4],
    pub buttons: [Button; 3],
    pub menu_button: Vec<MenuButton>,
    pub scrollbar: Scrollbar,
    reset_button: bool,
    pub selected_menu: usize,
}

impl Preference {
    pub fn new(draw_setting: &mut DrawSetting) -> Self {
        // This image is for the transparent shadow that will render behind the preference
        let mut bg = Rect::new(&mut draw_setting.renderer, 0);
        bg.set_position(Vec3::new(0.0, 0.0, ORDER_PREFERENCE_SHADOW))
            .set_size(Vec2::new(draw_setting.size.width, draw_setting.size.height))
            .set_color(Color::rgba(0, 0, 0, 200))
            .set_use_camera(true);

        // This will consist all rect that will shape the preference window design
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
        
        Self {
            is_open: false,
            bg,
            window,
            buttons,
            reset_button: false,
            menu_button,
            scrollbar,
            selected_menu: 0,
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
        self.scrollbar.show();
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.scrollbar.hide();
    }

    pub fn hover_buttons(&mut self, mouse_pos: Vec2) {
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

    pub fn update_scroll(&mut self, _cur_value: usize) -> bool {
        false
    }

    pub fn update_list(&mut self) {

    }
}

pub fn open_preference_tab(preference: &mut Preference) {
    let _key_pos = Vec2::new(preference.window[2].position.x,
                                preference.window[2].position.y - preference.window[2].size.y);
    match preference.selected_menu {
        PREF_TAB_KEYBIND => {preference.scrollbar.update_scroll_max_value(5)} // Keybind
        _ => {preference.scrollbar.update_scroll_max_value(0)} // General: Default
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