pub mod map_input;
pub mod attributes;
mod recording;

use indexmap::IndexMap;
use cosmic_text::{Attrs, Metrics, Weight};
use graphics::*;

pub use map_input::*;
pub use attributes::*;
use recording::*;

use crate::{
    DrawSetting,
    collection::TEXTURE_SIZE,
    gfx_order::*,
    map_data::*,
};


pub struct MapAttributes {
    pub pos: Vec2,
    pub image: Rect,
    pub text: Text,
    pub attribute: MapAttribute
}

impl MapAttributes {
    pub fn set_attribute(&mut self, renderer: &mut GpuRenderer, attribute: MapAttribute) {
        self.attribute = attribute.clone();
        self.image.set_color(MapAttribute::get_color(&self.attribute));
        self.text.set_text(renderer, MapAttribute::as_map_str(&self.attribute), Attrs::new());
        let size = self.text.measure();
        self.text.pos.x = self.pos.x + (TEXTURE_SIZE as f32 * 0.5) - (size.x * 0.5);
        self.text.changed = true;
    }
}

#[derive(Default)]
pub struct MapZone {
    pub pos: Vec<Vec2>,
}

#[derive(Default)]
pub struct MapZoneSetting {
    pub max_npc: u64,
    pub npc_id: [Option<u64>; 5],
}

pub struct MapView {
    pub maps: Vec<Map>,
    pub link_map_selection: Vec<Rect>,
    pub selection_preview: Rect,
    preview_pos: Vec2,
    preview_size: Vec2,

    pub map_attributes: Vec<MapAttributes>,
    pub map_zone: Vec<Rect>,
    pub map_zone_loc: [MapZone; 5],
    pub map_zone_setting: [MapZoneSetting; 5],
    pub fixed_weather: u8,

    // Recording
    pub record: Records,
}

