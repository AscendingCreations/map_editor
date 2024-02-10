pub mod dialog;
pub mod preference;
pub mod widgets;
mod tileset_list;

use cosmic_text::{Attrs, Metrics};
use indexmap::IndexMap;
use graphics::*;

pub use dialog::*;
pub use preference::*;
pub use widgets::*;
use tileset_list::*;

use crate::{
    collection::*,
    config,
    map::*,
    tileset::*,
    DrawSetting,
    GameInput,
    ConfigData,
};

// Labels
pub const LABEL_FPS: usize = 0;
pub const LABEL_TILESET: usize = 1;
pub const LABEL_MAPNAME: usize = 2;
pub const LABEL_TILEPOS: usize = 3;
pub const LABEL_OPT_HEADER_TEXT: usize = 4;

// Buttons
pub const TOOL_LOAD: usize = 0;
pub const TOOL_SAVE: usize = 1;
pub const TOOL_UNDO: usize = 2;
pub const TOOL_REDO: usize = 3;
pub const TOOL_DRAW: usize = 4;
pub const TOOL_ERASE: usize = 5;
pub const TOOL_FILL: usize = 6;
pub const TOOL_EYEDROP: usize = 7;
pub const TAB_LAYER: usize = 8;
pub const TAB_ATTRIBUTE: usize = 9;
pub const TAB_ZONE: usize = 10;
pub const TAB_PROPERTIES: usize = 11;
pub const BUTTON_TILESET: usize = 12;

const MAX_TOOL: usize = 8;
const MAX_SETTING_TAB: usize = 4;
const MAX_EXTRA_BUTTON: usize = 1;
pub const MAX_TAB_LABEL: usize = 14;
pub const MAX_LABEL: usize = 5;

pub struct Interface {
    pub bg_layout: Vec<Image>,
    pub labels: Vec<Text>,
    pub buttons: Vec<ToolButton>,
    pub current_tool: usize,
    pub tileset_list: TilesetList,
    pub current_tab: usize,
    reset_tool_button: bool,
    reset_button: bool,
    reset_selectionbox: bool,
    pub dialog: Option<Dialog>,
    pub preference: Preference,
    // Tab Contents
    pub current_tab_data: u32,
    pub current_selected_area: i32,
    pub tab_labels: Vec<TabText>,
    pub scrollbar_bg: Rect,
    pub scrollbar: Scrollbar,
    pub start_view: usize,
    pub tab_opt_bg: Vec<Rect>,
    pub editor_label: Vec<Text>,
    pub editor_textbox: Vec<Textbox>,
    pub editor_button: Vec<Button>,
    pub editor_selectionbox: Vec<SelectionBox>,
    pub selected_textbox: i32,
    pub selected_dropbox: i32,
}

