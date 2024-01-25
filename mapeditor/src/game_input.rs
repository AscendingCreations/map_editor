use graphics::*;
use serde::{Deserialize, Serialize};
use winit::dpi::PhysicalSize;
use cosmic_text::{Attrs, Metrics};

use crate::Content;
use crate::interface::*;
use crate::tileset::*;
use crate::map::*;
use crate::resource::*;
use crate::map_data::*;
use crate::collection::{TEXTURE_SIZE, ZOOM_LEVEL};

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
    screen_pos.x >= tileset.map.pos.x &&
        screen_pos.x <= tileset.map.pos.x + (MAX_TILE_X * TEXTURE_SIZE) as f32 &&
        screen_pos.y >= tileset.map.pos.y &&
        screen_pos.y <= tileset.map.pos.y + (MAX_TILE_Y * TEXTURE_SIZE) as f32
}

fn get_tileset_pos(screen_pos: Vec2, tileset: &Tileset) -> Vec2 {
    let tile_pos = screen_pos - Vec2::new(tileset.map.pos.x, tileset.map.pos.y);
    Vec2::new(
        (tile_pos.x / TEXTURE_SIZE as f32).floor(), 
        (tile_pos.y / TEXTURE_SIZE as f32).floor()
    )
}

// Map //
fn in_map(screen_pos: Vec2, mapview: &MapView) -> bool {
    screen_pos.x >= mapview.maps[0].pos.x &&
        screen_pos.x <= mapview.maps[0].pos.x + (32 * TEXTURE_SIZE) as f32 &&
        screen_pos.y >= mapview.maps[0].pos.y &&
        screen_pos.y <= mapview.maps[0].pos.y + (32 * TEXTURE_SIZE) as f32
}

fn get_map_pos(screen_pos: Vec2, mapview: &MapView) -> Vec2 {
    let tile_pos = screen_pos - Vec2::new(mapview.maps[0].pos.x, mapview.maps[0].pos.y);
    Vec2::new(
        (tile_pos.x / TEXTURE_SIZE as f32).floor(), 
        (tile_pos.y / TEXTURE_SIZE as f32).floor()
    )
}

fn interact_with_map(tile_pos: Vec2, 
                    gui: &mut Interface,
                    tileset: &mut Tileset,
                    mapview: &mut MapView,
                    editor_data: &mut EditorData)
{
    match gui.current_setting_tab {
        TAB_LAYER => {
            if gui.current_tool == TOOL_DRAW {
                mapview.set_tile_group(tile_pos, gui.get_tab_option_data(), 
                                &tileset.map, 
                                tileset.select_start, 
                                tileset.select_size);
                editor_data.set_map_change();
            } else if gui.current_tool == TOOL_ERASE {
                mapview.delete_tile_group(tile_pos, gui.get_tab_option_data(),  
                                tileset.select_size);
                editor_data.set_map_change();
            }
        }
        TAB_ATTRIBUTE => {},
        TAB_PROPERTIES => {},
        _ => {},
    }
}