impl MapView {
    pub fn new(draw_setting: &mut DrawSetting) -> Self {
        let mut maps = Vec::with_capacity(9);
        let mut link_map_selection = Vec::with_capacity(8);
        
        // Create 9 maps for our view of the main map and the surrounding maps
        for count in 0..9 {
            let mut map = Map::new(&mut draw_setting.renderer, TEXTURE_SIZE);

            // Set default position of each view
            // Note: Index '0' is the main view on the center
            // while the other view are for surrounding maps
            match count {
                1 => { map.pos = Vec2::new(215.0, 719.0); }, // Top Left
                2 => { map.pos = Vec2::new(257.0, 719.0); }, // Top
                3 => { map.pos = Vec2::new(899.0, 719.0); }, // Top Right
                4 => { map.pos = Vec2::new(215.0, 77.0); }, // Left
                5 => { map.pos = Vec2::new(899.0, 77.0); }, // Right
                6 => { map.pos = Vec2::new(215.0, 35.0); }, // Bottom Left
                7 => { map.pos = Vec2::new(257.0, 35.0); }, // Bottom
                8 => { map.pos = Vec2::new(899.0, 35.0); }, // Bottom Right
                _ => { map.pos = Vec2::new(257.0, 77.0); }, // Center / Main
            }

            map.can_render = true;
            maps.push(map);
        };

        // We add the link selection overlay above the link map as a selecting effect
        for count in 0..8 {
            let mut image = Rect::new(&mut draw_setting.renderer, 0);
            image.set_size(match count {
                                1 => { Vec2::new(TEXTURE_SIZE as f32 * 32.0, TEXTURE_SIZE as f32 * 2.0) }, // Top
                                2 => { Vec2::new(TEXTURE_SIZE as f32 * 2.0, TEXTURE_SIZE as f32 * 2.0) }, // Top Right
                                3 => { Vec2::new(TEXTURE_SIZE as f32 * 2.0, TEXTURE_SIZE as f32 * 32.0) }, // Left
                                4 => { Vec2::new(TEXTURE_SIZE as f32 * 2.0, TEXTURE_SIZE as f32 * 32.0) }, // Right
                                5 => { Vec2::new(TEXTURE_SIZE as f32 * 2.0, TEXTURE_SIZE as f32 * 2.0) }, // Bottom Left
                                6 => { Vec2::new(TEXTURE_SIZE as f32 * 32.0, TEXTURE_SIZE as f32 * 2.0) }, // Bottom
                                7 => { Vec2::new(TEXTURE_SIZE as f32 * 2.0, TEXTURE_SIZE as f32 * 2.0) }, // Bottom Right
                                _ => { Vec2::new(TEXTURE_SIZE as f32 * 2.0, TEXTURE_SIZE as f32 * 2.0) }, // Top Left
                            })
                    // We set the link selection image at the same position as the linked map
                    // We add +1 on the count as the linked map started on index 1 instead of 0
                    .set_position(Vec3::new(maps[count + 1].pos.x, maps[count + 1].pos.y, ORDER_MAP_LINK_SELECT))
                    .set_color(Color::rgba(0, 0, 0, 130))
                    .set_use_camera(true);
            
            link_map_selection.push(image);
        }

        // This will create the selection box on the map view
        let mut selection_preview = Rect::new(&mut draw_setting.renderer, 0);
        selection_preview.set_size(Vec2::new(TEXTURE_SIZE as f32, TEXTURE_SIZE as f32))
                        .set_position(Vec3::new(maps[0].pos.x, maps[0].pos.y, ORDER_MAP_SELECTION))
                        .set_color(Color::rgba(0, 0, 150, 150))
                        .set_use_camera(true);

        // Map Attributes & Map Zones
        let mut map_attributes = Vec::with_capacity(1024);
        let mut map_zone = Vec::with_capacity(1024);
        for i in 0..1024 {
            let pos = Vec2::new(maps[0].pos.x + ((i % 32) * TEXTURE_SIZE) as f32,
                                    maps[0].pos.y + ((i / 32) * TEXTURE_SIZE) as f32);
            // BG
            let mut image = Rect::new(&mut draw_setting.renderer, 0);
            image.set_size(Vec2::new(TEXTURE_SIZE as f32, TEXTURE_SIZE as f32))
                .set_position(Vec3::new(pos.x, pos.y, ORDER_MAP_ATTRIBUTE_BG))
                .set_color(Color::rgba(0,0,0,0))
                .set_use_camera(true);

            // Text
            let label_size = Vec2::new(32.0, 32.0);
            let mut text = Text::new(
                &mut draw_setting.renderer,
                Some(Metrics::new(16.0, 16.0).scale(draw_setting.scale as f32)),
                Vec3::new(pos.x, pos.y - 13.0, ORDER_MAP_ATTRIBUTE_TEXT), 
                label_size,
                1.0
            );
            text.set_buffer_size(&mut draw_setting.renderer, draw_setting.size.width as i32, draw_setting.size.height as i32)
                .set_bounds(Some(Bounds::new(pos.x, pos.y, pos.x + label_size.x, pos.y + label_size.y)))
                .set_default_color(Color::rgba(255,255,255,255));
            text.use_camera = true;
            text.changed = true;

            map_attributes.push(MapAttributes {
                pos,
                image,
                text,
                attribute: MapAttribute::Walkable,
            });

            // Zone BG
            let mut zone_box = Rect::new(&mut draw_setting.renderer, 0);
            zone_box.set_size(Vec2::new(TEXTURE_SIZE as f32, TEXTURE_SIZE as f32))
                .set_position(Vec3::new(pos.x, pos.y, ORDER_MAP_ZONE))
                .set_color(Color::rgba(0,0,0,0))
                .set_use_camera(true);
            map_zone.push(zone_box);
        }
        

        Self {
            maps,
            link_map_selection,
            selection_preview,
            preview_pos: Vec2::new(0.0, 0.0),
            preview_size: Vec2::new(1.0, 1.0),
            map_attributes,
            map_zone,
            map_zone_loc: Default::default(),
            map_zone_setting: Default::default(),
            record: Records::new(),
            fixed_weather: 0,
        }
    }

