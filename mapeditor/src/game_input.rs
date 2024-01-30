use std::any::Any;

use cosmic_text::{Attrs, Metrics};
use graphics::*;
use serde::{Deserialize, Serialize};

use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    keyboard::*,
    window::{WindowBuilder, WindowButtons},
};

mod textbox;

use textbox::*;

use crate::collection::{TEXTURE_SIZE, ZOOM_LEVEL};
use crate::interface::dialog::DialogButtonType;
use crate::interface::*;
use crate::map::*;
use crate::map_data::*;
use crate::resource::*;
use crate::tileset::*;
use crate::Content;

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
    tileset_start: Vec2,
    tileset_end: Vec2,
    return_size: Vec2,
    // Map
    selected_link_map: Option<usize>,
    pub dialog_button_press: bool,
    selected_dialog_type: DialogButtonType,
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

fn interact_with_map(
    renderer: &mut GpuRenderer,
    resource: &TextureAllocation,
    tile_pos: Vec2,
    gui: &mut Interface,
    tileset: &mut Tileset,
    mapview: &mut MapView,
    editor_data: &mut EditorData,
    gameinput: &mut GameInput,
) {
    match gui.current_setting_tab {
        TAB_LAYER => {
            match gui.current_tool {
                TOOL_DRAW => {
                    mapview.set_tile_group(
                        tile_pos,
                        gui.get_tab_option_data(),
                        &tileset.map,
                        tileset.select_start,
                        tileset.select_size,
                    );
                    if editor_data.set_map_change() {
                        update_map_name(renderer, gui, editor_data);
                    };
                    mapview.record.clear_redo();
                }
                TOOL_ERASE => {
                    mapview.delete_tile_group(
                        tile_pos,
                        gui.get_tab_option_data(),
                        tileset.select_size,
                    );
                    if editor_data.set_map_change() {
                        update_map_name(renderer, gui, editor_data);
                    };
                    mapview.record.clear_redo();
                }
                TOOL_FILL => {
                    mapview.set_tile_fill(
                        tile_pos,
                        gui.get_tab_option_data(),
                        &tileset.map,
                        tileset.select_start,
                    );
                    if editor_data.set_map_change() {
                        update_map_name(renderer, gui, editor_data);
                    };
                    mapview.record.clear_redo();
                }
                TOOL_EYEDROP => {
                    let tiledata = mapview.get_tile_data(tile_pos);
                    let id = tiledata.id;
                    if let Some((x, y, tile)) = resource.tile_location.get(&id)
                    {
                        // Change the loaded tileset
                        gui.tileset_list.selected_tileset =
                            tile.clone() as usize;
                        gui.labels[LABEL_TILESET].set_text(
                            renderer,
                            &resource.tilesheet
                                [gui.tileset_list.selected_tileset]
                                .name,
                            Attrs::new(),
                        );
                        tileset.change_tileset(
                            resource,
                            gui.tileset_list.selected_tileset,
                        );
                        gui.tileset_list.update_list(resource, renderer);

                        // Set the selected tile position
                        let (posx, posy) = (
                            x / TEXTURE_SIZE,
                            (MAX_TILE_Y - (y / TEXTURE_SIZE) - 1),
                        );
                        gameinput.tileset_start =
                            Vec2::new(posx as f32, posy as f32);
                        gameinput.tileset_end =
                            Vec2::new(posx as f32, posy as f32);
                        gameinput.return_size = tileset.set_selection(
                            gameinput.tileset_start,
                            gameinput.tileset_end,
                        );
                        mapview.change_selection_preview_size(
                            gameinput.return_size,
                        );
                    }
                }
                _ => {}
            }
        }
        TAB_ATTRIBUTE => {}
        TAB_PROPERTIES => {}
        _ => {}
    }
}

pub fn update_map_name(
    renderer: &mut GpuRenderer,
    gui: &mut Interface,
    editor_data: &mut EditorData,
) {
    if editor_data.did_change(editor_data.x, editor_data.y, editor_data.group) {
        gui.labels[LABEL_MAPNAME].set_text(
            renderer,
            &format!(
                "Map [ X: {} Y: {} Group: {} ] Unsaved",
                editor_data.x, editor_data.y, editor_data.group
            ),
            Attrs::new(),
        );
    } else {
        gui.labels[LABEL_MAPNAME].set_text(
            renderer,
            &format!(
                "Map [ X: {} Y: {} Group: {} ]",
                editor_data.x, editor_data.y, editor_data.group
            ),
            Attrs::new(),
        );
    }
}

