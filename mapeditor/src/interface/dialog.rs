use graphics::*;
use cosmic_text::{Attrs, Metrics};
use indexmap::IndexMap;

use crate::collection::ZOOM_LEVEL;

use crate::{
    collection::*,
    interface::{
        scrollbar::*,
        textbox::*,
        label::*,
    },
    DrawSetting,
};

#[derive(Clone, PartialEq, Eq)]
pub enum DialogType {
    TypeNone,
    TypeExitConfirm,
    TypeMapSave,
    TypeMapLoad,
}

#[derive(Clone, PartialEq, Eq)]
pub enum DialogButtonType {
    ButtonNone,
    ButtonConfirm,
    ButtonDecline,
    ButtonCancel,
}

#[derive(Debug)]
pub enum DialogData {
    DataNone,
    MapLocation((i32, i32, i64)),
    MapList(IndexMap<String, (i32, i32, i64)>),
}

pub struct DialogButton {
    pub image: Image,
    pub text: Text,
    pub button_type: DialogButtonType,
    in_hover: bool,
    in_click: bool,
}

impl DialogButton {
    pub fn new(systems: &mut DrawSetting,
                message: &str,
                pos: Vec2,
                text_size: Vec2,
                button_type: DialogButtonType) -> Self {
        let mut image = Image::new(Some(systems.resource.dialog_button.allocation), &mut systems.renderer, 1);
        image.pos = Vec3::new(pos.x, pos.y, ORDER_DIALOG_BUTTON);
        image.hw = Vec2::new(103.0, 36.0);
        image.uv = Vec4::new(0.0, 0.0, 103.0, 36.0);

        let adjust_x = 51.0 - (text_size.x * 0.5).floor();
        let mut text = create_label(systems,
            Vec3::new(pos.x + adjust_x, pos.y + 8.0, ORDER_DIALOG_BUTTON_TEXT), 
            Vec2::new(text_size.x, text_size.y),
            Bounds::new(pos.x, pos.y + 8.0, pos.x + 103.0, pos.y + 28.0),
            Color::rgba(120, 120, 120, 255));
        text.set_text(&mut systems.renderer, message, Attrs::new());
        // Adjust text x position
        let message_size = text.measure();
        text.pos.x =  pos.x + (51.0 - (message_size.x * 0.5)).floor();
        text.changed = true;

        Self {
            image,
            text,
            button_type,
            in_hover: false,
            in_click: false,
        }
    }

    pub fn set_hover(&mut self, in_hover: bool) {
        if self.in_hover == in_hover {
            return;
        }
        self.in_hover = in_hover;
        if !self.in_click {
            if self.in_hover {
                self.image.uv.y = 36.0;
            } else {
                self.image.uv.y = 0.0;
            }
            self.image.changed = true;
        }
    }

    pub fn set_click(&mut self, in_click: bool) {
        if self.in_click == in_click {
            return;
        }
        self.in_click = in_click;
        if self.in_click {
            self.image.uv.y = 72.0;
            self.text.pos.y = self.image.pos.y + 6.0;
        } else {
            if !self.in_hover {
                self.image.uv.y = 0.0;
            } else {
                self.image.uv.y = 36.0;
            }
            self.text.pos.y = self.image.pos.y + 8.0;
        }
        self.image.changed = true;
        self.text.changed = true;
    }
}

pub struct Dialog {
    pub dialog_type: DialogType,
    pub bg: Rect,
    pub window: Rect,
    pub buttons: Vec<DialogButton>,
    pub message: Text,
    did_click: bool,
    // Content Data
    pub content_image: Vec<Rect>,
    pub content_text: Vec<Text>,
    pub editor_textbox: Vec<Textbox>,
    pub editor_data: Vec<String>,
    //pub editor_text: Vec<Text>,
    pub editing_index: usize,
    pub scrollbar: Scrollbar,
    start_view_index: usize, // Use for scrollbar
}