    // This function create an effect when we are hovering on the linked map
    pub fn hover_linked_selection(&mut self, pos: Vec2) -> Option<usize> {
        let mut result = None;
        for (index, selection) in self.link_map_selection.iter_mut().enumerate() {
            let is_within_pos =
                pos.x >= selection.position.x
                    && pos.x <= selection.position.x + selection.size.x
                    && pos.y >= selection.position.y
                    && pos.y <= selection.position.y + selection.size.y as f32;
    
            if is_within_pos {
                if selection.color != Color::rgba(0, 0, 0, 0) {
                    selection.color = Color::rgba(0, 0, 0, 0);
                    selection.changed = true;
                }
                result = Some(index);
            } else {
                if selection.color != Color::rgba(0, 0, 0, 130) {
                    selection.color = Color::rgba(0, 0, 0, 130);
                    selection.changed = true;
                }
            }
        }
        result
    }

    pub fn set_attribute(&mut self, renderer: &mut GpuRenderer, set_pos: Vec2, attributes: MapAttribute) {
        let tilepos = get_tile_pos(set_pos.x as i32, set_pos.y as i32);

        // Record change for undo purpose
        let last_attribute = self.map_attributes[tilepos].attribute.clone();
        let last_attribute_num = MapAttribute::convert_to_num(&last_attribute);
        let data = match last_attribute {
            MapAttribute::Warp(mx, my, mg, tx, ty) => {
                vec![InsertTypes::Int(mx as i64),
                    InsertTypes::Int(my as i64),
                    InsertTypes::UInt(mg as u64),
                    InsertTypes::UInt(tx as u64),
                    InsertTypes::UInt(ty as u64)]
                },
            MapAttribute::Sign(text) => vec![InsertTypes::Str(text)],
            _ => vec![],
        };
        self.record.push_undo(Vec3::new(set_pos.x, set_pos.y, 0.0), RecordType::RecordAttribute, last_attribute_num as i64, data);
        
        self.map_attributes[tilepos].set_attribute(renderer, attributes);
    }

    pub fn get_attribute(&mut self, pos: Vec2) -> MapAttribute {
        let tilepos = get_tile_pos(pos.x as i32, pos.y as i32);
        self.map_attributes[tilepos].attribute.clone()
    }

    pub fn set_attribute_fill(&mut self, renderer: &mut GpuRenderer, set_pos: Vec2, attribute: MapAttribute) {
        let tilepos = get_tile_pos(set_pos.x as i32, set_pos.y as i32);

        // We will only change the tiles that have a similar texture id, and this will be use to check
        let comparedata = self.map_attributes[tilepos].attribute.clone();
        if comparedata == attribute {
            return;
        }

        // This will hold the location that need to be paint
        let mut paint_to_map: Vec<Vec2> = Vec::with_capacity(0);

        // Place our starting location on to be paint collection
        paint_to_map.push(set_pos);

        // Loop through our collections of position that requires to be paint
        while let Some(pos) = paint_to_map.pop() {
            // Record change for undo purpose
            let last_attribute = self.map_attributes[tilepos].attribute.clone();
            let last_attribute_num = MapAttribute::convert_to_num(&last_attribute);
            let data = match last_attribute {
                MapAttribute::Warp(mx, my, mg, tx, ty) => {
                    vec![InsertTypes::Int(mx as i64),
                        InsertTypes::Int(my as i64),
                        InsertTypes::UInt(mg as u64),
                        InsertTypes::UInt(tx as u64),
                        InsertTypes::UInt(ty as u64)]
                    },
                MapAttribute::Sign(text) => vec![InsertTypes::Str(text)],
                _ => vec![],
            };
            self.record.push_undo(Vec3::new(set_pos.x, set_pos.y, 0.0), RecordType::RecordAttribute, last_attribute_num as i64, data);

            // Paint the map
            let tile_pos = get_tile_pos(pos.x as i32, pos.y as i32);
            self.map_attributes[tile_pos].set_attribute(renderer, attribute.clone());

            // Check direction
            for dir in 0..4 {
                // Get the surrounding map position
                let mut adjust_pos = Vec2::new(0.0, 0.0);
                match dir {
                    1 => { adjust_pos.y = 1.0; }, // Up
                    2 => { adjust_pos.x = -1.0; }, // Left
                    3 => { adjust_pos.x = 1.0; }, // Right
                    _ => { adjust_pos.y = -1.0; }, // Down
                }
                let checkpos = pos + adjust_pos;

                if checkpos.x >= 0.0 && checkpos.x < 32.0 && checkpos.y >= 0.0 && checkpos.y < 32.0 {
                    // Check the map texture id and we make sure that we only change
                    // if they have the same texture id as the starting tile
                    let check_tilepos = get_tile_pos(checkpos.x as i32, checkpos.y as i32);
                    let check_data = self.map_attributes[check_tilepos].attribute.clone();
                    if check_data == comparedata {
                        paint_to_map.push(checkpos);
                    }
                }
            }
        }
    }

