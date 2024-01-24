use graphics::*;
use cosmic_text::{Attrs, Metrics};
use winit::dpi::PhysicalSize;
use crate::resource::*;
use crate::collection::ZOOM_LEVEL;

mod tabtext;
mod tool;
mod tileset_list;
mod scrollbar;

use tabtext::*;
use tool::*;
use tileset_list::*;

pub const LABEL_FPS: usize = 0;
pub const LABEL_TILESET: usize = 1;

pub const TOOL_LOAD: usize = 0;
pub const TOOL_SAVE: usize = 1;
pub const TOOL_UNDO: usize = 2;
pub const TOOL_DRAW: usize = 3;
pub const TOOL_ERASE: usize = 4;
pub const TOOL_FILL: usize = 5;
pub const TOOL_EYEDROP: usize = 6;
pub const TAB_LAYER: usize = 7;
pub const TAB_ATTRIBUTE: usize = 8;
pub const TAB_PROPERTIES: usize = 9;
pub const BUTTON_TILESET: usize = 10;

const MAX_TOOL: usize = 7;
const MAX_SETTING_TAB: usize = 3;
const MAX_EXTRA_BUTTON: usize = 1;

pub struct Interface {
    pub bg_layout: Image,
    pub labels: Vec<Text>,
    pub buttons: Vec<Tool>,
    pub current_tool: usize,
    pub current_setting_tab: usize,
    reset_button: bool,
    pub tab_labels: Vec<TabText>,
    pub current_tab_data: u32,
    pub tileset_list: TilesetList,
}

impl Interface {
    pub fn new(resource: &TextureAllocation, renderer: &mut GpuRenderer, size: &PhysicalSize<f32>, scale: f64) -> Self {
        // Load the texture
        let mut bg_layout = Image::new(Some(resource.bg_layout.allocation), renderer, 1);

        // Setup the interface position, height, width, color and texture coordinate
        bg_layout.pos = Vec3::new(0.0, 0.0, 11.0);
        bg_layout.hw = Vec2::new(949.0, 802.0);
        bg_layout.uv = Vec4::new(0.0, 0.0, 949.0, 802.0);
        bg_layout.color = Color::rgba(255, 255, 255, 255);

        // Preparing labels
        let mut labels = Vec::with_capacity(1);

        // Prepare all labels that will be drawn in the interface
        let text = create_label(renderer, size, scale,
                            Vec3::new(221.0, 16.0, 1.0), 
                            Vec2::new(100.0, 16.0),
                            Color::rgba(120, 120, 120, 255)); // FPS
        labels.push(text);
        let text = create_label(renderer, size, scale,
                            Vec3::new(37.0, 770.0, 1.0), 
                            Vec2::new(152.0, 20.0),
                            Color::rgba(0, 0, 0, 255)); // Tileset Label
        labels.push(text);

        // Prepare Tools
        let mut buttons = Vec::with_capacity(10);

        let mut last_index = 0;
        let mut last_pos_x = 217.0;
        for index in last_index..MAX_TOOL {
            let mut tool = Tool {
                index,
                image: Image::new(Some(resource.tool_icon.allocation), renderer, 1),
                state: ButtonState::Normal,
                in_hover: false,
                in_click: false,
            };
            tool.image.pos = Vec3::new(last_pos_x, 760.0, 10.0);
            match index {
                TOOL_SAVE => { last_pos_x += 39.0; }
                TOOL_UNDO => { last_pos_x += 39.0; }
                _ => { last_pos_x += 32.0; }
            }
            tool.image.hw = Vec2::new(30.0, 30.0);
            tool.image.uv = Vec4::new(30.0 * index as f32, 0.0, 30.0, 30.0);
            tool.image.color = Color::rgba(255, 255, 255, 255);
            buttons.push(tool);

            last_index += 1;
        }
        // Tab Buttons
        for index in last_index..(MAX_TOOL + MAX_SETTING_TAB) {
            let mut button = Tool {
                index,
                image: Image::new(Some(resource.tab_icon.allocation), renderer, 1),
                state: ButtonState::Normal,
                in_hover: false,
                in_click: false,
            };
            button.image.pos = Vec3::new(10.0 + (65.0 * (index - MAX_TOOL) as f32), 332.0, 10.0);
            button.image.hw = Vec2::new(66.0, 34.0);
            button.image.uv = Vec4::new(66.0 * (index - MAX_TOOL) as f32, 0.0, 66.0, 34.0);
            button.image.color = Color::rgba(255, 255, 255, 255);
            buttons.push(button);
            last_index += 1;
        }
        // Extra Buttons
        for index in last_index..(MAX_TOOL + MAX_SETTING_TAB + MAX_EXTRA_BUTTON) {
            let mut button = Tool {
                index,
                image: Image::new(Some(resource.tileset_button.allocation), renderer, 1),
                state: ButtonState::Normal,
                in_hover: false,
                in_click: false,
            };
            button.image.pos = Vec3::new(10.0, 769.0, 10.0);
            button.image.hw = Vec2::new(202.0, 23.0);
            button.image.uv = Vec4::new(0.0, 0.0, 202.0, 23.0);
            button.image.color = Color::rgba(255, 255, 255, 255);
            buttons.push(button);
            last_index += 1;
        }

        // Tab labels
        let mut tab_labels = Vec::with_capacity(MapLayers::Count as usize);
        for index in 0..MapLayers::Count as usize {
            tab_labels.push(TabText::new(resource, renderer, size, scale, 
                MapLayers::as_str(index as u32), 
                Vec2::new(14.0, 298.0 - (21 * index) as f32)));
        }

        // Tileset List
        let tileset_list = TilesetList::new(resource, renderer, size, scale);

        // We set the intial data of gui settings
        buttons[TOOL_DRAW].set_state(ButtonState::Selected);
        buttons[TAB_LAYER].set_state(ButtonState::Selected);
        tab_labels[0].set_select(true); // Set Ground as selected

        labels[LABEL_TILESET].set_text(renderer, &resource.tilesheet[0].name, Attrs::new());

        // Completed! We can now pass the struct
        Self {
            bg_layout,
            labels,
            buttons,
            current_tool: TOOL_DRAW,
            current_setting_tab: TAB_LAYER,
            reset_button: false,
            tab_labels,
            current_tab_data: 0,
            tileset_list,
        }
    }