impl Dialog {
    pub fn new(systems: &mut DrawSetting,
                dialog_type: DialogType,
                data: Option<IndexMap<String, bool>>) -> Self {
        // This image is for the transparent shadow that will render behind the dialog
        let mut bg = Rect::new(&mut systems.renderer, 0);
        bg.set_position(Vec3::new(0.0, 0.0, ORDER_DIALOG_SHADOW))
            .set_size(Vec2::new(systems.size.width, systems.size.height))
            .set_color(Color::rgba(0, 0, 0, 200))
            .set_use_camera(true);

        // Window and button position/size calculations
        let window_size;
        let window_pos;
        let message_pos_y;
        let button_pos;

        window_size = Vec2::new(
            match dialog_type {
                DialogType::TypeExitConfirm => 384.0,
                DialogType::TypeMapSave => 456.0,
                DialogType::TypeMapLoad => 456.0,
                _ => { 384.0 },
            }, match dialog_type {
                DialogType::TypeExitConfirm => 108.0,
                DialogType::TypeMapSave => 201.0,
                DialogType::TypeMapLoad => 144.0,
                _ => { 108.0 },
            });
        window_pos = Vec2::new(((systems.size.width / ZOOM_LEVEL) * 0.5) - (window_size.x * 0.5),
                            ((systems.size.height / ZOOM_LEVEL) * 0.5) - (window_size.y * 0.5)).floor();
        message_pos_y = match dialog_type {
            DialogType::TypeExitConfirm => window_pos.y + 62.0,
            DialogType::TypeMapSave => window_pos.y + 155.0,
            DialogType::TypeMapLoad => window_pos.y + 98.0,
            _ => { 62.0 },
        };
        button_pos = Vec2::new(match dialog_type {
            DialogType::TypeExitConfirm => window_pos.x + 84.0,
            DialogType::TypeMapLoad => window_pos.x + 120.0,
            DialogType::TypeMapSave => window_pos.x + 64.0,
            _ => { window_pos.x + 84.0 },
        }, window_pos.y + 18.0);

        // Buttons
        let buttons = match dialog_type {
            DialogType::TypeExitConfirm => {
                vec![
                    DialogButton::new(systems, "Yes", button_pos, Vec2::new(103.0, 20.0), DialogButtonType::ButtonConfirm),
                    DialogButton::new(systems, "No", button_pos + Vec2::new(113.0, 0.0), Vec2::new(103.0, 20.0), DialogButtonType::ButtonCancel),
                ]
            }
            DialogType::TypeMapSave => {
                vec![
                    DialogButton::new(systems, "Save", button_pos, Vec2::new(103.0, 20.0), DialogButtonType::ButtonConfirm),
                    DialogButton::new(systems, "Don't Save", button_pos + Vec2::new(113.0, 0.0), Vec2::new(103.0, 20.0), DialogButtonType::ButtonDecline),
                    DialogButton::new(systems, "Cancel", button_pos + Vec2::new(226.0, 0.0), Vec2::new(103.0, 20.0), DialogButtonType::ButtonCancel),
                ]
            }
            DialogType::TypeMapLoad => {
                vec![
                    DialogButton::new(systems, "Load", button_pos, Vec2::new(103.0, 20.0), DialogButtonType::ButtonConfirm),
                    DialogButton::new(systems, "Cancel", button_pos + Vec2::new(113.0, 0.0), Vec2::new(103.0, 20.0), DialogButtonType::ButtonCancel),
                ]
            }
            _ => {vec![]}
        };

        // This will be the dialog window
        let mut window = Rect::new(&mut systems.renderer, 0);
        window.set_size(window_size)
            .set_position(Vec3::new(window_pos.x, window_pos.y, ORDER_DIALOG_WINDOW))
            .set_radius(3.0)
            .set_border_color(Color::rgba(10, 10, 10, 255))
            .set_border_width(2.0)
            .set_color(Color::rgba(50,50,50,255))
            .set_use_camera(true);

        let msg = match dialog_type {
            DialogType::TypeExitConfirm => "Are you sure that you want to close the editor?",
            DialogType::TypeMapSave => "Would you like to save the changes to the following map/s?",
            DialogType::TypeMapLoad => "Please enter the map location that you would like to load",
            _ => "Error",
        };

        // Message
        let mut message = create_label(systems,
            Vec3::new(300.0, message_pos_y, ORDER_DIALOG_MSG), 
            Vec2::new(window_size.x, 20.0),
            Bounds::new(window_pos.x, message_pos_y, window_pos.x + window_size.x, message_pos_y + 20.0),
            Color::rgba(120, 120, 120, 255)); // FPS
        message.set_text(&mut systems.renderer, msg, Attrs::new());
        // Adjust message x position based on message text
        let message_size = message.measure();
        message.pos.x = window_pos.x + ((window_size.x * 0.5) - (message_size.x * 0.5)).floor();
        message.changed = true;

        // Stored Data
        let editor_data = match dialog_type {
            DialogType::TypeMapSave => {
                let list_data = data.unwrap();
                let mut text_data = Vec::with_capacity(list_data.len());
                for (key, value) in list_data.iter() {
                    if *value {
                        text_data.push(key.clone());
                    }
                }
                text_data
            },
            DialogType::TypeMapLoad => {
                //vec![String::new(); 3]
                Vec::with_capacity(0)
            },
            _ => { Vec::with_capacity(0) },
        };

        // Content
        let mut scrollbar_x = window_pos.x;
        let content_image = match dialog_type {
            DialogType::TypeMapSave => {
                let label_box_size = Vec2::new(364.0, 85.0);
                let label_box_pos = Vec2::new(window_pos.x + ((window_size.x * 0.5) - (label_box_size.x * 0.5)).floor(), window_pos.y + 65.0);
                scrollbar_x = label_box_pos.x;
                let mut label_box = Rect::new(&mut systems.renderer, 0);
                label_box.set_size(label_box_size)
                        .set_position(Vec3::new(label_box_pos.x, label_box_pos.y, ORDER_DIALOG_CONTENT_IMG1))
                        .set_color(Color::rgba(60, 60, 60, 255))
                        .set_use_camera(true);

                let mut scrollbar_box = Rect::new(&mut systems.renderer, 0);
                scrollbar_box.set_size(Vec2::new(8.0, label_box_size.y - 4.0))
                        .set_position(Vec3::new(label_box.position.x + 354.0, label_box.position.y + 2.0, ORDER_DIALOG_CONTENT_IMG2))
                        .set_color(Color::rgba(40, 40, 40, 255))
                        .set_use_camera(true);
                vec![label_box, scrollbar_box]
            }
            _ => { Vec::with_capacity(0) },
        };
        let content_text = match dialog_type {
            DialogType::TypeMapSave => {
                let mut data = Vec::with_capacity(4);
                for index in 0..4 {
                    let label_size = Vec2::new(362.0, 20.0);
                    let content_pos = Vec2::new(window_pos.x + ((window_size.x * 0.5) - (label_size.x * 0.5)).floor(), window_pos.y + 129.0 - (21.0 * index as f32)).floor();
                    let mut text = create_label(systems,
                        Vec3::new(content_pos.x, content_pos.y, ORDER_DIALOG_CONTENT_TEXT), 
                        label_size,
                        Bounds::new(content_pos.x, content_pos.y, content_pos.x + label_size.x - 14.0, content_pos.y + 20.0),
                        Color::rgba(120, 120, 120, 255)); // X
                    if index < editor_data.len() {
                        text.set_text(&mut systems.renderer, &editor_data[index], Attrs::new());
                    } else {
                        text.set_text(&mut systems.renderer, "", Attrs::new());
                    }
                    data.push(text);
                }
                data
            },
            DialogType::TypeMapLoad => {
                // Text Size = X[10] Y[10] Group[45]
                let textbox_total_size = 240.0; // [10][5][50][5][10][5][50][5][45][5][50]
                let content_pos = Vec2::new(window_pos.x + ((window_size.x * 0.5) - (textbox_total_size * 0.5)), window_pos.y + 66.0).floor();
                let mut mapx = create_label(systems,
                    Vec3::new(content_pos.x, content_pos.y, ORDER_DIALOG_CONTENT_TEXT), 
                    Vec2::new(window_size.x, 20.0),
                    Bounds::new(content_pos.x, content_pos.y , content_pos.x + 10.0, content_pos.y + 20.0),
                    Color::rgba(120, 120, 120, 255)); // X
                mapx.set_text(&mut systems.renderer, "X", Attrs::new());
                let mut mapy = create_label(systems,
                    Vec3::new(content_pos.x + 70.0, content_pos.y, ORDER_DIALOG_CONTENT_TEXT), 
                    Vec2::new(window_size.x, 20.0),
                    Bounds::new(content_pos.x + 70.0, content_pos.y, content_pos.x + 80.0, content_pos.y + 20.0),
                    Color::rgba(120, 120, 120, 255)); // Y
                mapy.set_text(&mut systems.renderer, "Y", Attrs::new());
                let mut mapgroup = create_label(systems,
                    Vec3::new(content_pos.x + 140.0, content_pos.y, ORDER_DIALOG_CONTENT_TEXT), 
                    Vec2::new(window_size.x, 20.0),
                    Bounds::new(content_pos.x + 140.0, content_pos.y, content_pos.x + 185.0, content_pos.y + 20.0),
                    Color::rgba(120, 120, 120, 255)); // Group
                mapgroup.set_text(&mut systems.renderer, "Group", Attrs::new());
                vec![mapx, mapy, mapgroup]
            },
            _ => { Vec::with_capacity(0) },
        };
        
        // Textbox
        let editor_textbox = match dialog_type {
            DialogType::TypeMapLoad => {
                let textbox_total_size = 240.0; // [10][50][5][10][50][5][45][50]
                let content_pos = Vec2::new(window_pos.x + ((window_size.x * 0.5) - (textbox_total_size * 0.5)), window_pos.y + 66.0).floor();
                vec![
                    Textbox::new(systems, 
                                Vec3::new(content_pos.x + 15.0, content_pos.y, ORDER_DIALOG_CONTENT_IMG1),
                                Vec2::new(50.0, 24.0), false),
                    Textbox::new(systems, 
                                Vec3::new(content_pos.x + 85.0, content_pos.y, ORDER_DIALOG_CONTENT_IMG1),
                                Vec2::new(50.0, 24.0), false),
                    Textbox::new(systems, 
                                Vec3::new(content_pos.x + 190.0, content_pos.y, ORDER_DIALOG_CONTENT_IMG1),
                                Vec2::new(50.0, 24.0), false),
                ]
            }
            _ => { vec![] },
        };

        // Handle Scrollbar data
        let mut scrollbar_amount = 0;
        if dialog_type == DialogType::TypeMapSave && editor_data.len() > 4 {
            scrollbar_amount = editor_data.len() - 4;
        }
        let mut scrollbar = Scrollbar::new(systems, 
                            Vec3::new(scrollbar_x + 353.0, window_pos.y + 145.0, ORDER_DIALOG_SCROLLBAR), 
                            scrollbar_amount, 75, 5);
        scrollbar.show();

        Self {
            dialog_type,
            bg,
            message,
            window,
            buttons,
            did_click: false,
            content_image,
            content_text,
            editor_textbox,
            editor_data,
            editing_index: 0,
            scrollbar,
            start_view_index: 0,
        }
    }