impl Interface {
    pub fn new(systems: &mut DrawSetting, config_data: &mut ConfigData) -> Self {
        // Load the texture
        let mut bg_layout = vec![
            Image::new(Some(systems.resource.bg_layout.allocation), &mut systems.renderer, 0),
            Image::new(Some(systems.resource.mapview_bg.allocation), &mut systems.renderer, 0),
            Image::new(Some(systems.resource.tileset_bg.allocation), &mut systems.renderer, 0),
        ];
        bg_layout[0].pos = Vec3::new(0.0, 0.0, ORDER_BG);
        bg_layout[0].hw = Vec2::new(949.0, 802.0);
        bg_layout[0].uv = Vec4::new(0.0, 0.0, 949.0, 802.0);
        bg_layout[1].pos = Vec3::new(215.0, 35.0, ORDER_ALPHA_BG);
        bg_layout[1].hw = Vec2::new(724.0, 724.0);
        bg_layout[1].uv = Vec4::new(0.0, 0.0, 724.0, 724.0);
        bg_layout[2].pos = Vec3::new(11.0, 369.0, ORDER_ALPHA_BG);
        bg_layout[2].hw = Vec2::new(200.0, 400.0);
        bg_layout[2].uv = Vec4::new(0.0, 0.0, 200.0, 400.0);

        // This prepare most labels on the interface
        let mut labels = vec![
            create_basic_label(systems,
                Vec3::new(870.0, 767.0, ORDER_BG_LABEL), 
                Vec2::new(100.0, 16.0),
                Color::rgba(180, 180, 180, 255)), // FPS
            create_basic_label(systems,
                Vec3::new(37.0, 770.0, ORDER_BG_LABEL),
                Vec2::new(152.0, 20.0),
                Color::rgba(0, 0, 0, 255)), // Tileset Label
            create_basic_label(systems,
                Vec3::new(221.0, 13.0, ORDER_BG_LABEL), 
                Vec2::new(600.0, 20.0),
                Color::rgba(180, 180, 180, 255)), // Map Name
            create_basic_label(systems,
                Vec3::new(810.0, 13.0, ORDER_BG_LABEL), 
                Vec2::new(130.0, 20.0),
                Color::rgba(180, 180, 180, 255)), // Tile Pos
            create_basic_label(systems,
                Vec3::new(11.0, 768.0, ORDER_BG_LABEL),
                Vec2::new(202.0, 20.0),
                Color::rgba(180, 180, 180, 255)), // Opt Header Text
        ];

        let mut buttons = Vec::with_capacity(MAX_TOOL + MAX_SETTING_TAB + MAX_EXTRA_BUTTON);
        let mut last_pos_x = 185.0;

        // This will prepare most buttons on the interface
        for index in 0..(MAX_TOOL + MAX_SETTING_TAB + MAX_EXTRA_BUTTON) {
            let button = if index < MAX_TOOL {
                last_pos_x += if index == TOOL_UNDO || index == TOOL_DRAW { 39.0 } else { 32.0 };
                create_tool_button(
                    systems.resource.tool_icon.allocation,
                    &mut systems.renderer,
                    index,
                    Vec3::new(last_pos_x, 760.0, ORDER_BG_BUTTON),
                    Vec2::new(30.0, 30.0),
                    Vec4::new(30.0 * index as f32, 0.0, 30.0, 30.0),
                )
            } else if index < MAX_TOOL + MAX_SETTING_TAB {
                create_tool_button(
                    systems.resource.tab_icon.allocation,
                    &mut systems.renderer,
                    index,
                    Vec3::new(10.0 + (47.0 * (index - MAX_TOOL) as f32), 332.0, ORDER_BG_BUTTON),
                    Vec2::new(48.0, 34.0),
                    Vec4::new(48.0 * (index - MAX_TOOL) as f32, 0.0, 48.0, 34.0),
                )
            } else {
                create_tool_button(
                    systems.resource.tileset_button.allocation,
                    &mut systems.renderer,
                    index,
                    Vec3::new(10.0, 769.0, ORDER_BG_BUTTON),
                    Vec2::new(202.0, 23.0),
                    Vec4::new(0.0, 0.0, 202.0, 23.0),
                )
            };
            buttons.push(button);
        }

        // This prepare the selectable tab labels
        let mut tab_labels = Vec::with_capacity(MAX_TAB_LABEL);
        for index in 0..MAX_TAB_LABEL {
            tab_labels.push(TabText::new(systems, 
                Vec2::new(14.0, 298.0 - (21 * index) as f32)));
        }

        // This calculate the scrollable value that the scrollbar will have
        let mut scroll_amount = 0;
        if MAX_TAB_LABEL < MAX_ATTRIBUTE - 1 {
            scroll_amount = MAX_ATTRIBUTE - MAX_TAB_LABEL - 1;
        }
        // This will create the visual image of the scrollable area
        let mut scrollbar_bg = Rect::new(&mut systems.renderer, 0);
        scrollbar_bg.set_size(Vec2::new(8.0, 313.0))
                        .set_position(Vec3::new(200.0, 15.0, ORDER_TAB_SCROLLBAR_BG))
                        .set_color(Color::rgba(35, 35, 35, 255))
                        .set_use_camera(true);
        // This create the actual scrollbar
        let scrollbar = Scrollbar::new(systems,
                        Vec3::new(199.0, 326.0, ORDER_TAB_SCROLLBAR),
                        scroll_amount, 
                        309,
                        20);
        
        // Tileset List
        let tileset_list = TilesetList::new(systems);

        // Attributes Properties Window
        let mut tab_opt_bg = vec![Rect::new(&mut systems.renderer, 0), Rect::new(&mut systems.renderer, 0)];
        tab_opt_bg[0].set_size(Vec2::new(200.0, 422.0))
                    .set_position(Vec3::new(11.0, 369.0, ORDER_ATTRIBUTE_BG))
                    .set_color(Color::rgba(50,50,50,255))
                    .set_use_camera(true);
        tab_opt_bg[1].set_size(Vec2::new(200.0, 24.0))
                    .set_position(Vec3::new(11.0, 767.0, ORDER_ATTRIBUTE_HEADER_BG))
                    .set_color(Color::rgba(25,25,25,255))
                    .set_use_camera(true);

        // Preference
        let mut preference = Preference::new(systems);
        open_preference_tab(&mut preference, systems, config_data);

        // We set the intial data of gui settings
        buttons[TOOL_DRAW].set_state(ButtonState::Selected);
        buttons[TAB_LAYER].set_state(ButtonState::Selected);
        labels[LABEL_TILESET].set_text(&mut systems.renderer, &systems.resource.tilesheet[0].name, Attrs::new());
        labels[LABEL_MAPNAME].set_text(&mut systems.renderer, "Map [ X: 0 Y: 0 Group: 0 ]", Attrs::new());
        labels[LABEL_TILEPOS].set_text(&mut systems.renderer, "Tile [ X: 32 Y: 32 ]", Attrs::new());
        for index in 0..MapLayers::Count as usize {
            tab_labels[index].init(&mut systems.renderer, MapLayers::as_str(index as u32), 194.0);
        }
        tab_labels[0].set_select(true); // Set Ground as selected

        // Completed! We can now pass the struct
        Self {
            bg_layout,
            labels,
            buttons,
            current_tool: TOOL_DRAW,
            current_tab: TAB_LAYER,
            reset_tool_button: false,
            reset_button: false,
            reset_selectionbox: false,
            tab_labels,
            current_tab_data: 0,
            tileset_list,
            dialog: None,
            preference,
            scrollbar_bg,
            scrollbar,
            start_view: 0,
            current_selected_area: 0,
            tab_opt_bg,
            editor_label: Vec::new(),
            editor_textbox: Vec::new(),
            editor_button: Vec::new(),
            editor_selectionbox: Vec::new(),
            selected_textbox: -1,
            selected_dropbox: -1,
        }
    }