pub fn handle_input(renderer: &mut GpuRenderer,
                    resource: &TextureAllocation,
                    inputtype: InputType, 
                    mouse_pos: &Vec2, 
                    screen_size: &PhysicalSize<f32>,
                    gameinput: &mut GameInput,
                    gui: &mut Interface, 
                    tileset: &mut Tileset,
                    mapview: &mut MapView,
                    editor_data: &mut EditorData
                ) 
{
    // We convert the mouse position to render position as the y pos increase upward
    let screen_pos = Vec2::new(mouse_pos.x / ZOOM_LEVEL, (screen_size.height - mouse_pos.y) / ZOOM_LEVEL);

    match inputtype {
        InputType::MouseLeftDown => {
            if gui.tileset_list.scrollbar.in_scrollbar(screen_pos) {
                gui.tileset_list.scrollbar.hold_scrollbar(screen_pos.y);
            }

            if !gui.tileset_list.scrollbar.in_hold {
                // Check if mouse position is pointing to our tileset
                if in_tileset(screen_pos, tileset) {
                    // Calculate the tile position on the tileset based on mouse position
                    let tile_map_pos = get_tileset_pos(screen_pos, tileset);
                    gameinput.tileset_start = tile_map_pos.clone();
                    gameinput.tileset_end = tile_map_pos.clone();
                    gameinput.return_size = tileset.set_selection(gameinput.tileset_start, gameinput.tileset_end);
                    mapview.change_selection_preview_size(gameinput.return_size);

                    gameinput.presstype = PressType::PressTileset;
                }

                // Check if mouse position is pointing to our map view
                if in_map(screen_pos, mapview) {
                    interact_with_map(get_map_pos(screen_pos, mapview), gui, tileset, mapview, editor_data);
                    gameinput.presstype = PressType::PressMap;
                }

                // Linked Map
                if gameinput.selected_link_map.is_some() {
                    let direction = convert_to_dir(gameinput.selected_link_map.unwrap());
                    let temp_key = editor_data.move_map(direction);
                    if temp_key.is_some() {
                        // We will store a temporary map data when changes happen
                        editor_data.save_map_data(&mapview.maps[0], temp_key);
                    };
                    // Load the initial map
                    editor_data.load_map_data(mapview);
                    editor_data.load_link_maps(mapview);
                }

                // Tools
                let click_button = gui.click_button(screen_pos);
                if click_button.is_some() {
                    let button_index = click_button.unwrap();
                    match button_index {
                        TOOL_LOAD => { println!("To Do!"); },
                        TOOL_SAVE => { 
                            editor_data.save_map_data(&mapview.maps[0], None);
                            println!("Map Saved!");
                        },
                        TOOL_UNDO => { println!("To Do!"); },
                        TOOL_FILL => { println!("To Do!"); },
                        TOOL_EYEDROP => {
                            println!("Current Change {:?}", &editor_data.did_map_change);
                        },
                        TOOL_DRAW | TOOL_ERASE => {
                            gui.set_tool(button_index);
                        },
                        TAB_ATTRIBUTE | TAB_LAYER | TAB_PROPERTIES => {
                            gui.set_tab(button_index);
                        },
                        BUTTON_TILESET => {
                            if gui.tileset_list.visible {
                                gui.tileset_list.hide();
                            } else {
                                gui.tileset_list.show();
                            }
                        },
                        _ => {},
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
                    gui.labels[LABEL_TILESET].set_text(renderer, &resource.tilesheet[tileset_index].name, Attrs::new());
                    tileset.change_tileset(resource, tileset_index);
                    gui.tileset_list.hide();
                }
            }
        },
        InputType::MouseLeftDownMove => {
            if !gui.tileset_list.scrollbar.in_hold {
                // Check if mouse position is pointing to our tileset
                if in_tileset(screen_pos, tileset) && gameinput.presstype == PressType::PressTileset {
                    // Calculate the tile position on the tileset based on mouse position
                    let tile_map_pos = get_tileset_pos(screen_pos, tileset);
                    if gameinput.tileset_end != tile_map_pos { 
                        gameinput.tileset_end = tile_map_pos.clone();
                        gameinput.return_size = tileset.set_selection(gameinput.tileset_start, gameinput.tileset_end);
                        mapview.change_selection_preview_size(gameinput.return_size);
                    }
                }

                // Check if mouse position is pointing to our map view
                if in_map(screen_pos, mapview) && gameinput.presstype == PressType::PressMap {
                    // Calculate the tile position on the map based on mouse position
                    let tile_map_pos = get_map_pos(screen_pos, mapview);
                    
                    interact_with_map(tile_map_pos, gui, tileset, mapview, editor_data);

                    mapview.hover_selection_preview(tile_map_pos);
                }
            } else {
                // Update our tileset list based on the scrollbar value
                gui.tileset_list.scrollbar.move_scrollbar(screen_pos.y);
                if gui.tileset_list.update_scroll(gui.tileset_list.scrollbar.cur_value) {
                    gui.tileset_list.update_list(resource, renderer);
                }
                gui.tileset_list.scrollbar.set_hover(screen_pos);
            }
        },
        InputType::MouseMove => {
            // We check if we can create the effect if the linked map is being hover
            gameinput.selected_link_map = mapview.hover_linked_selection(screen_pos);

            // Calculate the tile position on the map based on mouse position
            if in_map(screen_pos, mapview) {
                let tile_map_pos = get_map_pos(screen_pos, mapview);
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
        },
    }
}