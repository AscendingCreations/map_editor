pub mod dialog_input;
mod preference_input;
mod mapview_input;
mod tileset_input;
mod interface_input;

use cosmic_text::{Attrs, Metrics};
use winit::{
    event::*,
    keyboard::*,
};
use graphics::*;

pub use dialog_input::*;
use preference_input::*;
use mapview_input::*;
use tileset_input::*;
use interface_input::*;

use crate::{
    collection::{TEXTURE_SIZE, ZOOM_LEVEL},
    config::*,
    interface::*,
    map::*,
    map_data::*,
    tileset::*,
    DrawSetting
};

pub enum InputType {
    MouseLeftDown,
    MouseLeftDownMove,
    MouseMove,
    MouseRelease,
}

#[derive(PartialEq, Eq)]
pub enum PressType {
    PressNone,
    PressTileset,
    PressMap,
}

pub struct GameInput {
    // General
    pub last_mouse_pos: (f32, f32),
    pub presstype: PressType,
    pub did_mouse_press: bool,
    pub mouse_release: bool,
    // Tileset selection
    pub tileset_start: Vec2,
    pub tileset_end: Vec2,
    pub selected_size: Vec2,
    // Map
    pub selected_link_map: Option<usize>,
    // Dialog
    pub dialog_button_press: bool,
    pub selected_dialog_type: DialogButtonType,
    // Shortcut
    pub hold_key_modifier: [bool; 3],
}

impl GameInput {
    pub fn new() -> Self {
        Self {
            last_mouse_pos: (0.0, 0.0),
            presstype: PressType::PressNone,
            did_mouse_press: false,
            mouse_release: false,
            tileset_start: Vec2::new(0.0, 0.0),
            tileset_end: Vec2::new(0.0, 0.0),
            selected_size: Vec2::new(1.0, 1.0),
            selected_link_map: None,
            dialog_button_press: false,
            selected_dialog_type: DialogButtonType::ButtonNone,
            hold_key_modifier: [false; 3],
        }
    }
}

pub fn handle_input(
    systems: &mut DrawSetting,
    inputtype: InputType,
    mouse_pos: &Vec2,
    gameinput: &mut GameInput,
    gui: &mut Interface,
    tileset: &mut Tileset,
    mapview: &mut MapView,
    database: &mut EditorData,
    config_data: &mut ConfigData,
    elwt: &winit::event_loop::EventLoopWindowTarget<()>,
) {
    // We convert the mouse position to render position as the y pos increase upward
    let screen_pos = Vec2::new(
        mouse_pos.x / ZOOM_LEVEL,
        (systems.size.height - mouse_pos.y) / ZOOM_LEVEL,
    );

    // If dialog open, cancel all other inputs
    if gui.dialog.is_some() {
        dialog_input(systems, 
                    &inputtype, 
                    screen_pos, 
                    gameinput,
                    gui,
                    database,
                    mapview,
                    elwt,);
        return;
    }
    // If preference is open, cancel all other inputs
    if gui.preference.is_open {
        preference_input(systems,
                    &inputtype,
                    screen_pos,
                    gui,
                    tileset,
                    mapview,
                    config_data);
        return;
    }

    // Handle Mapview Inputs
    mapview_input(systems,
        &inputtype,
        screen_pos,
        gameinput,
        gui,
        tileset,
        mapview,
        database);
    // Handle tileset inputs
    tileset_input(systems,
        &inputtype,
        screen_pos,
        gameinput,
        gui,
        tileset,
        mapview);
    
    // Handle interface inputs
    interface_input(systems,
        &inputtype,
        screen_pos,
        gameinput,
        gui,
        tileset,
        mapview,
        database,
        config_data);
}

pub fn handle_key_input(
    event: &KeyEvent,
    gui: &mut Interface,
    mapview: &mut MapView,
    database: &mut EditorData,
    systems: &mut DrawSetting,
) -> bool {
    if gui.preference.is_open {
        preference_key_input(&mut systems.renderer, event, gui);
        return true;
    }

    if let Some(dialog) = &mut gui.dialog {
        dialog_key_input(&mut systems.renderer, event, dialog);
        return true;
    }

    if !event.state.is_pressed() { return false; }

    interface_key_input(event,
        gui,
        mapview,
        database,
        systems,)
}