pub fn handle_dialog_input(
    renderer: &mut GpuRenderer,
    gameinput: &mut GameInput,
    gui: &mut Interface,
    elwt: &winit::event_loop::EventLoopWindowTarget<()>,
    editor_data: &mut EditorData,
    mapview: &mut MapView,
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
                    let (mut x, mut y, mut group) =
                        (0 as i32, 0 as i32, 0 as u64);
                    for (index, data) in
                        dialog_data.editor_data.iter().enumerate()
                    {
                        let value = data.parse::<i64>().unwrap_or_default();
                        match index {
                            1 => {
                                y = value as i32;
                            }
                            2 => {
                                group = value as u64;
                            }
                            _ => {
                                x = value as i32;
                            }
                        }
                    }
                    editor_data.init_map(x, y, group);
                    editor_data.load_map_data(mapview);
                    editor_data.load_link_maps(mapview);
                    update_map_name(renderer, gui, editor_data);
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
    renderer: &mut GpuRenderer,
    resource: &TextureAllocation,
    inputtype: InputType,
    mouse_pos: &Vec2,
    screen_size: &PhysicalSize<f32>,
    scale: f64,
    gameinput: &mut GameInput,
    gui: &mut Interface,
    tileset: &mut Tileset,
    mapview: &mut MapView,
    editor_data: &mut EditorData,
) {
    // We convert the mouse position to render position as the y pos increase upward
    let screen_pos = Vec2::new(
        mouse_pos.x / ZOOM_LEVEL,
        (screen_size.height - mouse_pos.y) / ZOOM_LEVEL,
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
                    dialog.scrollbar.move_scrollbar(screen_pos.y);
                    if dialog.update_scroll(dialog.scrollbar.cur_value) {
                        dialog.update_list(renderer);
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

    match inputtype {
        InputType::MouseLeftDown => {
            if gui.tileset_list.scrollbar.in_scrollbar(screen_pos) {
                gui.tileset_list.scrollbar.hold_scrollbar(screen_pos.y);
            }

            if !gui.tileset_list.scrollbar.in_hold {
                // Check if mouse position is pointing to our tileset
                if in_tileset(screen_pos, tileset) {
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
                        renderer,
                        resource,
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
                        editor_data.save_map_data(&mapview.maps[0], temp_key);
                    };
                    // Load the initial map
                    editor_data.load_map_data(mapview);
                    editor_data.load_link_maps(mapview);
                    update_map_name(renderer, gui, editor_data);
                }

                // Tools
                let click_button = gui.click_button(screen_pos);
                if click_button.is_some() {
                    let button_index = click_button.unwrap();
                    match button_index {
                        TOOL_LOAD => {
                            gui.open_dialog(
                                resource,
                                renderer,
                                screen_size,
                                scale,
                                DialogType::TypeMapLoad,
                                None,
                            );
                        }
                        TOOL_SAVE => {
                            editor_data.save_map_data(&mapview.maps[0], None);
                            update_map_name(renderer, gui, editor_data);
                        }
                        TOOL_UNDO => {
                            mapview.apply_undo();
                        }
                        TOOL_REDO => {
                            mapview.apply_redo();
                        }
                        TOOL_DRAW | TOOL_ERASE | TOOL_FILL | TOOL_EYEDROP => {
                            gui.set_tool(button_index);
                        }
                        TAB_ATTRIBUTE | TAB_LAYER | TAB_PROPERTIES => {
                            gui.set_tab(button_index);
                        }
                        BUTTON_TILESET => {
                            if gui.tileset_list.visible {
                                gui.tileset_list.hide();
                            } else {
                                gui.tileset_list.show();
                            }
                        }
                        _ => {}
                    }
                }

                // Tab Options
                let click_tab_option = gui.click_tab_option(screen_pos);
                if click_tab_option.is_some() {
                    gui.select_tab_option(click_tab_option.unwrap());
                }

                // Tileset List
                if gui.tileset_list.select_list(screen_pos) {
                    // This will process the switching of tileset
                    let tileset_index = gui.tileset_list.selected_tileset;
                    gui.labels[LABEL_TILESET].set_text(
                        renderer,
                        &resource.tilesheet[tileset_index].name,
                        Attrs::new(),
                    );
                    tileset.change_tileset(resource, tileset_index);
                    gui.tileset_list.hide();
                }
            }
        }
        InputType::MouseLeftDownMove => {
            if !gui.tileset_list.scrollbar.in_hold {
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

                    gui.labels[LABEL_TILEPOS].set_text(
                        renderer,
                        &format!(
                            "Tile [ X: {} Y: {} ]",
                            tile_map_pos.x, tile_map_pos.y
                        ),
                        Attrs::new(),
                    );

                    interact_with_map(
                        renderer,
                        resource,
                        tile_map_pos,
                        gui,
                        tileset,
                        mapview,
                        editor_data,
                        gameinput,
                    );

                    mapview.hover_selection_preview(tile_map_pos);
                }
            } else {
                // Update our tileset list based on the scrollbar value
                gui.tileset_list.scrollbar.move_scrollbar(screen_pos.y);
                if gui
                    .tileset_list
                    .update_scroll(gui.tileset_list.scrollbar.cur_value)
                {
                    gui.tileset_list.update_list(resource, renderer);
                }
                gui.tileset_list.scrollbar.set_hover(screen_pos);
            }
        }
        InputType::MouseMove => {
            // We check if we can create the effect if the linked map is being hover
            gameinput.selected_link_map =
                mapview.hover_linked_selection(screen_pos);

            // Calculate the tile position on the map based on mouse position
            if in_map(screen_pos, mapview) {
                let tile_map_pos = get_map_pos(screen_pos, mapview);
                gui.labels[LABEL_TILEPOS].set_text(
                    renderer,
                    &format!(
                        "Tile [ X: {} Y: {} ]",
                        tile_map_pos.x, tile_map_pos.y
                    ),
                    Attrs::new(),
                );
                mapview.hover_selection_preview(tile_map_pos);
            }

            // Buttons
            gui.hover_button(screen_pos);
            // Tab Options
            gui.hover_tab_option(screen_pos);
            // Tileset List Selection
            gui.tileset_list.hover_selection(screen_pos);
            // Scrollbar
            gui.tileset_list.scrollbar.set_hover(screen_pos);
        }
    }
}

pub fn handle_key_input(
    renderer: &mut GpuRenderer,
    event: &KeyEvent,
    gui: &mut Interface,
) {
    if !event.state.is_pressed() {
        return;
    }

    if let Some(dialog) = &mut gui.dialog {
        if dialog.dialog_type == DialogType::TypeMapLoad {
            if dialog.editing_index < 2 {
                enter_numeric(
                    &mut dialog.editor_data[dialog.editing_index],
                    event,
                    5,
                    true,
                );
            } else {
                enter_numeric(
                    &mut dialog.editor_data[dialog.editing_index],
                    event,
                    5,
                    false,
                );
            }
            dialog.update_editor_data(renderer);
        }
    }
}
