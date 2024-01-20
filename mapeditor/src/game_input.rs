use graphics::*;
use serde::{Deserialize, Serialize};
use winit::dpi::PhysicalSize;

use crate::Content;
use crate::interface::*;
use crate::tileset::*;
use crate::map::*;
use crate::collection::TEXTURE_SIZE;

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
}

impl GameInput {
    pub fn new() -> Self {
        Self {
            last_mouse_pos: (0.0, 0.0),
            presstype: PressType::PressNone,
            tileset_start: Vec2::new(0.0, 0.0),
            tileset_end: Vec2::new(0.0, 0.0),
            return_size: Vec2::new(1.0, 1.0),
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

pub fn handle_input(inputtype: InputType, 
                    mouse_pos: &Vec2, 
                    screen_size: &PhysicalSize<f32>,
                    gameinput: &mut GameInput,
                    _gui: &Interface, 
                    tileset: &mut Tileset,
                    mapview: &mut MapView,) 
{
    // We convert the mouse position to render position as the y pos increase upward
    let screen_pos = Vec2::new(mouse_pos.x, screen_size.height - mouse_pos.y);

    match inputtype {
        InputType::MouseLeftDown => {
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
                // Calculate the tile position on the map based on mouse position
                let tile_map_pos = get_map_pos(screen_pos, mapview);
                mapview.set_tile_group(tile_map_pos, 0, 
                                &tileset.map, 
                                tileset.select_start, 
                                tileset.select_size);
                
                gameinput.presstype = PressType::PressMap;
            }
        },
        InputType::MouseLeftDownMove => {
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
                mapview.set_tile_group(tile_map_pos, 0, 
                                &tileset.map, 
                                tileset.select_start, 
                                tileset.select_size);
                mapview.hover_selection_preview(tile_map_pos);
            }
        },
        InputType::MouseMove => {
            // We check if we can create the effect if the linked map is being hover
            mapview.hover_linked_selection(screen_pos);

            // Calculate the tile position on the map based on mouse position
            if in_map(screen_pos, mapview) {
                let tile_map_pos = get_map_pos(screen_pos, mapview);
                mapview.hover_selection_preview(tile_map_pos);
            }
        },
    }
}