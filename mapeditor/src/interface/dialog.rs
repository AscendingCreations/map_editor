use graphics::*;
use cosmic_text::{Attrs, Metrics};
use winit::dpi::PhysicalSize;
use crate::resource::*;
use crate::collection::ZOOM_LEVEL;
use crate::interface::scrollbar::*;
use indexmap::IndexMap;

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
    pub fn new(resource: &TextureAllocation, 
                renderer: &mut GpuRenderer, 
                size: &PhysicalSize<f32>, 
                scale: f64,
                message: &str,
                pos: Vec2,
                text_size: Vec2,
                button_type: DialogButtonType) -> Self {
        let mut image = Image::new(Some(resource.dialog_button.allocation), renderer, 1);
        image.pos = Vec3::new(pos.x, pos.y, 0.7);
        image.hw = Vec2::new(103.0, 36.0);
        image.uv = Vec4::new(0.0, 0.0, 103.0, 36.0);
        image.color = Color::rgba(255, 255, 255, 255);

        let adjust_x = 51.0 - (text_size.x * 0.5).floor();
        let mut text = create_label(renderer, size, scale,
            Vec3::new(pos.x + adjust_x, pos.y + 8.0, 0.6), 
            Vec2::new(text_size.x, text_size.y),
            Bounds::new(pos.x * ZOOM_LEVEL, (pos.y + 8.0) * ZOOM_LEVEL, (pos.x + 103.0) * ZOOM_LEVEL, (pos.y + 28.0) * ZOOM_LEVEL),
            Color::rgba(120, 120, 120, 255)); // FPS
        text.set_text(renderer, message, Attrs::new());
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
        } else {
            if !self.in_hover {
                self.image.uv.y = 0.0;
            } else {
                self.image.uv.y = 36.0;
            }
        }
        self.image.changed = true;
    }
}

pub struct Dialog {
    pub is_open: bool,
    pub dialog_type: DialogType,
    pub bg: Image,
    pub window: Rect,
    pub buttons: Vec<DialogButton>,
    pub message: Text,
    did_click: bool,
    // Content Data
    pub content_image: Vec<Rect>,
    pub content_text: Vec<Text>,
    pub editor_text: Vec<Text>,
    pub editor_data: Vec<String>,
    pub editing_index: usize,
    pub scrollbar: Scrollbar,
    start_view_index: usize, // Use for scrollbar
}