    pub fn set_tile_group(&mut self, set_pos: Vec2, layer: u32, tileset: &Map, start_pos: Vec2, selection_size: Vec2) {
        for x in 0..selection_size.x as u32 {
            for y in 0..selection_size.y as u32 {
                // We load the tile data from the tileset
                let tiledata = tileset.get_tile((start_pos.x as u32 + x, start_pos.y as u32 + y, 0));

                // Make sure we only add tile that are not empty
                if tiledata.id > 0 {
                    // Make sure we wont set map outside the map size limit
                    if (set_pos.x as u32 + x) < 32 && (set_pos.y as u32 + y) < 32 {
                        // Record change for undo purpose
                        let last_texture = self.maps[0].get_tile((set_pos.x as u32 + x, set_pos.y as u32 + y, layer)).id;
                        self.record.push_undo(Vec3::new(set_pos.x + x as f32, set_pos.y + y as f32, layer as f32), RecordType::RecordLayer, last_texture as i64, vec![]);

                        self.maps[0].set_tile((set_pos.x as u32 + x, set_pos.y as u32 + y, layer),tiledata);
                    }
                }
            }
        }
    }

    pub fn delete_tile_group(&mut self, set_pos: Vec2, layer: u32, size: Vec2) {
        for x in 0..size.x as u32 {
            for y in 0..size.y as u32 {
                // Make sure we wont set map outside the map size limit
                if (set_pos.x as u32 + x) < 32 && (set_pos.y as u32 + y) < 32 {
                    let texture_id = self.maps[0].get_tile((set_pos.x as u32 + x, set_pos.y as u32 + y, layer)).id;
                    if texture_id > 0 {
                        // Record change for undo purpose
                        let last_texture = self.maps[0].get_tile((set_pos.x as u32 + x, set_pos.y as u32 + y, layer)).id;
                        self.record.push_undo(Vec3::new(set_pos.x + x as f32, set_pos.y + y as f32, layer as f32), RecordType::RecordLayer, last_texture as i64, vec![]);
                        
                        self.maps[0].set_tile(
                            (set_pos.x as u32 + x, set_pos.y as u32 + y, layer), 
                            TileData::default());
                    }
                }
            }
        }
    }

    pub fn get_tile_data(&mut self, set_pos: Vec2) -> TileData {
        self.maps[0].get_tile((set_pos.x as u32, set_pos.y as u32, 0))
    }