    pub fn hover_buttons(&mut self, mouse_pos: Vec2) {
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

    pub fn release_click(&mut self) {
        if !self.did_click {
            return;
        }
        
        self.buttons.iter_mut().for_each(|button| {
            button.set_click(false);
        });
    }

    pub fn click_buttons(&mut self, mouse_pos: Vec2) -> DialogButtonType {
        let mut button_type = DialogButtonType::ButtonNone;
        if let Some(buttons) = self.buttons.iter_mut().find(|button| {
            (mouse_pos.x) >= button.image.pos.x
                && (mouse_pos.x) <= button.image.pos.x + button.image.hw.x
                && (mouse_pos.y) >= button.image.pos.y
                && (mouse_pos.y) <= button.image.pos.y + button.image.hw.y
        }) {
            buttons.set_click(true);
            button_type = buttons.button_type.clone();
        }
        if button_type != DialogButtonType::ButtonNone {
            self.did_click = true;
        }
        button_type
    }

    pub fn select_text(&mut self, mouse_pos: Vec2) {
        if self.dialog_type != DialogType::TypeMapLoad {
            return;
        }

        let last_selected = self.editing_index;
        let mut selected_index = -1;
        for (index, textbox) in self.editor_textbox.iter_mut().enumerate() {
            if (mouse_pos.x) >= textbox.image.position.x
                && (mouse_pos.x) <= textbox.image.position.x + textbox.image.size.x
                && (mouse_pos.y) >= textbox.image.position.y
                && (mouse_pos.y) <= textbox.image.position.y + textbox.image.size.y
            {
                textbox.set_select(true);
                selected_index = index as i32;
            } else {
                textbox.set_select(false);
            }
        }
        if selected_index < 0 {
            selected_index = last_selected as i32;
            self.editor_textbox[last_selected].set_select(false);
        }
        self.editing_index = selected_index as usize;
    }

    pub fn update_list(&mut self, renderer: &mut GpuRenderer) {
        for index in 0..4 {
            let text_index = index + self.start_view_index;
            if text_index < self.editor_data.len() {
                self.content_text[index].set_text(renderer, &self.editor_data[text_index], Attrs::new());
            }
        }
    }

    pub fn update_scroll(&mut self, scroll_index: usize) -> bool {
        if self.start_view_index != scroll_index {
            self.start_view_index = scroll_index as usize;
            return true;
        }
        false
    }
}