    pub fn hover_button(&mut self, mouse_pos: Vec2) {
        // We check if buttons are within the mouse position
        for index in 0..(MAX_TOOL + MAX_SETTING_TAB + MAX_EXTRA_BUTTON) {
            if (mouse_pos.x) >= self.buttons[index].image.pos.x
                && (mouse_pos.x) <= self.buttons[index].image.pos.x + self.buttons[index].image.hw.x
                && (mouse_pos.y) >= self.buttons[index].image.pos.y
                && (mouse_pos.y) <= self.buttons[index].image.pos.y + self.buttons[index].image.hw.y {
                self.buttons[index].set_hover(true);
            } else {
                self.buttons[index].set_hover(false);
            }
        }
    }

    // This function check which buttons are within the click position and return the tool index
    pub fn click_button(&mut self, mouse_pos: Vec2) -> Option<usize> {
        let found_tool = self.buttons.iter().find(|tool| {
            (mouse_pos.x) >= tool.image.pos.x
                && (mouse_pos.x) <= tool.image.pos.x + tool.image.hw.x
                && (mouse_pos.y) >= tool.image.pos.y
                && (mouse_pos.y) <= tool.image.pos.y + tool.image.hw.y
        })?;
        let tool_index = found_tool.index;
        self.buttons[tool_index].set_click(true);
        self.reset_button = true; // This remind us that a button has been clicked and needed to be reset
        Some(tool_index)
    }

    // This function should be called when the mouse button is not being pressed
    // This check if a tool has been clicked, if yes, it will reset their click status
    pub fn reset_button_click(&mut self) {
        if self.reset_button {
            self.buttons.iter_mut().for_each(|button| button.set_click(false));
        }
    }

    // This function help us switch the current tool that the editor is using
    pub fn set_tool(&mut self, tool_index: usize) {
        if self.current_tool != tool_index {
            self.buttons[self.current_tool].set_state(ButtonState::Normal);
            self.buttons[tool_index].set_state(ButtonState::Selected);
            self.current_tool = tool_index;
        }
    }

