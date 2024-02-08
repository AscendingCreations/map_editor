use std::any::Any;
use cosmic_text::{Attrs, Metrics};
use serde::{Deserialize, Serialize};
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    keyboard::*,
    window::{WindowBuilder, WindowButtons},
};
use graphics::*;

use crate::{
    collection::{TEXTURE_SIZE, ZOOM_LEVEL},
    interface::*,
    map::*,
    map_data::*,
    tileset::*,
    Content,
    DrawSetting,
    interface::preference::keybind::*,
};

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    Quit,
    Select,
}

pub enum InputType {
    MouseLeftDown,
    MouseLeftDownMove,
    MouseMove,
}

#[derive(PartialEq, Eq)]
enum PressType {
    PressNone,
    PressTileset,
    PressMap,
}

pub const ACTION_SIZE: usize = 3;

pub struct GameInput {
    // General
    pub last_mouse_pos: (f32, f32),
    presstype: PressType,
    // Tileset
    pub tileset_start: Vec2,
    pub tileset_end: Vec2,
    pub return_size: Vec2,
    // Map
    selected_link_map: Option<usize>,
    pub dialog_button_press: bool,
    selected_dialog_type: DialogButtonType,
    // Shortcut
    pub hold_key_modifier: [bool; 3],
}

impl GameInput {
    pub fn new() -> Self {
        Self {
            last_mouse_pos: (0.0, 0.0),
            presstype: PressType::PressNone,
            tileset_start: Vec2::new(0.0, 0.0),
            tileset_end: Vec2::new(0.0, 0.0),
            return_size: Vec2::new(1.0, 1.0),
            selected_link_map: None,
            dialog_button_press: false,
            selected_dialog_type: DialogButtonType::ButtonNone,
            hold_key_modifier: [false; 3],
        }
    }
}

pub fn action_index(action: Action) -> usize {
    match action {
        Action::Quit => 0,
        Action::Select => 1,
    }
}

// Tileset //
fn in_tileset(screen_pos: Vec2, tileset: &Tileset) -> bool {
    screen_pos.x >= tileset.map.pos.x
        && screen_pos.x
            <= tileset.map.pos.x + (MAX_TILE_X * TEXTURE_SIZE) as f32
        && screen_pos.y >= tileset.map.pos.y
        && screen_pos.y
            <= tileset.map.pos.y + (MAX_TILE_Y * TEXTURE_SIZE) as f32
}

fn get_tileset_pos(screen_pos: Vec2, tileset: &Tileset) -> Vec2 {
    let tile_pos = screen_pos - Vec2::new(tileset.map.pos.x, tileset.map.pos.y);
    Vec2::new(
        (tile_pos.x / TEXTURE_SIZE as f32).floor(),
        (tile_pos.y / TEXTURE_SIZE as f32).floor(),
    )
}

// Map //
fn in_map(screen_pos: Vec2, mapview: &MapView) -> bool {
    screen_pos.x >= mapview.maps[0].pos.x
        && screen_pos.x <= mapview.maps[0].pos.x + (32 * TEXTURE_SIZE) as f32
        && screen_pos.y >= mapview.maps[0].pos.y
        && screen_pos.y <= mapview.maps[0].pos.y + (32 * TEXTURE_SIZE) as f32
}

fn get_map_pos(screen_pos: Vec2, mapview: &MapView) -> Vec2 {
    let tile_pos =
        screen_pos - Vec2::new(mapview.maps[0].pos.x, mapview.maps[0].pos.y);
    Vec2::new(
        (tile_pos.x / TEXTURE_SIZE as f32).floor(),
        (tile_pos.y / TEXTURE_SIZE as f32).floor(),
    )
}

pub fn update_map_name(
    draw_setting: &mut DrawSetting,
    gui: &mut Interface,
    editor_data: &mut EditorData,
) {
    if editor_data.did_change(editor_data.x, editor_data.y, editor_data.group) {
        gui.labels[LABEL_MAPNAME].set_text(
            &mut draw_setting.renderer,
            &format!(
                "Map [ X: {} Y: {} Group: {} ] Unsaved",
                editor_data.x, editor_data.y, editor_data.group
            ),
            Attrs::new(),
        );
    } else {
        gui.labels[LABEL_MAPNAME].set_text(
            &mut draw_setting.renderer,
            &format!(
                "Map [ X: {} Y: {} Group: {} ]",
                editor_data.x, editor_data.y, editor_data.group
            ),
            Attrs::new(),
        );
    }
}