    pub fn hover_tool_button(&mut self, mouse_pos: Vec2) {
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
    pub fn click_tool_button(&mut self, mouse_pos: Vec2) -> Option<usize> {
        let found_tool = self.buttons.iter().find(|tool| {
            (mouse_pos.x) >= tool.image.pos.x
                && (mouse_pos.x) <= tool.image.pos.x + tool.image.hw.x
                && (mouse_pos.y) >= tool.image.pos.y
                && (mouse_pos.y) <= tool.image.pos.y + tool.image.hw.y
        })?;
        let tool_index = found_tool.index;
        self.buttons[tool_index].set_click(true);
        self.reset_tool_button = true; // This remind us that a button has been clicked and needed to be reset
        Some(tool_index)
    }

    // This function should be called when the mouse button is not being pressed
    // This check if a tool has been clicked, if yes, it will reset their click status
    pub fn reset_tool_button_click(&mut self) {
        if !self.reset_tool_button {
            return;
        }
        self.buttons.iter_mut().for_each(|button| button.set_click(false));
    }

    // This function help us switch the current tool that the editor is using
    pub fn set_tool(&mut self, tool_index: usize) {
        if self.current_tool != tool_index {
            self.buttons[self.current_tool].set_state(ButtonState::Normal);
            self.buttons[tool_index].set_state(ButtonState::Selected);
            self.current_tool = tool_index;
        }
    }

    pub fn hover_buttons(&mut self, mouse_pos: Vec2) {
        // We check if buttons are within the mouse position
        self.editor_button.iter_mut().for_each(|button| {
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
        
        self.editor_button.iter_mut().for_each(|button| {
            button.set_click(false);
        });
    }

    // This function check which buttons are within the click position and return the button index
    pub fn click_buttons(&mut self, mouse_pos: Vec2) -> Option<usize> {
        let mut found_button = None;
        for (index, button) in self.editor_button.iter().enumerate() {
            if (mouse_pos.x) >= button.image.pos.x
                && (mouse_pos.x) <= button.image.pos.x + button.image.hw.x
                && (mouse_pos.y) >= button.image.pos.y
                && (mouse_pos.y) <= button.image.pos.y + button.image.hw.y {
                found_button = Some(index);
            }
        }
        if let Some(index) = found_button {
            self.editor_button[index].set_click(true);
            self.reset_button = true; // This remind us that a button has been clicked and needed to be reset
        }
        found_button
    }

    pub fn hover_selectionbox(&mut self, mouse_pos: Vec2) {
        // We check if buttons are within the mouse position
        self.editor_selectionbox.iter_mut().for_each(|selection_box| {
            if (mouse_pos.x) >= selection_box.rect[0].position.x
                && (mouse_pos.x) <= selection_box.rect[0].position.x + selection_box.rect[0].size.x + 21.0
                && (mouse_pos.y) >= selection_box.rect[0].position.y
                && (mouse_pos.y) <= selection_box.rect[0].position.y + selection_box.rect[0].size.y {
                selection_box.set_hover(true);
            } else {
                selection_box.set_hover(false);
            }
        });
    }

    // This function should be called when the mouse button is not being pressed
    // This check if a button has been clicked, if yes, it will reset their click status
    pub fn release_selectionbox_click(&mut self) {
        if !self.reset_selectionbox {
            return;
        }
        
        self.editor_selectionbox.iter_mut().for_each(|selection_box| {
            selection_box.set_click(false);
        });
    }

    // This function check which buttons are within the click position and return the button index
    pub fn click_selectionbox(&mut self, mouse_pos: Vec2) -> Option<usize> {
        let mut found_button = None;
        for (index, selection_box) in self.editor_selectionbox.iter().enumerate() {
            if (mouse_pos.x) >= selection_box.rect[0].position.x
                && (mouse_pos.x) <= selection_box.rect[0].position.x + selection_box.rect[0].size.x + 21.0
                && (mouse_pos.y) >= selection_box.rect[0].position.y
                && (mouse_pos.y) <= selection_box.rect[0].position.y + selection_box.rect[0].size.y {
                found_button = Some(index);
            }
        }
        if let Some(index) = found_button {
            self.editor_selectionbox[index].set_click(true);
            self.reset_selectionbox = true; // This remind us that a button has been clicked and needed to be reset
        }
        found_button
    }

    // We separate this from the button as this will not have a click state
    pub fn hover_tab_option(&mut self, mouse_pos: Vec2) {
        // We will check which tab is option so only the selected option button will be checked
        match self.current_tab {
            TAB_LAYER | TAB_ATTRIBUTE | TAB_ZONE => {
                for index in 0..MAX_TAB_LABEL {
                    if (mouse_pos.x) >= self.tab_labels[index].button.pos.x
                        && (mouse_pos.x) <= self.tab_labels[index].button.pos.x + self.tab_labels[index].button.hw.x
                        && (mouse_pos.y) >= self.tab_labels[index].button.pos.y
                        && (mouse_pos.y) <= self.tab_labels[index].button.pos.y + self.tab_labels[index].button.hw.y
                        && self.tab_labels[index].is_visible {
                        self.tab_labels[index].set_hover(true);
                    } else {
                        self.tab_labels[index].set_hover(false);
                    }
                }
            },
            TAB_PROPERTIES => {},
            _ => {},
        }
    }

    pub fn click_tab_option(&mut self, mouse_pos: Vec2) -> Option<usize> {
        match self.current_tab {
            TAB_LAYER | TAB_ATTRIBUTE | TAB_ZONE => {
                self.tab_labels
                    .iter()
                    .enumerate()
                    .filter(|(index, label)| {
                        *index < MAX_TAB_LABEL as usize && label.is_visible
                            && mouse_pos.x >= label.button.pos.x
                            && mouse_pos.x <= label.button.pos.x + label.button.hw.x
                            && mouse_pos.y >= label.button.pos.y
                            && mouse_pos.y <= label.button.pos.y + label.button.hw.y
                    })
                    .map(|(index, _)| index)
                    .next()
            },
            TAB_PROPERTIES => { None },
            _ => { None },
        }
    }

    pub fn select_tab_option(&mut self, tab_index: usize) {
        if self.current_selected_area != tab_index as i32 {
            match self.current_tab {
                TAB_LAYER | TAB_ZONE => {
                    if self.tab_labels[tab_index].is_visible {
                        // We will unselect the previous selection and select the current selection
                        self.tab_labels[self.current_selected_area as usize].set_select(false);
                        self.tab_labels[tab_index].set_select(true);
                        self.current_tab_data = tab_index as u32;
                        self.current_selected_area = tab_index as i32;
                    }
                }
                TAB_ATTRIBUTE => {
                    if self.tab_labels[tab_index].is_visible {
                        // We will unselect the previous selection and select the current selection
                        // Note: On this part, since a scrollbar is available on Tab_attribute
                        // We must make sure that our current selected area is visible
                        if self.current_selected_area >= 0 {
                            self.tab_labels[self.current_selected_area as usize].set_select(false);
                        }
                        self.tab_labels[tab_index].set_select(true);
                        self.current_tab_data = (self.start_view + tab_index) as u32;
                        self.current_selected_area = tab_index as i32;
                    }
                }
                _ => {},
            }
        }
    }

    // We will use this function to accurately get the selected option index
    // As the index was adjusted to match the position of the option on Vec
    pub fn get_tab_option_data(&mut self) -> u32 {
        match self.current_tab {
            TAB_LAYER | TAB_ATTRIBUTE | TAB_ZONE=> { self.current_tab_data as u32 }
            TAB_PROPERTIES => { 0 as u32 },
            _ => { 0 as u32 },
        }
    }

    pub fn update_scroll(&mut self, renderer: &mut GpuRenderer, cur_value: usize) {
        if self.start_view == cur_value {
            return;
        }
        self.start_view = cur_value;

        // We use -1 value to reset our current selected area
        self.current_selected_area = -1;
        for index in 0..MAX_TAB_LABEL {
            let sel_index = self.start_view + index;
            if sel_index < MAX_ATTRIBUTE as usize - 1 {
                if self.current_tab_data == sel_index as u32 {
                    self.tab_labels[index].update(renderer, MapAttribute::as_str(sel_index as u32 + 1), true);
                    self.current_selected_area = index as i32;
                } else {
                    self.tab_labels[index].update(renderer, MapAttribute::as_str(sel_index as u32 + 1), false);
                }
            }
        }
    }

    pub fn open_zone_settings(&mut self, systems: &mut DrawSetting, mapview: &mut MapView) {
        let zone_index = self.current_tab_data;
        // Max NPC
        self.editor_textbox[0].input_text(&mut systems.renderer, mapview.map_zone_setting[zone_index as usize].max_npc.to_string()); // Max Npc
        // NPC
        for i in 0..5 {
            if mapview.map_zone_setting[zone_index as usize].npc_id[i].is_some() {
                self.editor_textbox[i + 1].input_text(&mut systems.renderer, mapview.map_zone_setting[zone_index as usize].npc_id[i].unwrap().to_string());
            } else {
                self.editor_textbox[i + 1].input_text(&mut systems.renderer, String::new());
            }
        }
    }

    pub fn select_textbox(&mut self, mouse_pos: Vec2) {
        if let Some(index) = self.editor_textbox.iter().position(|textbox| {
            (mouse_pos.x) >= textbox.image.position.x
                && (mouse_pos.x) <= textbox.image.position.x + textbox.image.size.x
                && (mouse_pos.y) >= textbox.image.position.y
                && (mouse_pos.y) <= textbox.image.position.y + textbox.image.size.y
        }) {
            if self.selected_textbox as usize == index {
                return;
            }

            if let Some(selected_textbox) = self.editor_textbox.get_mut(self.selected_textbox as usize) {
                selected_textbox.set_select(false);
            }
            self.editor_textbox[index].set_select(true);
            self.selected_textbox = index as i32;
        } else {
            if let Some(selected_textbox) = self.editor_textbox.get_mut(self.selected_textbox as usize) {
                selected_textbox.set_select(false);
            }
            self.selected_textbox = -1;
        }
    }

    pub fn get_attribute_setting(&mut self) -> MapAttribute {
        // We adjust the index of the selected attribute as the index 0 is for the walkable attribute
        // which is not available on the tab data selection
        let attribute = MapAttribute::convert_to_plain_enum(self.current_tab_data + 1);
        match attribute {
            MapAttribute::Warp(_,_,_,_,_) => {
                let (mut mx, mut my, mut mg, mut tx, mut ty) = 
                    (0 as i32, 0 as i32, 0 as u64, 0 as u32, 0 as u32);
                for (index, textbox) in self.editor_textbox.iter().enumerate() {
                    let value = textbox.data.parse::<i64>().unwrap_or_default();
                    match index {
                        1 => {my = value as i32;}
                        2 => {mg = value as u64;}
                        3 => {tx = value as u32;}
                        4 => {ty = value as u32;}
                        _ => {mx = value as i32;}
                    }
                }
                MapAttribute::Warp(mx, my, mg, tx, ty)
            },
            MapAttribute::Sign(_) => {
                let text = self.editor_textbox[0].data.clone();
                MapAttribute::Sign(text)
            }
            _ => attribute,
        }
    }

    pub fn open_dialog(&mut self, systems: &mut DrawSetting, dialogtype: DialogType, data: Option<IndexMap<String, bool>>) {
        if self.dialog.is_some() {
            return;
        }
        self.dialog = Some(Dialog::new(systems, dialogtype, data));
    }

    pub fn close_dialog(&mut self) {
        self.dialog = None;
    }
}

// Function to create a tool button
fn create_tool_button(resource: usize, renderer: &mut GpuRenderer, index: usize, pos: Vec3, hw: Vec2, uv: Vec4) -> ToolButton {
    let mut tool = ToolButton {
        index,
        image: Image::new(Some(resource), renderer, 1),
        state: ButtonState::Normal,
        in_hover: false,
        in_click: false,
    };
    tool.image.pos = pos;
    tool.image.hw = hw;
    tool.image.uv = uv;
    tool
}