    pub fn set_tile_fill(&mut self, set_pos: Vec2, layer: u32, tileset: &Map, tileset_pos: Vec2) {
        // Get the tile data from the tileset
        let tiledata = tileset.get_tile((tileset_pos.x as u32, tileset_pos.y as u32, 0));
        if tiledata.id == 0 {
            return;
        }

        // We will only change the tiles that have a similar texture id, and this will be use to check
        let comparedata = self.maps[0].get_tile((set_pos.x as u32, set_pos.y as u32, layer)).id;
        if comparedata == tiledata.id {
            return;
        }

        // This will hold the location that need to be paint
        let mut paint_to_map: Vec<Vec2> = Vec::with_capacity(0);

        // Place our starting location on to be paint collection
        paint_to_map.push(set_pos);

        // Loop through our collections of position that requires to be paint
        while let Some(pos) = paint_to_map.pop() {
            // Record change for undo purpose
            let last_texture = self.maps[0].get_tile((pos.x as u32, pos.y as u32, layer)).id;
            self.record.push_undo(Vec3::new(pos.x, pos.y, layer as f32), RecordType::RecordLayer, last_texture as i64, vec![]);

            // Paint the map
            self.maps[0].set_tile((pos.x as u32, pos.y as u32, layer), tiledata);
            
            // Check direction
            for dir in 0..4 {
                // Get the surrounding map position
                let mut adjust_pos = Vec2::new(0.0, 0.0);
                match dir {
                    1 => { adjust_pos.y = 1.0; }, // Up
                    2 => { adjust_pos.x = -1.0; }, // Left
                    3 => { adjust_pos.x = 1.0; }, // Right
                    _ => { adjust_pos.y = -1.0; }, // Down
                }
                let checkpos = pos + adjust_pos;

                if checkpos.x >= 0.0 && checkpos.x < 32.0 && checkpos.y >= 0.0 && checkpos.y < 32.0 {
                    // Check the map texture id and we make sure that we only change
                    // if they have the same texture id as the starting tile
                    let check_data = self.maps[0].get_tile((checkpos.x as u32, checkpos.y as u32, layer)).id;
                    if check_data == comparedata {
                        paint_to_map.push(checkpos);
                    }
                }
            }
        }
    }

    pub fn update_map_zone(&mut self, zone_index: usize) {
        // Clear all
        self.map_zone.iter_mut().for_each(|zone| {
            zone.set_color(Color::rgba(0,0,0,0));
        });
        // Add the selected zone
        for data in self.map_zone_loc[zone_index].pos.iter() {
            let tilenum = get_tile_pos(data.x as i32, data.y as i32);
            self.map_zone[tilenum].set_color(get_zone_color(zone_index));
        }
    }

    pub fn add_map_zone(&mut self, zone_index: usize, pos: Vec2) {
        // Record change for undo purpose
        let does_exist = if self.map_zone_loc[zone_index].pos.iter().any(|&check_pos| check_pos == pos) { 1 } else { 0 };
        self.record.push_undo(Vec3::new(pos.x, pos.y, zone_index as f32), RecordType::RecordZone, does_exist, vec![]);

        let tilenum = get_tile_pos(pos.x as i32, pos.y as i32);
        self.map_zone[tilenum].set_color(get_zone_color(zone_index));
        if !self.map_zone_loc[zone_index].pos.iter().any(|&check_pos| check_pos == pos) {
            self.map_zone_loc[zone_index].pos.push(pos);
        }
    }

    pub fn set_zone_fill(&mut self, set_pos: Vec2, zone_index: usize) {
        // Fill only empty area
        if self.map_zone_loc[zone_index].pos.iter().any(|&check_pos| check_pos == set_pos) {
            return;
        }

        // This will hold the location that need to be paint
        let mut paint_to_map: Vec<Vec2> = Vec::with_capacity(0);

        // Place our starting location on to be paint collection
        paint_to_map.push(set_pos);

        // Loop through our collections of position that requires to be paint
        while let Some(pos) = paint_to_map.pop() {
            // Record change for undo purpose
            let does_exist = if self.map_zone_loc[zone_index].pos.iter().any(|&check_pos| check_pos == pos) { 1 } else { 0 };
            self.record.push_undo(Vec3::new(pos.x, pos.y, zone_index as f32), RecordType::RecordZone, does_exist, vec![]);
            
            // Paint the map
            let tilenum = get_tile_pos(pos.x as i32, pos.y as i32);
            self.map_zone[tilenum].set_color(get_zone_color(zone_index));
            if !self.map_zone_loc[zone_index].pos.iter().any(|&check_pos| check_pos == pos) {
                self.map_zone_loc[zone_index].pos.push(pos);
            }
            
            // Check direction
            for dir in 0..4 {
                // Get the surrounding map position
                let mut adjust_pos = Vec2::new(0.0, 0.0);
                match dir {
                    1 => { adjust_pos.y = 1.0; }, // Up
                    2 => { adjust_pos.x = -1.0; }, // Left
                    3 => { adjust_pos.x = 1.0; }, // Right
                    _ => { adjust_pos.y = -1.0; }, // Down
                }
                let checkpos = pos + adjust_pos;

                if checkpos.x >= 0.0 && checkpos.x < 32.0 && checkpos.y >= 0.0 && checkpos.y < 32.0 {
                    // Check if zone is empty
                    if !self.map_zone_loc[zone_index].pos.iter().any(|&check_pos| check_pos == checkpos) {
                        paint_to_map.push(checkpos);
                    }
                }
            }
        }
    }