pub fn handle_dialog_input(
    draw_setting: &mut DrawSetting,
    gameinput: &mut GameInput,
    gui: &mut Interface,
    editor_data: &mut EditorData,
    mapview: &mut MapView,
    elwt: &winit::event_loop::EventLoopWindowTarget<()>,
) {
    if !gameinput.dialog_button_press || gui.dialog.is_none() {
        return;
    }

    gameinput.dialog_button_press = false;
    if let Some(dialog_data) = &mut gui.dialog {
        let dialogtype = dialog_data.dialog_type.clone();

        match gameinput.selected_dialog_type {
            DialogButtonType::ButtonConfirm => match dialogtype {
                DialogType::TypeExitConfirm => elwt.exit(),
                DialogType::TypeMapLoad => {
                    let (mut x, mut y, mut group) = (0 as i32, 0 as i32, 0 as u64);
                    for (index, textbox) in dialog_data.editor_textbox.iter().enumerate() {
                        let value = textbox.data.parse::<i64>().unwrap_or_default();
                        match index {
                            1 => {y = value as i32;}
                            2 => {group = value as u64;}
                            _ => {x = value as i32;}
                        }
                    }
                    editor_data.init_map(x, y, group);
                    editor_data.load_map_data(&mut draw_setting.renderer, mapview);
                    editor_data.load_link_maps(mapview);
                    update_map_name(draw_setting, gui, editor_data);
                    gui.close_dialog();
                }
                DialogType::TypeMapSave => {
                    editor_data.save_all_maps();
                    elwt.exit()
                }
                _ => {}
            },
            DialogButtonType::ButtonDecline => match dialogtype {
                DialogType::TypeMapSave => elwt.exit(),
                _ => {}
            },
            DialogButtonType::ButtonCancel => {
                gui.close_dialog();
            }
            _ => {}
        }
    }
}