impl Dialog {
    pub fn new(resource: &TextureAllocation,
                renderer: &mut GpuRenderer,
                size: &PhysicalSize<f32>,
                scale: f64,
                dialog_type: DialogType,
                data: Option<IndexMap<String, bool>>) -> Self {
        // This image is for the transparent shadow that will render behind the dialog
        let mut bg: Image = Image::new(Some(resource.white.allocation), renderer, 1);
        bg.pos = Vec3::new(0.0, 0.0, 0.9);
        bg.hw = Vec2::new(size.width, size.height);
        bg.uv = Vec4::new(2.0, 2.0, 17.0, 17.0);
        bg.color = Color::rgba(0, 0, 0, 200);

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
        window_pos = Vec2::new((size.width * 0.5) - ((window_size.x * 0.5) * ZOOM_LEVEL),
                            (size.height * 0.5) - ((window_size.y * 0.5) * ZOOM_LEVEL)).floor();
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
                    DialogButton::new(resource, renderer, size, scale, "Yes", button_pos, Vec2::new(103.0, 20.0), DialogButtonType::ButtonConfirm),
                    DialogButton::new(resource, renderer, size, scale, "No", button_pos + Vec2::new(113.0, 0.0), Vec2::new(103.0, 20.0), DialogButtonType::ButtonCancel),
                ]
            }
            DialogType::TypeMapSave => {
                vec![
                    DialogButton::new(resource, renderer, size, scale, "Save", button_pos, Vec2::new(103.0, 20.0), DialogButtonType::ButtonConfirm),
                    DialogButton::new(resource, renderer, size, scale, "Don't Save", button_pos + Vec2::new(113.0, 0.0), Vec2::new(103.0, 20.0), DialogButtonType::ButtonDecline),
                    DialogButton::new(resource, renderer, size, scale, "Cancel", button_pos + Vec2::new(226.0, 0.0), Vec2::new(103.0, 20.0), DialogButtonType::ButtonCancel),
                ]
            }
            DialogType::TypeMapLoad => {
                vec![
                    DialogButton::new(resource, renderer, size, scale, "Load", button_pos, Vec2::new(103.0, 20.0), DialogButtonType::ButtonConfirm),
                    DialogButton::new(resource, renderer, size, scale, "Cancel", button_pos + Vec2::new(113.0, 0.0), Vec2::new(103.0, 20.0), DialogButtonType::ButtonCancel),
                ]
            }
            _ => {vec![]}
        };

        // This will be the dialog window
        let mut window = Rect::new(renderer, 0);
        window.set_size(window_size)
            .set_position(Vec3::new(window_pos.x, window_pos.y, 0.8))
            .set_radius(3.0)
            .set_border_color(Color::rgba(10, 10, 10, 255))
            .set_border_width(2.0)
            .set_color(Color::rgba(50,50,50,255));

        let msg = match dialog_type {
            DialogType::TypeExitConfirm => "Are you sure that you want to close the editor?",
            DialogType::TypeMapSave => "Would you like to save the changes to the following map/s?",
            DialogType::TypeMapLoad => "Please enter the map location that you would like to load",
            _ => "Error",
        };

        // Message
        let mut message = create_label(renderer, size, scale,
            Vec3::new(300.0, message_pos_y, 0.7), 
            Vec2::new(window_size.x, 20.0),
            Bounds::new(window_pos.x * ZOOM_LEVEL, message_pos_y * ZOOM_LEVEL, (window_pos.x + window_size.x) * ZOOM_LEVEL, (message_pos_y + 20.0) * ZOOM_LEVEL),
            Color::rgba(120, 120, 120, 255)); // FPS
        message.set_text(renderer, msg, Attrs::new());
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
                vec![String::new(); 3]
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
                let mut label_box = Rect::new(renderer, 0);
                label_box.set_size(label_box_size)
                        .set_position(Vec3::new(label_box_pos.x, label_box_pos.y, 0.75))
                        .set_color(Color::rgba(60, 60, 60, 255));

                let mut scrollbar_box = Rect::new(renderer, 0);
                scrollbar_box.set_size(Vec2::new(8.0, label_box_size.y - 4.0))
                        .set_position(Vec3::new(label_box.position.x + 354.0, label_box.position.y + 2.0, 0.7))
                        .set_color(Color::rgba(40, 40, 40, 255));
                vec![label_box, scrollbar_box]
            }
            DialogType::TypeMapLoad => {
                // Text Size = X[10] Y[10] Group[45]
                let textbox_total_size = 240.0; // [10][50][5][10][50][5][45][50]
                let content_pos = Vec2::new(window_pos.x + ((window_size.x * 0.5) - (textbox_total_size * 0.5)), window_pos.y + 66.0).floor();
                let mut mapx = Rect::new(renderer, 0);
                mapx.set_size(Vec2::new(50.0, 24.0))
                            .set_position(Vec3::new(content_pos.x + 15.0, content_pos.y, 0.7))
                            .set_border_color(Color::rgba(150, 150, 150, 255))
                            .set_border_width(1.0)
                            .set_color(Color::rgba(80,80,80,255));
                let mut mapy = Rect::new(renderer, 0);
                mapy.set_size(Vec2::new(50.0, 24.0))
                            .set_position(Vec3::new(content_pos.x + 85.0, content_pos.y, 0.7))
                            .set_border_color(Color::rgba(80, 80, 80, 255))
                            .set_border_width(1.0)
                            .set_color(Color::rgba(80,80,80,255));
                let mut mapgroup = Rect::new(renderer, 0);
                mapgroup.set_size(Vec2::new(50.0, 24.0))
                            .set_position(Vec3::new(content_pos.x + 190.0, content_pos.y, 0.7))
                            .set_border_color(Color::rgba(80, 80, 80, 255))
                            .set_border_width(1.0)
                            .set_color(Color::rgba(80,80,80,255));
                vec![mapx, mapy, mapgroup]
            }
            _ => { Vec::with_capacity(0) },
        };
        let content_text = match dialog_type {
            DialogType::TypeMapSave => {
                let mut data = Vec::with_capacity(4);
                for index in 0..4 {
                    let label_size = Vec2::new(362.0, 20.0);
                    let content_pos = Vec2::new(window_pos.x + ((window_size.x * 0.5) - (label_size.x * 0.5)).floor(), window_pos.y + 129.0 - (21.0 * index as f32)).floor();
                    let mut text = create_label(renderer, size, scale,
                        Vec3::new(content_pos.x, content_pos.y, 0.7), 
                        label_size,
                        Bounds::new(content_pos.x * ZOOM_LEVEL, content_pos.y * ZOOM_LEVEL, (content_pos.x + label_size.x - 14.0) * ZOOM_LEVEL, (content_pos.y + 20.0) * ZOOM_LEVEL),
                        Color::rgba(120, 120, 120, 255)); // X
                    if index < editor_data.len() {
                        text.set_text(renderer, &editor_data[index], Attrs::new());
                    } else {
                        text.set_text(renderer, "", Attrs::new());
                    }
                    data.push(text);
                }
                data
            },
            DialogType::TypeMapLoad => {
                // Text Size = X[10] Y[10] Group[45]
                let textbox_total_size = 240.0; // [10][5][50][5][10][5][50][5][45][5][50]
                let content_pos = Vec2::new(window_pos.x + ((window_size.x * 0.5) - (textbox_total_size * 0.5)), window_pos.y + 66.0).floor();
                let mut mapx = create_label(renderer, size, scale,
                    Vec3::new(content_pos.x, content_pos.y, 0.7), 
                    Vec2::new(window_size.x, 20.0),
                    Bounds::new(content_pos.x * ZOOM_LEVEL, content_pos.y * ZOOM_LEVEL, (content_pos.x + 10.0) * ZOOM_LEVEL, (content_pos.y + 20.0) * ZOOM_LEVEL),
                    Color::rgba(120, 120, 120, 255)); // X
                mapx.set_text(renderer, "X", Attrs::new());
                let mut mapy = create_label(renderer, size, scale,
                    Vec3::new(content_pos.x + 70.0, content_pos.y, 0.7), 
                    Vec2::new(window_size.x, 20.0),
                    Bounds::new((content_pos.x + 70.0) * ZOOM_LEVEL, content_pos.y * ZOOM_LEVEL, (content_pos.x + 80.0) * ZOOM_LEVEL, (content_pos.y + 20.0) * ZOOM_LEVEL),
                    Color::rgba(120, 120, 120, 255)); // Y
                mapy.set_text(renderer, "Y", Attrs::new());
                let mut mapgroup = create_label(renderer, size, scale,
                    Vec3::new(content_pos.x + 140.0, content_pos.y, 0.7), 
                    Vec2::new(window_size.x, 20.0),
                    Bounds::new((content_pos.x + 140.0) * ZOOM_LEVEL, content_pos.y * ZOOM_LEVEL, (content_pos.x + 185.0) * ZOOM_LEVEL, (content_pos.y + 20.0) * ZOOM_LEVEL),
                    Color::rgba(120, 120, 120, 255)); // Group
                mapgroup.set_text(renderer, "Group", Attrs::new());
                vec![mapx, mapy, mapgroup]
            },
            _ => { Vec::with_capacity(0) },
        };
        let editor_text = match dialog_type {
            DialogType::TypeMapSave => { Vec::with_capacity(0) },
            DialogType::TypeMapLoad => {
                // Text Size = X[10] Y[10] Group[45]
                let textbox_total_size = 240.0; // [10][5][50][5][10][5][50][5][45][5][50]
                let content_pos = Vec2::new(window_pos.x + ((window_size.x * 0.5) - (textbox_total_size * 0.5)), window_pos.y + 66.0).floor();
                let mut mapx = create_label(renderer, size, scale,
                    Vec3::new(content_pos.x + 17.0, content_pos.y, 0.6), 
                    Vec2::new(50.0, 20.0),
                    Bounds::new((content_pos.x + 15.0) * ZOOM_LEVEL, content_pos.y * ZOOM_LEVEL, (content_pos.x + 65.0) * ZOOM_LEVEL, (content_pos.y + 20.0) * ZOOM_LEVEL),
                    Color::rgba(200, 200, 200, 255)); // X
                mapx.set_text(renderer, "", Attrs::new());
                let mut mapy = create_label(renderer, size, scale,
                    Vec3::new(content_pos.x + 87.0, content_pos.y, 0.6), 
                    Vec2::new(50.0, 20.0),
                    Bounds::new((content_pos.x + 85.0) * ZOOM_LEVEL, content_pos.y * ZOOM_LEVEL, (content_pos.x + 135.0) * ZOOM_LEVEL, (content_pos.y + 20.0) * ZOOM_LEVEL),
                    Color::rgba(200, 200, 200, 255)); // Y
                mapy.set_text(renderer, "", Attrs::new());
                let mut mapgroup = create_label(renderer, size, scale,
                    Vec3::new(content_pos.x + 192.0, content_pos.y, 0.6), 
                    Vec2::new(50.0, 20.0),
                    Bounds::new((content_pos.x + 190.0) * ZOOM_LEVEL, content_pos.y * ZOOM_LEVEL, (content_pos.x + 240.0) * ZOOM_LEVEL, (content_pos.y + 20.0) * ZOOM_LEVEL),
                    Color::rgba(200, 200, 200, 255)); // Group
                mapgroup.set_text(renderer, "", Attrs::new());
                vec![mapx, mapy, mapgroup]
            },
            _ => { Vec::with_capacity(0) },
        };

        // Handle Scrollbar data
        let mut scrollbar_amount = 0;
        if dialog_type == DialogType::TypeMapSave && editor_data.len() > 4 {
            scrollbar_amount = editor_data.len() - 4;
        }
        let mut scrollbar = Scrollbar::new(resource, renderer, 
                            Vec3::new(scrollbar_x + 353.0, window_pos.y + 145.0, 0.5), 
                            scrollbar_amount, 75, 5);
        scrollbar.show();

        Self {
            is_open: false,
            dialog_type,
            bg,
            message,
            window,
            buttons,
            did_click: false,
            content_image,
            content_text,
            editor_text,
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

    pub fn update_editor_data(&mut self, renderer: &mut GpuRenderer) {
        if self.dialog_type != DialogType::TypeMapLoad {
            return;
        }
        self.editor_text[self.editing_index].set_text(renderer, &self.editor_data[self.editing_index], Attrs::new());
    }

    pub fn select_text(&mut self, mouse_pos: Vec2) {
        if self.dialog_type != DialogType::TypeMapLoad {
            return;
        }

        let mut selected_index = 0;
        for (index, textbox) in self.content_image.iter_mut().enumerate() {
            if (mouse_pos.x) >= textbox.position.x
                && (mouse_pos.x) <= textbox.position.x + textbox.size.x
                && (mouse_pos.y) >= textbox.position.y
                && (mouse_pos.y) <= textbox.position.y + textbox.size.y
            {
                textbox.set_border_color(Color::rgba(180,180,180,255));
                selected_index = index;
            } else {
                textbox.set_border_color(Color::rgba(80,80,80,255));
            }
        }
        self.editing_index = selected_index;
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

fn create_label(renderer: &mut GpuRenderer, 
    size: &PhysicalSize<f32>, 
    scale: f64,
    pos: Vec3,
    label_size: Vec2,
    bounds: Bounds,
    color: Color,
) -> Text {
    let mut text = Text::new(
        renderer,
        Some(Metrics::new(16.0, 16.0).scale(scale as f32)),
        Vec3::new(pos.x * ZOOM_LEVEL, pos.y * ZOOM_LEVEL, pos.z), label_size, 1.0
    );
    text.set_buffer_size(renderer, size.width as i32, size.height as i32)
            .set_bounds(Some(bounds))
            .set_default_color(color);
    text
}