pub fn access_shortcut(event: &KeyEvent,
    systems: &mut DrawSetting,
                        gameinput: &mut GameInput,
                        database: &mut EditorData,
                        tileset: &mut Tileset,
                        mapview: &mut MapView,
                        gui: &mut Interface,
                        config_data: &mut ConfigData,) {
    let mut got_key = None;
    let mut key_modifier = [false; 3];

    if gui.dialog.is_some() || gui.preference.is_open {
        return;
    }
    
    // Read Input
    match event.physical_key {
        PhysicalKey::Code(KeyCode::ControlLeft) | PhysicalKey::Code(KeyCode::ControlRight) => 
            gameinput.hold_key_modifier[0] = event.state.is_pressed(),
        PhysicalKey::Code(KeyCode::ShiftLeft) | PhysicalKey::Code(KeyCode::ShiftRight) => 
            gameinput.hold_key_modifier[1] = event.state.is_pressed(),
        PhysicalKey::Code(KeyCode::Space) => 
            gameinput.hold_key_modifier[2] = event.state.is_pressed(),
        _ => {
            if is_valid_key_code(event) && event.state.is_pressed() {
                got_key = Some(event.logical_key.clone());
                key_modifier = gameinput.hold_key_modifier.clone();
                gameinput.hold_key_modifier = [false; 3];
            }
        }
    }

    // Handle Input Logic
    if let Some(keycode) = got_key {
        if let Some(got_index) = (0..EditorKey::Count as usize).find(|&index| {
            config_data.key_code[index] == keycode &&
            config_data.key_code_modifier[index] == key_modifier
        }) {
            let button_index = match got_index {
                1 => TOOL_SAVE,
                2 => TOOL_UNDO,
                3 => TOOL_REDO,
                4 => TOOL_DRAW,
                5 => TOOL_ERASE,
                6 => TOOL_FILL,
                7 => TOOL_EYEDROP,
                _ => TOOL_LOAD,
            };

            gui_button_select(button_index,
                systems,
                gameinput,
                gui,
                tileset,
                mapview,
                database,
                config_data);
        }
    }
}

pub fn gui_button_select(button_index: usize,
                        systems: &mut DrawSetting,
                        gameinput: &mut GameInput,
                        gui: &mut Interface,
                        tileset: &mut Tileset,
                        mapview: &mut MapView,
                        database: &mut EditorData,
                        config_data: &mut ConfigData,
) {
    match button_index {
        TOOL_LOAD => {
            if gui.preference.is_open {
                config_data.set_data(load_config());
                gui.preference.close();
            }
            gui.open_dialog(
                systems,
                DialogType::TypeMapLoad,
                None,
            );
        }
        TOOL_SAVE => {
            database.save_map_data(&mapview, None);
            update_map_name(systems, gui, database);
        }
        TOOL_UNDO => {
            mapview.apply_change(&mut systems.renderer, true);
        }
        TOOL_REDO => {
            mapview.apply_change(&mut systems.renderer, false);
        }
        TOOL_DRAW | TOOL_ERASE | TOOL_FILL | TOOL_EYEDROP => {
            gui.set_tool(button_index);
        }
        TAB_ATTRIBUTE | TAB_LAYER | TAB_PROPERTIES | TAB_ZONE => {
            set_tab(systems, gui, button_index, mapview, tileset, gameinput);
            if gui.tileset_list.visible {
                gui.tileset_list.hide();
            }
        }
        BUTTON_TILESET => {
            if gui.current_tab == TAB_LAYER {
                if gui.tileset_list.visible {
                    gui.tileset_list.hide();
                } else {
                    gui.tileset_list.show();
                }
            }
        }
        _ => {}
    }
}

pub fn update_map_name(
    systems: &mut DrawSetting,
    gui: &mut Interface,
    database: &EditorData,
) {
    if database.did_change(database.x, database.y, database.group) {
        gui.labels[LABEL_MAPNAME].set_text(
            &mut systems.renderer,
            &format!(
                "Map [ X: {} Y: {} Group: {} ] *",
                database.x, database.y, database.group
            ),
            Attrs::new(),
        );
    } else {
        gui.labels[LABEL_MAPNAME].set_text(
            &mut systems.renderer,
            &format!(
                "Map [ X: {} Y: {} Group: {} ]",
                database.x, database.y, database.group
            ),
            Attrs::new(),
        );
    }
}

pub fn is_scrollbar_in_hold(gui: &mut Interface) -> bool {
    if gui.tileset_list.scrollbar.in_hold || gui.scrollbar.in_hold {
        return true;
    } else if gui.current_tab == TAB_PROPERTIES && gui.selected_dropbox >= 0{
        if gui.editor_selectionbox[gui.selected_dropbox as usize].scrollbar.in_hold {
            return true;
        }
    }
    false
}