pub fn handle_input(
    draw_setting: &mut DrawSetting,
    inputtype: InputType,
    mouse_pos: &Vec2,
    gameinput: &mut GameInput,
    gui: &mut Interface,
    tileset: &mut Tileset,
    mapview: &mut MapView,
    editor_data: &mut EditorData,
) {
    // We convert the mouse position to render position as the y pos increase upward
    let screen_pos = Vec2::new(
        mouse_pos.x / ZOOM_LEVEL,
        (draw_setting.size.height - mouse_pos.y) / ZOOM_LEVEL,
    );

    // If dialog open, cancel all other inputs
    if let Some(dialog) = &mut gui.dialog {
        match inputtype {
            InputType::MouseLeftDown => {
                if dialog.dialog_type == DialogType::TypeMapSave {
                    if dialog.scrollbar.in_scrollbar(screen_pos) {
                        dialog.scrollbar.hold_scrollbar(screen_pos.y);
                    }
                }

                if !dialog.scrollbar.in_hold {
                    gameinput.selected_dialog_type =
                        dialog.click_buttons(screen_pos);
                    gameinput.dialog_button_press = true;
                    dialog.select_text(screen_pos);
                }
            }
            InputType::MouseLeftDownMove => {
                if dialog.dialog_type == DialogType::TypeMapSave {
                    // Update our tileset list based on the scrollbar value
                    dialog.scrollbar.move_scrollbar(screen_pos.y, false);
                    if dialog.update_scroll(dialog.scrollbar.cur_value) {
                        dialog.update_list(&mut draw_setting.renderer);
                    }
                    dialog.scrollbar.set_hover(screen_pos);
                }
            }
            InputType::MouseMove => {
                dialog.hover_buttons(screen_pos);
                // Scrollbar
                dialog.scrollbar.set_hover(screen_pos);
            }
        }
        return;
    }

    // If preference is open, cancel all other inputs
    if gui.preference.is_open {
        match inputtype {
            InputType::MouseLeftDown => {
                if !gui.preference.keywindow.is_open {
                    if gui.preference.scrollbar.in_scrollbar(screen_pos) {
                        gui.preference.scrollbar.hold_scrollbar(screen_pos.y);
                    }

                    if !gui.preference.scrollbar.in_hold {
                        let click_button = gui.preference.click_buttons(screen_pos);
                        if let Some(index) = click_button {
                            match index {
                                0 => {
                                    gui.preference.config_data = match load_config() {
                                        Ok(data) => data,
                                        Err(_) => ConfigData::default(),
                                    };
                                    gui.preference.close()
                                }, // Cancel
                                1 => {
                                    gui.preference.reset_preference(draw_setting);
                                }, // Reset
                                _ => {
                                    gui.preference.config_data.save_config().unwrap();
                                    // Apply settings
                                    mapview.selection_preview.set_color(Color::rgba(
                                        gui.preference.config_data.map_selection_color[0],
                                        gui.preference.config_data.map_selection_color[1],
                                        gui.preference.config_data.map_selection_color[2],
                                        150,
                                    ));
                                    tileset.selection.set_color(Color::rgba(
                                        gui.preference.config_data.tile_selection_color[0],
                                        gui.preference.config_data.tile_selection_color[1],
                                        gui.preference.config_data.tile_selection_color[2],
                                        150,
                                    ));
                                    gui.preference.close()
                                }, // Save
                            }
                        }

                        if gui.preference.select_menu_button(screen_pos) {
                            open_preference_tab(&mut gui.preference, draw_setting);
                        }

                        match gui.preference.selected_menu {
                            PREF_TAB_GENERAL => {
                                if gui.preference.in_color_selection(screen_pos) {
                                    gui.preference.select_text(screen_pos);
                                    if gui.preference.click_color_selection_button(screen_pos) {
                                        if let Some(index) = gui.preference.is_coloreditor_open {
                                            if let SettingData::ColorSelection(colorselection) = &mut gui.preference.setting_data[index] {
                                                let data = colorselection.color_editor.data.clone();
                                                colorselection.image.set_color(Color::rgba(data[0], data[1], data[2], data[3]));
                                                match index {
                                                    1 => gui.preference.config_data.map_selection_color = data,
                                                    2 => gui.preference.config_data.tile_selection_color = data,
                                                    _ => {}
                                                }
                                            }
                                        }
                                        gui.preference.hide_color_selection();
                                    }
                                } else {
                                    if let Some(config_index) = gui.preference.select_config(screen_pos) {
                                        match &mut gui.preference.setting_data[config_index] {
                                            SettingData::Checkbox(checkbox) => {
                                                if checkbox.is_select {
                                                    checkbox.set_select(false);
                                                } else {
                                                    checkbox.set_select(true);
                                                }
                                                // Hide color selection if it is visible
                                                gui.preference.hide_color_selection();
                                            },
                                            SettingData::ColorSelection(colorselection) => {
                                                if gui.preference.is_coloreditor_open.is_none() {
                                                    colorselection.open_color_editor();
                                                    gui.preference.is_coloreditor_open = Some(config_index);
                                                } else {
                                                    // Hide color selection if it is visible
                                                    gui.preference.hide_color_selection();
                                                }
                                            },
                                            _ => {},
                                        }
                                    } else {
                                        gui.preference.hide_color_selection();
                                    }
                                }
                            },
                            PREF_TAB_KEYBIND => {
                                if let Some(key_index) = gui.preference.select_keylist(screen_pos) {
                                    gui.preference.keywindow.open_key(draw_setting, key_index);
                                }
                            },
                            _ => {},
                        }
                    }
                } else {
                    let click_button = gui.preference.keywindow.click_buttons(screen_pos);
                    if let Some(index) = click_button {
                        match index {
                            0 => gui.preference.keywindow.close_key(), // Cancel
                            _ => {
                                if let Some(keycode) = &gui.preference.keywindow.key_code {
                                    let index = gui.preference.keywindow.key_index;
                                    gui.preference.config_data.key_code[index] = keycode.clone();
                                    gui.preference.config_data.key_code_modifier[index] = gui.preference.keywindow.key_modifier.clone();
                                    gui.preference.update_key_list(draw_setting, index);
                                }
                                gui.preference.keywindow.close_key()
                            }, // Save
                        }
                    }
                }
            }
            InputType::MouseLeftDownMove => {
                gui.preference.scrollbar.move_scrollbar(screen_pos.y, false);
                if gui.preference.update_scroll(gui.preference.scrollbar.cur_value) {
                    gui.preference.update_list();
                }
                gui.preference.scrollbar.set_hover(screen_pos);
            }
            InputType::MouseMove => {
                gui.preference.hover_buttons(screen_pos);
                gui.preference.scrollbar.set_hover(screen_pos);
                if gui.preference.keywindow.is_open {
                    gui.preference.keywindow.hover_buttons(screen_pos);
                }
            }
        }
        return;
    }

    match inputtype {
        InputType::MouseLeftDown => {
            if gui.tileset_list.scrollbar.in_scrollbar(screen_pos) {
                gui.tileset_list.scrollbar.hold_scrollbar(screen_pos.y);
            } else if gui.scrollbar.in_scrollbar(screen_pos) {
                gui.scrollbar.hold_scrollbar(screen_pos.y);
            } else if gui.current_setting_tab == TAB_PROPERTIES && gui.selected_dropbox >= 0 {
                if gui.editor_selectionbox[gui.selected_dropbox as usize].scrollbar.in_scrollbar(screen_pos) {
                    gui.editor_selectionbox[gui.selected_dropbox as usize].scrollbar.hold_scrollbar(screen_pos.y);
                }
            }

            if !is_scrollbar_in_hold(gui) {
                // Check if mouse position is pointing to our tileset
                if in_tileset(screen_pos, tileset) && gui.current_setting_tab == TAB_LAYER {
                    // Calculate the tile position on the tileset based on mouse position
                    let tile_map_pos = get_tileset_pos(screen_pos, tileset)
                        .min(Vec2::new(
                            (MAX_TILE_X - 1) as f32,
                            (MAX_TILE_Y - 1) as f32,
                        ));
                    gameinput.tileset_start = tile_map_pos.clone();
                    gameinput.tileset_end = tile_map_pos.clone();
                    gameinput.return_size = tileset.set_selection(
                        gameinput.tileset_start,
                        gameinput.tileset_end,
                    );
                    mapview
                        .change_selection_preview_size(gameinput.return_size);
                    gameinput.presstype = PressType::PressTileset;
                }

                // Check if mouse position is pointing to our map view
                if in_map(screen_pos, mapview) {
                    mapview.record.set_undo_record();
                    interact_with_map(
                        draw_setting,
                        get_map_pos(screen_pos, mapview),
                        gui,
                        tileset,
                        mapview,
                        editor_data,
                        gameinput,
                    );
                    gameinput.presstype = PressType::PressMap;
                }

                // Linked Map
                if gameinput.selected_link_map.is_some() {
                    let direction =
                        convert_to_dir(gameinput.selected_link_map.unwrap());
                    let temp_key = editor_data.move_map(direction);
                    if temp_key.is_some() {
                        // We will store a temporary map data when changes happen
                        editor_data.save_map_data(&mapview, temp_key);
                    };
                    // Load the initial map
                    editor_data.load_map_data(&mut draw_setting.renderer, mapview);
                    editor_data.load_link_maps(mapview);
                    update_map_name(draw_setting, gui, editor_data);

                    match gui.current_setting_tab {
                        TAB_ZONE => {
                            mapview.update_map_zone(gui.current_tab_data as usize);
                        },
                        TAB_PROPERTIES => {
                            gui.editor_selectionbox[0].switch_list(&mut draw_setting.renderer, mapview.fixed_weather as usize);
                        },
                        _ => {},
                    }
                }

                // Tools
                let click_button = gui.click_tool_button(screen_pos);
                if let Some(button_index) = click_button {
                    match button_index {
                        TOOL_LOAD => {
                            if gui.preference.is_open {
                                if gui.preference.keywindow.is_open {
                                    gui.preference.keywindow.close_key();
                                }
                                gui.preference.config_data = match load_config() {
                                    Ok(data) => data,
                                    Err(_) => ConfigData::default(),
                                };
                                gui.preference.close();
                            }
                            gui.open_dialog(
                                draw_setting,
                                DialogType::TypeMapLoad,
                                None,
                            );
                        }
                        TOOL_SAVE => {
                            editor_data.save_map_data(&mapview, None);
                            update_map_name(draw_setting, gui, editor_data);
                        }
                        TOOL_UNDO => {
                            mapview.apply_change(&mut draw_setting.renderer, true);
                        }
                        TOOL_REDO => {
                            mapview.apply_change(&mut draw_setting.renderer, false);
                        }
                        TOOL_DRAW | TOOL_ERASE | TOOL_FILL | TOOL_EYEDROP => {
                            gui.set_tool(button_index);
                        }
                        TAB_ATTRIBUTE | TAB_LAYER | TAB_PROPERTIES | TAB_ZONE => {
                            gui.set_tab(draw_setting, button_index, mapview, tileset, gameinput);
                            if gui.tileset_list.visible {
                                gui.tileset_list.hide();
                            }
                        }
                        BUTTON_TILESET => {
                            if gui.current_setting_tab == TAB_LAYER {
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

                // Tab Options
                let click_tab_option = gui.click_tab_option(screen_pos);
                if click_tab_option.is_some() {
                    gui.select_tab_option(click_tab_option.unwrap());
                    // Open
                    match gui.current_setting_tab {
                        TAB_ATTRIBUTE => gui.open_attribute_settings(draw_setting, gui.current_tab_data + 1, vec![]),
                        TAB_ZONE => {
                            mapview.update_map_zone(gui.current_tab_data as usize);
                            gui.open_zone_settings(draw_setting, mapview);
                        },
                        _ => {},
                    }
                }
                
                // Textbox / Buttons
                match gui.current_setting_tab {
                    TAB_ATTRIBUTE | TAB_ZONE => gui.select_textbox(screen_pos),
                    TAB_PROPERTIES => {
                        // Buttons
                        let click_button = gui.click_buttons(screen_pos);
                        if let Some(button_index) = click_button {
                            match button_index {
                                0 => {
                                    println!("Save All");
                                },
                                1 => {
                                    println!("Reset All");
                                },
                                2 => {
                                    gui.preference.open();
                                    open_preference_tab(&mut gui.preference, draw_setting);
                                },
                                _ => {},
                            }
                        }

                        // Selection box
                        let click_button = gui.click_selectionbox(screen_pos);
                        if let Some(selection_index) = click_button {
                            match selection_index {
                                0 => {
                                    if !gui.editor_selectionbox[selection_index].is_list_visible {
                                        gui.editor_selectionbox[selection_index].show_list(&mut draw_setting.renderer);
                                        gui.selected_dropbox = selection_index as i32;
                                    } else {
                                        gui.editor_selectionbox[selection_index].hide_list();
                                        gui.selected_dropbox = -1;
                                    }
                                }, // Weather
                                _ => {},
                            }
                        }

                        // Dropdown List
                        if gui.selected_dropbox >= 0 {
                            let click_button = gui.editor_selectionbox[gui.selected_dropbox as usize].click_list(screen_pos);
                            if let Some(selection_index) = click_button {
                                gui.editor_selectionbox[gui.selected_dropbox as usize].switch_list(&mut draw_setting.renderer, selection_index);

                                match gui.selected_dropbox {
                                    0 => {
                                        mapview.fixed_weather = gui.editor_selectionbox[gui.selected_dropbox as usize].selected_index as u8;
                                        if editor_data.set_map_change() {
                                            update_map_name(draw_setting, gui, editor_data);
                                        };
                                        mapview.record.clear_redo();
                                    }
                                    _ => {},
                                }

                                gui.editor_selectionbox[gui.selected_dropbox as usize].hide_list();
                            }
                        }
                    },
                    _ => {},
                }

                // Tileset List
                if gui.tileset_list.select_list(screen_pos) {
                    // This will process the switching of tileset
                    let tileset_index = gui.tileset_list.selected_tileset;
                    gui.labels[LABEL_TILESET].set_text(
                        &mut draw_setting.renderer,
                        &draw_setting.resource.tilesheet[tileset_index].name,
                        Attrs::new(),
                    );
                    tileset.change_tileset(&mut draw_setting.resource, tileset_index);
                    gui.tileset_list.hide();
                }
            }
        }
        InputType::MouseLeftDownMove => {
            if !is_scrollbar_in_hold(gui) {
                // Check if mouse position is pointing to our tileset
                if in_tileset(screen_pos, tileset)
                    && gameinput.presstype == PressType::PressTileset
                {
                    // Calculate the tile position on the tileset based on mouse position
                    let tile_map_pos = get_tileset_pos(screen_pos, tileset)
                        .min(Vec2::new(
                            (MAX_TILE_X - 1) as f32,
                            (MAX_TILE_Y - 1) as f32,
                        ));
                    if gameinput.tileset_end != tile_map_pos {
                        gameinput.tileset_end = tile_map_pos;
                        gameinput.return_size = tileset.set_selection(
                            gameinput.tileset_start,
                            gameinput.tileset_end,
                        );
                        mapview.change_selection_preview_size(
                            gameinput.return_size,
                        );
                    }
                }

                // Check if mouse position is pointing to our map view
                if in_map(screen_pos, mapview)
                    && gameinput.presstype == PressType::PressMap
                {
                    // Calculate the tile position on the map based on mouse position
                    let tile_map_pos = get_map_pos(screen_pos, mapview);

                    gui.labels[LABEL_TILEPOS].set_text(&mut draw_setting.renderer,&format!("Tile [ X: {} Y: {} ]", tile_map_pos.x, tile_map_pos.y), Attrs::new());

                    interact_with_map(
                        draw_setting,
                        tile_map_pos,
                        gui,
                        tileset,
                        mapview,
                        editor_data,
                        gameinput,
                    );

                    mapview.hover_selection_preview(tile_map_pos);
                }
            } else if gui.tileset_list.scrollbar.in_hold {
                // Update our tileset list based on the scrollbar value
                gui.tileset_list.scrollbar.move_scrollbar(screen_pos.y, false);
                if gui
                    .tileset_list
                    .update_scroll(gui.tileset_list.scrollbar.cur_value)
                {
                    gui.tileset_list.update_list(&mut draw_setting.renderer, &draw_setting.resource);
                }
                gui.tileset_list.scrollbar.set_hover(screen_pos);
            } else if gui.scrollbar.in_hold {
                gui.scrollbar.move_scrollbar(screen_pos.y, false);
                gui.update_scroll(&mut draw_setting.renderer, gui.scrollbar.cur_value);
                gui.scrollbar.set_hover(screen_pos);
            } else if gui.current_setting_tab == TAB_PROPERTIES && gui.selected_dropbox >= 0 {
                if gui.editor_selectionbox[gui.selected_dropbox as usize].scrollbar.in_hold {
                    gui.editor_selectionbox[gui.selected_dropbox as usize].scrollbar.move_scrollbar(screen_pos.y, false);
                    let scrollbar_value = gui.editor_selectionbox[gui.selected_dropbox as usize].scrollbar.cur_value;
                    gui.editor_selectionbox[gui.selected_dropbox as usize].update_list(&mut draw_setting.renderer, scrollbar_value);
                    gui.editor_selectionbox[gui.selected_dropbox as usize].scrollbar.set_hover(screen_pos);
                }
            }
        }
        InputType::MouseMove => {
            // We check if we can create the effect if the linked map is being hover
            gameinput.selected_link_map =
                mapview.hover_linked_selection(screen_pos);

            // Calculate the tile position on the map based on mouse position
            if in_map(screen_pos, mapview) {
                let tile_map_pos = get_map_pos(screen_pos, mapview);
                gui.labels[LABEL_TILEPOS].set_text(&mut draw_setting.renderer,&format!("Tile [ X: {} Y: {} ]",tile_map_pos.x, tile_map_pos.y),Attrs::new());
                mapview.hover_selection_preview(tile_map_pos);
            }

            // Buttons
            gui.hover_tool_button(screen_pos);
            gui.hover_buttons(screen_pos);
            gui.hover_selectionbox(screen_pos);
            // Tab Options
            gui.hover_tab_option(screen_pos);
            // Tileset List Selection
            gui.tileset_list.hover_selection(screen_pos);
            // Scrollbar
            gui.tileset_list.scrollbar.set_hover(screen_pos);
            gui.scrollbar.set_hover(screen_pos);
            if gui.current_setting_tab == TAB_PROPERTIES && gui.selected_dropbox >= 0 {
                gui.editor_selectionbox[gui.selected_dropbox as usize].hover_list(screen_pos);
                gui.editor_selectionbox[gui.selected_dropbox as usize].scrollbar.set_hover(screen_pos);
            }
        }
    }
}

pub fn handle_key_input(
    renderer: &mut GpuRenderer,
    event: &KeyEvent,
    gui: &mut Interface,
    mapview: &mut MapView,
) -> bool {
    if gui.preference.is_open {
        match gui.preference.selected_menu {
            PREF_TAB_KEYBIND => {
                if gui.preference.keywindow.is_open {
                    gui.preference.keywindow.edit_key(event, renderer);
                }
            },
            PREF_TAB_GENERAL => {
                if event.state.is_pressed() {
                    if let Some(index) = gui.preference.is_coloreditor_open {
                        if let SettingData::ColorSelection(colorselection) = &mut gui.preference.setting_data[index] {
                            if colorselection.color_editor.is_open {
                                colorselection.color_editor.textbox[gui.preference.editing_index]
                                    .enter_numeric(renderer, event, 3, false);

                                let value = colorselection.color_editor.textbox[gui.preference.editing_index as usize].data.parse::<i64>().unwrap_or_default();
                                colorselection.color_editor.data[gui.preference.editing_index] = (value as u8).min(255);
                            }
                        }
                    }
                }
            },
            _ => {},
        }
        return true;
    }

    if !event.state.is_pressed() {
        return false;
    }

    let mut result = false;

    if let Some(dialog) = &mut gui.dialog {
        if dialog.dialog_type == DialogType::TypeMapLoad {
            if dialog.editing_index < 2 {
                dialog.editor_textbox[dialog.editing_index].enter_numeric(renderer, event, 5, true);
            } else {
                dialog.editor_textbox[dialog.editing_index].enter_numeric(renderer, event, 5, false);
            }
            result = true;
        }
    } else {
        match gui.current_setting_tab {
            TAB_ATTRIBUTE => {
                let attribute = MapAttribute::convert_to_plain_enum(gui.current_tab_data + 1);
                match attribute {
                    MapAttribute::Warp(_, _, _, _, _) => {
                        if gui.selected_textbox >= 0 {
                            if gui.selected_textbox < 2 {
                                gui.editor_textbox[gui.selected_textbox as usize].enter_numeric(renderer, event, 5, true);
                            } else {
                                gui.editor_textbox[gui.selected_textbox as usize].enter_numeric(renderer, event, 5, false);
                            }
                            result = true;
                        }
                    },
                    MapAttribute::Sign(_) => {
                        if gui.selected_textbox >= 0 {
                            gui.editor_textbox[gui.selected_textbox as usize].enter_text(renderer, event, 100);
                            result = true;
                        }
                    },
                    _ => {},
                }
            },
            TAB_ZONE => {
                if gui.selected_textbox >= 0 {
                    gui.editor_textbox[gui.selected_textbox as usize].enter_numeric(renderer, event, 5, false);
                    match gui.selected_textbox {
                        0 => {
                            let value = gui.editor_textbox[gui.selected_textbox as usize].data.parse::<i64>().unwrap_or_default();
                            mapview.map_zone_setting[gui.current_tab_data as usize]
                                    .max_npc = value as u64
                        }, // Max NPC
                        _ => {
                            if gui.editor_textbox[gui.selected_textbox as usize].data.len() > 0 {
                                let value = gui.editor_textbox[gui.selected_textbox as usize].data.parse::<i64>().unwrap_or_default();
                                mapview.map_zone_setting[gui.current_tab_data as usize]
                                        .npc_id[(gui.selected_textbox - 1) as usize] = Some(value as u64);
                            } else {
                                mapview.map_zone_setting[gui.current_tab_data as usize]
                                        .npc_id[(gui.selected_textbox - 1) as usize] = None;
                            }
                        }, // Npc ID
                    }
                    result = true;
                }
            },
            _ => {},
        }
    }
    result
}

pub fn is_scrollbar_in_hold(gui: &mut Interface) -> bool {
    if gui.tileset_list.scrollbar.in_hold {
        return true;
    } else if gui.scrollbar.in_hold {
        return true;
    } else if gui.current_setting_tab == TAB_PROPERTIES && gui.selected_dropbox >= 0{
        if gui.editor_selectionbox[gui.selected_dropbox as usize].scrollbar.in_hold {
            return true;
        }
    }
    false
}

pub fn set_key_modifier_value(game_input: &mut GameInput, modifier_index: usize, is_pressed: bool) {
    if game_input.hold_key_modifier[modifier_index] == is_pressed {
        return;
    }
    game_input.hold_key_modifier[modifier_index] = is_pressed;
}

pub fn handle_shortcut(event: &KeyEvent,
                        draw_setting: &mut DrawSetting,
                        gameinput: &mut GameInput,
                        editor_data: &mut EditorData,
                        mapview: &mut MapView,
                        gui: &mut Interface,) {
    let mut got_key = None;
    let mut key_modifier = [false; 3];

    if gui.dialog.is_some() || gui.preference.is_open {
        return;
    }
    
    // Read Input
    match event.physical_key {
        PhysicalKey::Code(KeyCode::ControlLeft) | PhysicalKey::Code(KeyCode::ControlRight) => 
            set_key_modifier_value(gameinput, 0, event.state.is_pressed()),
        PhysicalKey::Code(KeyCode::ShiftLeft) | PhysicalKey::Code(KeyCode::ShiftRight) => 
            set_key_modifier_value(gameinput, 1, event.state.is_pressed()),
        PhysicalKey::Code(KeyCode::Space) => 
            set_key_modifier_value(gameinput, 2, event.state.is_pressed()),
        _ => {
            if is_valid_key_code(event) && event.state.is_pressed() {
                got_key = Some(event.logical_key.clone());
                key_modifier = gameinput.hold_key_modifier.clone();
                gameinput.hold_key_modifier = [false; 3];
            }
        },
    }

    // Handle Input Logic
    if let Some(keycode) = got_key {
        if let Some(got_index) = (0..EditorKey::Count as usize).find(|&index| {
            gui.preference.config_data.key_code[index] == keycode &&
            gui.preference.config_data.key_code_modifier[index] == key_modifier
        }) {
            match got_index {
                1 => {
                    editor_data.save_map_data(&mapview, None);
                    update_map_name(draw_setting, gui, editor_data);
                }, // Save
                2 => mapview.apply_change(&mut draw_setting.renderer, true), // Undo
                3 => mapview.apply_change(&mut draw_setting.renderer, false), // Redo
                4 => gui.set_tool(TOOL_DRAW), // Draw
                5 => gui.set_tool(TOOL_ERASE), // Erase
                6 => gui.set_tool(TOOL_FILL), // Fill
                7 => gui.set_tool(TOOL_EYEDROP), // Eyetool
                _ => {
                    if gui.preference.is_open {
                        if gui.preference.keywindow.is_open {
                            gui.preference.keywindow.close_key();
                        }
                        gui.preference.config_data = match load_config() {
                            Ok(data) => data,
                            Err(_) => ConfigData::default(),
                        };
                        gui.preference.close();
                    }
                    gui.open_dialog(
                        draw_setting,
                        DialogType::TypeMapLoad,
                        None,
                    );
                }, // Load
            }
        }
    }
}

fn is_valid_key_code(event: &KeyEvent) -> bool {
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
            | KeyCode::Backslash | KeyCode::Semicolon | KeyCode::Slash
        ) => true,
        _ => false,
    }
}