    pub fn delete_map_zone(&mut self, zone_index: usize, pos: Vec2) {
        // Record change for undo purpose
        let does_exist = if self.map_zone_loc[zone_index].pos.iter().any(|&check_pos| check_pos == pos) { 1 } else { 0 };
        self.record.push_undo(Vec3::new(pos.x, pos.y, zone_index as f32), RecordType::RecordZone, does_exist, vec![]);

        let tilenum = get_tile_pos(pos.x as i32, pos.y as i32);
        self.map_zone[tilenum].set_color(Color::rgba(0, 0, 0, 0));
        self.map_zone_loc[zone_index].pos.retain(|&check_pos| check_pos != pos);
    }

    pub fn hover_selection_preview(&mut self, set_pos: Vec2) {
        if self.preview_pos != set_pos && set_pos.x < 32.0 && set_pos.y < 32.0 {
            self.preview_pos = set_pos;
            self.selection_preview.set_position(Vec3::new(self.maps[0].pos.x + set_pos.x * TEXTURE_SIZE as f32, 
                                                        self.maps[0].pos.y + set_pos.y * TEXTURE_SIZE as f32, 
                                                        ORDER_MAP_SELECTION));
            self.adjust_selection_preview();
            self.selection_preview.changed = true;
        }
    }
    
    pub fn change_selection_preview_size(&mut self, size: Vec2) {
        self.preview_size = size;
        self.adjust_selection_preview();
        self.selection_preview.changed = true;
    }

    pub fn clear_map(&mut self, index: usize) {
        (0..8).for_each(|layer| {
            (0..32).for_each(|x| {
                (0..32).for_each(|y| {
                    self.maps[index].set_tile((x, y, layer), TileData::default());
                });
            });
        });
    }

    // This function ensure that the selection preview does not show outside the map boundary
    fn adjust_selection_preview(&mut self) {
        let max_size = Vec2::new(32.0, 32.0);
    
        let clamped_x = (self.preview_pos.x + self.preview_size.x).min(max_size.x);
        let clamped_y = (self.preview_pos.y + self.preview_size.y).min(max_size.y);

        let new_size = Vec2::new(clamped_x - self.preview_pos.x, clamped_y - self.preview_pos.y);

        self.selection_preview.set_size(Vec2::new(new_size.x * TEXTURE_SIZE as f32, 
                                                    new_size.y * TEXTURE_SIZE as f32));
    }