    // This function help us switch the map setting tab that the editor is using
    pub fn set_tab(&mut self, tab_index: usize) {
        if self.current_setting_tab != tab_index {
            self.buttons[self.current_setting_tab].set_state(ButtonState::Normal);
            self.buttons[tab_index].set_state(ButtonState::Selected);
            self.current_setting_tab = tab_index;
            match self.current_setting_tab {
                TAB_LAYER => {
                    for index in 0..MapLayers::Count as usize {
                        self.tab_labels[index].button.changed = true;
                        self.tab_labels[index].text.changed = true;
                    }
                },
                TAB_ATTRIBUTE => {},
                TAB_PROPERTIES => {},
                _ => {},
            }
        }
    }

    // We separate this from the button as this will not have a click state
    pub fn hover_tab_option(&mut self, mouse_pos: Vec2) {
        // We will check which tab is option so only the selected option button will be checked
        match self.current_setting_tab {
            TAB_LAYER => {
                for index in 0..MapLayers::Count as usize {
                    if (mouse_pos.x) >= self.tab_labels[index].button.pos.x
                        && (mouse_pos.x) <= self.tab_labels[index].button.pos.x + self.tab_labels[index].button.hw.x
                        && (mouse_pos.y) >= self.tab_labels[index].button.pos.y
                        && (mouse_pos.y) <= self.tab_labels[index].button.pos.y + self.tab_labels[index].button.hw.y {
                        self.tab_labels[index].set_hover(true);
                    } else {
                        self.tab_labels[index].set_hover(false);
                    }
                }
            },
            TAB_ATTRIBUTE => {},
            TAB_PROPERTIES => {},
            _ => {},
        }
    }

    pub fn click_tab_option(&mut self, mouse_pos: Vec2) -> Option<usize> {
        match self.current_setting_tab {
            TAB_LAYER => {
                self.tab_labels
                    .iter()
                    .enumerate()
                    .filter(|(index, label)| {
                        *index < MapLayers::Count as usize
                            && mouse_pos.x >= label.button.pos.x
                            && mouse_pos.x <= label.button.pos.x + label.button.hw.x
                            && mouse_pos.y >= label.button.pos.y
                            && mouse_pos.y <= label.button.pos.y + label.button.hw.y
                    })
                    .map(|(index, _)| index)
                    .next()
            },
            TAB_ATTRIBUTE => { None },
            TAB_PROPERTIES => { None },
            _ => { None },
        }
    }

    pub fn select_tab_option(&mut self, tab_index: usize) {
        if self.current_tab_data != tab_index as u32 {
            match self.current_setting_tab {
                TAB_LAYER => {
                    self.tab_labels[self.current_tab_data as usize].set_select(false);
                    self.tab_labels[tab_index].set_select(true);
                    self.current_tab_data = tab_index as u32;
                }
                TAB_ATTRIBUTE => {},
                TAB_PROPERTIES => {},
                _ => {},
            }
        }
    }

    // We will use this function to accurately get the selected option index
    // As the index was adjusted to match the position of the option on Vec
    pub fn get_tab_option_data(&mut self) -> u32 {
        match self.current_setting_tab {
            TAB_LAYER => { self.current_tab_data as u32 }
            TAB_ATTRIBUTE => { 0 as u32 },
            TAB_PROPERTIES => { 0 as u32 },
            _ => { 0 as u32 },
        }
    }
}

fn create_label(renderer: &mut GpuRenderer, 
                size: &PhysicalSize<f32>, 
                scale: f64,
                pos: Vec3,
                label_size: Vec2,
                color: Color,
) -> Text {
    let mut text = Text::new(
        renderer,
        Some(Metrics::new(16.0, 16.0).scale(scale as f32)),
        Vec3::new(pos.x * ZOOM_LEVEL, pos.y * ZOOM_LEVEL, pos.z), label_size, 1.0
    );
    text.set_buffer_size(renderer, size.width as i32, size.height as i32)
        .set_bounds(Some(Bounds::new(pos.x * ZOOM_LEVEL, pos.y * ZOOM_LEVEL, (pos.x + label_size.x) * ZOOM_LEVEL, (pos.y + label_size.y) * ZOOM_LEVEL)))
        .set_default_color(color);
    text
}