    pub fn apply_change(&mut self, renderer: &mut GpuRenderer, is_undo: bool) {
        let record_list = if is_undo { &self.record.undo } else { &self.record.redo };
        if record_list.is_empty() {
            return;
        }

        let get_change = if is_undo {
            self.record.get_last_undo()
        } else {
            self.record.get_last_redo()
        };

        if let Some(data) = get_change {
            if is_undo {
                self.record.set_redo_record();
            } else {
                self.record.set_undo_record();
            };

            for (_key, changedata) in data.changes.iter() {
                let pos = Vec3::new(changedata.pos.x, changedata.pos.y, changedata.pos.z);
    
                match changedata.record_type {
                    RecordType::RecordLayer => {
                        let last_texture = self.maps[0].get_tile((pos.x as u32, pos.y as u32, pos.z as u32)).id;
                        if is_undo {
                            self.record.push_redo(Vec3::new(pos.x, pos.y, pos.z), RecordType::RecordLayer, last_texture as i64, vec![]);
                        } else {
                            self.record.push_undo(Vec3::new(pos.x, pos.y, pos.z), RecordType::RecordLayer, last_texture as i64, vec![]);
                        }
    
                        let texture_id = changedata.id as u32;
                        self.maps[0].set_tile((pos.x as u32, pos.y as u32, pos.z as u32),
                            TileData {
                                id: texture_id as usize,
                                color: Color::rgba(255, 255, 255, 255),
                            });
                    },
                    RecordType::RecordAttribute => {
                        let tilenum = get_tile_pos(pos.x as i32, pos.y as i32);
                        let last_attribute = self.map_attributes[tilenum].attribute.clone();
                        let last_attribute_num = MapAttribute::convert_to_num(&last_attribute);
                        let data = match last_attribute {
                            MapAttribute::Warp(mx, my, mg, tx, ty) => {
                                vec![InsertTypes::Int(mx as i64),
                                    InsertTypes::Int(my as i64),
                                    InsertTypes::UInt(mg as u64),
                                    InsertTypes::UInt(tx as u64),
                                    InsertTypes::UInt(ty as u64)]
                            },
                            MapAttribute::Sign(text) => vec![InsertTypes::Str(text)],
                            _ => vec![],
                        };
                        if is_undo {
                            self.record.push_redo(Vec3::new(pos.x, pos.y, pos.z), RecordType::RecordAttribute, last_attribute_num as i64, data);
                        } else {
                            self.record.push_undo(Vec3::new(pos.x, pos.y, pos.z), RecordType::RecordAttribute, last_attribute_num as i64, data);
                        }
    
                        let attribute_enum = MapAttribute::convert_to_enum(changedata.id as u32, &changedata.data);
                        let tilenum = get_tile_pos(pos.x as i32, pos.y as i32);
                        self.map_attributes[tilenum].set_attribute(renderer, attribute_enum);
                    },
                    RecordType::RecordZone => {
                        // We will use the pos.z for the selected zone index
                        let zone_index = pos.z as usize;

                        let does_exist = if self.map_zone_loc[zone_index].pos.iter().any(|&check_pos| check_pos == Vec2::new(pos.x, pos.y)) { 1 } else { 0 };
                        if is_undo {
                            self.record.push_redo(Vec3::new(pos.x, pos.y, pos.z), RecordType::RecordZone, does_exist, vec![]);
                        } else {
                            self.record.push_undo(Vec3::new(pos.x, pos.y, pos.z), RecordType::RecordZone, does_exist, vec![]);
                        }

                        let tilenum = get_tile_pos(pos.x as i32, pos.y as i32);
                        if changedata.id > 0 { // Exist, add zone
                            self.map_zone[tilenum].set_color(get_zone_color(zone_index));
                            if !self.map_zone_loc[zone_index].pos.iter().any(|&check_pos| check_pos == Vec2::new(pos.x, pos.y)) {
                                self.map_zone_loc[zone_index].pos.push(Vec2::new(pos.x, pos.y));
                            }
                        } else { // Does not exist, remove zone
                            self.map_zone[tilenum].set_color(Color::rgba(0, 0, 0, 0));
                            self.map_zone_loc[zone_index].pos.retain(|&check_pos| check_pos != Vec2::new(pos.x, pos.y));
                        }
                    },
                }
            }
            self.record.stop_record();
        }
    }
}

pub fn get_zone_color(zone_index: usize) -> Color {
    match zone_index {
        1 => Color::rgba(200, 40, 40, 140),
        2 => Color::rgba(40, 200, 40, 140),
        3 => Color::rgba(150, 40, 150, 140),
        4 => Color::rgba(40, 150, 150, 140),
        _ => Color::rgba(40, 40, 200, 140),
    }
}