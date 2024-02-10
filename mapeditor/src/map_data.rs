use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::BufReader;
use std::path::Path;
use indexmap::IndexMap;
use graphics::*;

use crate::{
    attributes::*, map::*, DrawSetting, Interface
};

#[derive(Debug)]
pub enum Direction {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

pub struct EditorData {
    // Map ID
    pub x: i32,
    pub y: i32,
    pub group: u64,

    // Loaded Maps
    pub current_index: String,
    pub maps: IndexMap<String, MapData>,
    pub did_map_change: IndexMap<String, bool>,
}

impl EditorData {
    pub fn new() -> Result<EditorData, AscendingError> {
        let mut maps = IndexMap::new();
        let mut did_map_change = IndexMap::new();

        let current_index = format!("{}_{}_{}", 0, 0, 0);
        let map = load_file(0, 0, 0)?;
        maps.insert("0_0_0".to_string(), map);
        did_map_change.insert("0_0_0".to_string(), false);

        Ok(Self {
            x: 0,
            y: 0,
            group: 0,
            current_index,
            maps,
            did_map_change,
        })
    }

    pub fn init_map(&mut self, x: i32, y: i32, group: u64) {
        // Set position
        self.x = x;
        self.y = y;
        self.group = group;
        let key_data = format!("{}_{}_{}", x, y, group);
        
        if self.maps.contains_key(&key_data) {
            // Since the map is already loaded, we just switch the center map
            self.current_index = key_data;
        } else {
            // Change current center map
            self.current_index = key_data;
            // Since the map is not loaded, we must load the file and add it on the loaded maps
            let map = load_file(self.x, self.y, self.group).unwrap();
            self.maps.insert(self.current_index.clone(), map);
            self.did_map_change.insert(self.current_index.clone(), false);
        }
    }

    pub fn move_map(&mut self, direction: Direction) -> Option<String> {
        match direction {
            Direction::East => { self.x += 1; },
            Direction::North => { self.y += 1; },
            Direction::South => { self.y -= 1; },
            Direction::West => { self.x -= 1; },
            Direction::NorthEast => { self.x += 1; self.y += 1; },
            Direction::NorthWest => { self.x -= 1; self.y += 1; },
            Direction::SouthEast => { self.x += 1; self.y -= 1; },
            Direction::SouthWest => { self.x -= 1; self.y -= 1; },
        }
        let mut temp_key = None;

        // Check if the current map has changes
        if let Some(change) = self.did_map_change.get(&self.current_index) {
            // We will remove the map on the collection when no change has been done
            if !change {
                self.did_map_change.shift_remove(&self.current_index);
                self.maps.shift_remove(&self.current_index);
            } else {
                temp_key = Some(self.current_index.clone());
            }
        }

        let key_data = format!("{}_{}_{}", self.x, self.y, self.group);
        if self.maps.contains_key(&key_data) {
            // Since the map is already loaded, we just switch the center map
            self.current_index = key_data;
        } else {
            // Change current center map
            self.current_index = key_data;
            // Since the map is not loaded, we must load the file and add it on the loaded maps
            let map = load_file(self.x, self.y, self.group).unwrap();
            self.maps.insert(self.current_index.clone(), map);
            self.did_map_change.insert(self.current_index.clone(), false);
        }
        temp_key
    }

    pub fn save_map_data(&mut self, mapview: &MapView, old_map_key: Option<String>) {
        // Check if the map should be save as file or temporary data
        let (should_save, find_key);
        if old_map_key.is_some() {
            should_save = false;
            find_key = old_map_key.unwrap();
        } else {
            should_save = true;
            find_key = self.current_index.clone();
        }
        // This handles the copying of data from map tiles to map data
        if let Some(mapdata) = self.maps.get_mut(&find_key) {
            (0..32).for_each(|x| {
                (0..32).for_each(|y| {
                    let tile_num = get_tile_pos(x, y);
                    (0..8).for_each(|layer| {
                        mapdata.tile[layer].id[tile_num] = mapview.maps[0].get_tile((x as u32, y as u32, layer as u32)).id as u32;
                    });
                    mapdata.attribute[tile_num] = mapview.map_attributes[tile_num].attribute.clone();
                });
            });
            for i in 0..5 {
                mapdata.zonespawns[i] = Vec::new();
                mapview.map_zone_loc[i].pos.iter().for_each(|zone_pos| {
                    mapdata.zonespawns[i].push((zone_pos.x as u16, zone_pos.y as u16));
                });
                mapdata.zones[i].0 = mapview.map_zone_setting[i].max_npc;
                for npc_index in 0..5 {
                    mapdata.zones[i].1[npc_index] = mapview.map_zone_setting[i].npc_id[npc_index];
                }
            }
            mapdata.fixed_weather = mapview.fixed_weather;
            if should_save {
                mapdata.save_file().unwrap();
                // Since we have saved the map, let's mark the map as 'no change'
                if let Some(did_change) = self.did_map_change.get_mut(&self.current_index) {
                    *did_change = false;
                }
            }
        }
    }

    pub fn save_all_maps(&mut self, mapview: &MapView) {
        let keys_to_remove: Vec<_> = self.did_map_change
            .keys()
            .filter(|&key| self.did_map_change[key])
            .cloned()
            .collect();
    
        for key in keys_to_remove {
            let mut should_remove = true;
            if let Some(mapdata) = self.maps.get_mut(&key) {
                if self.x == mapdata.x && self.y == mapdata.y && self.group == mapdata.group {
                    should_remove = false;
                    if let Some(did_change) = self.did_map_change.get_mut(&key) {
                        *did_change = false;
                    }
                    self.save_map_data(mapview, None);
                } else {
                    mapdata.save_file().unwrap();
                }
            }
            if should_remove {
                self.maps.shift_remove(&key);
                self.did_map_change.shift_remove(&key);
            }
        }
    }

    pub fn reset_all_map(&mut self) {
        let keys_to_reset: Vec<_> = self.did_map_change
            .keys()
            .filter(|&key| self.did_map_change[key])
            .cloned()
            .collect();
    
        for key in keys_to_reset {
            if let Some(mapdata) = self.maps.get_mut(&key) {
                if self.x == mapdata.x && self.y == mapdata.y && self.group == mapdata.group {
                    if let Some(did_change) = self.did_map_change.get_mut(&key) {
                        *did_change = false;
                    }
                    if let Some(mapdata) = self.maps.get_mut(&key) {
                        *mapdata = load_file(self.x, self.y, self.group).unwrap();
                    }
                } else {
                    self.maps.shift_remove(&key);
                    self.did_map_change.shift_remove(&key);
                }
            }
        }
    }
    
    pub fn load_map_data(&mut self, renderer: &mut GpuRenderer, map: &mut MapView) {
        // Clear the map before we start adding the tiles
        map.clear_map(0);
        // Add the tiles
        if let Some(mapdata) = self.maps.get(&self.current_index) {
            (0..32).for_each(|x| {
                (0..32).for_each(|y| {
                    let tile_num = get_tile_pos(x, y);
                    (0..8).for_each(|layer| {
                        let id = mapdata.tile[layer].id[tile_num] as usize;
                        if id > 0 {
                            map.maps[0].set_tile((x as u32, y as u32, layer as u32), 
                                        TileData { 
                                            id,
                                            color: Color::rgba(255, 255, 255, 255),
                                        });
                        }
                    });
                    map.map_attributes[tile_num].set_attribute(renderer, mapdata.attribute[tile_num].clone());
                });
            });
            for i in 0..5 {
                map.map_zone_loc[i].pos = Vec::new();
                mapdata.zonespawns[i].iter().for_each(|zone| {
                    map.map_zone_loc[i].pos.push(Vec2::new(zone.0 as f32, zone.1 as f32));
                });
                map.map_zone_setting[i].max_npc = mapdata.zones[i].0;
                for npc_index in 0..5 {
                    map.map_zone_setting[i].npc_id[npc_index] = mapdata.zones[i].1[npc_index];
                }
            }
            map.fixed_weather = mapdata.fixed_weather;
        }
    }

    pub fn load_link_maps(&mut self, map: &mut MapView) {
        (0..8).for_each(|maplink| {
            // Clear the map before we start adding the tiles
            map.clear_map(maplink + 1);

            // Set the map id, position for loading
            let (start, size, x, y);
            match maplink {
                1 => { // Top
                    x = self.x; y = self.y + 1;
                    size = Vec2::new(32.0, 2.0);
                    start = Vec2::new(0.0, 0.0);
                },
                2 => { // Top Right
                    x = self.x + 1; y = self.y + 1;
                    size = Vec2::new(2.0, 2.0);
                    start = Vec2::new(0.0, 0.0);
                },
                3 => { // Left
                    x = self.x - 1; y = self.y;
                    size = Vec2::new(2.0, 32.0);
                    start = Vec2::new(30.0, 0.0);
                },
                4 => { // Right
                    x = self.x + 1; y = self.y;
                    size = Vec2::new(2.0, 32.0);
                    start = Vec2::new(0.0, 0.0);
                },
                5 => { // Bottom Left
                    x = self.x - 1; y = self.y - 1;
                    size = Vec2::new(2.0, 2.0);
                    start = Vec2::new(30.0, 30.0);
                },
                6 => { // Bottom
                    x = self.x; y = self.y - 1;
                    size = Vec2::new(32.0, 2.0);
                    start = Vec2::new(0.0, 30.0);
                },
                7 => { // Bottom Right
                    x = self.x + 1; y = self.y - 1;
                    size = Vec2::new(2.0, 2.0);
                    start = Vec2::new(0.0, 30.0);
                },
                _ => { // Top Left
                    x = self.x - 1; y = self.y + 1;
                    size = Vec2::new(2.0, 2.0);
                    start = Vec2::new(30.0, 0.0);
                },
            }
            let key = format!("{}_{}_{}", x, y, self.group);

            // Let's check if map exist, and only load if map exist
            if is_map_exist(x, y, self.group) {
                // Check if map is already on our indexmap, otherwise we load it
                let mapdata = if !self.maps.contains_key(&key) {
                    // Since the map is not loaded, we must load the file and add it on the loaded maps
                    load_file(x, y, self.group).unwrap()
                } else {
                    self.maps.get(&key).unwrap().clone()
                };
                
                (0..size.x as i32).for_each(|x| {
                    (0..size.y as i32).for_each(|y| {
                        (0..8).for_each(|layer| {
                            let tile_num = get_tile_pos(start.x as i32 + x, start.y as i32 + y);
                            let id = mapdata.tile[layer].id[tile_num] as usize;
                                
                            if id > 0 {
                                map.maps[maplink + 1].set_tile((x as u32, y as u32, layer as u32), 
                                            TileData { 
                                                id,
                                                color: Color::rgba(255, 255, 255, 255),
                                            });
                            }
                        });
                    });
                });
            }
        });
    }

    pub fn set_map_change(&mut self,
                        mapview: &mut MapView)
    {
        if let Some(did_change) = self.did_map_change.get_mut(&self.current_index) {
            *did_change = true;
            mapview.record.clear_redo();
        }
    }

    pub fn got_changes(&mut self) -> bool {
        self.did_map_change.values().any(|&value| value)
    }

    pub fn did_change(&self, x: i32, y: i32, group: u64) -> bool {
        let key_data = format!("{}_{}_{}", x, y, group);
        if !self.did_map_change.contains_key(&key_data) {
            return false;
        }
        *self.did_map_change.get(&key_data).unwrap()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub id: Vec<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MapData {
    pub x: i32,
    pub y: i32,
    pub group: u64,
    pub tile: Vec<Tile>,
    pub attribute: Vec<MapAttribute>,
    pub zonespawns: [Vec<(u16, u16)>; 5],
    pub zones: [(u64, [Option<u64>; 5]); 5],
    pub fixed_weather: u8,
}

impl MapData {
    pub fn default(x: i32, y: i32, group: u64) -> Self {
        Self {
            x,
            y,
            group,
            tile: vec![Tile { id: vec![0; 1024] }; 8],
            attribute: vec![MapAttribute::Walkable; 1024],
            zonespawns: Default::default(),
            zones: Default::default(),
            fixed_weather: 0,
        }
    }

    pub fn save_file(&self) -> Result<(), AscendingError> {
        let name = format!("./data/maps/{}_{}_{}.json", self.x, self.y, self.group);

        match OpenOptions::new().truncate(true).write(true).open(&name) {
            Ok(file) => {
                if let Err(e) = serde_json::to_writer_pretty(&file, self) {
                    Err(AscendingError::Other(OtherError::new(&format!("Serdes File Error Err {:?}", e))))
                } else {
                    Ok(())
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
            Err(e) => Err(AscendingError::Other(OtherError::new(&format!("Failed to open {}, Err {:?}", name, e)))),
        }
    }
}

pub fn create_file(x: i32, y: i32, group: u64, data: &MapData) -> Result<(), AscendingError> {
    let name = format!("./data/maps/{}_{}_{}.json", x, y, group);

    match OpenOptions::new().write(true).create_new(true).open(&name) {
        Ok(file) => {
            if let Err(e) = serde_json::to_writer_pretty(&file, &data) {
                Err(AscendingError::Other(OtherError::new(&format!("Serdes File Error Err {:?}", e))))
            } else {
                Ok(())
            }
        }
        Err(ref e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
        Err(e) => Err(AscendingError::Other(OtherError::new(&format!("Failed to open {}, Err {:?}", name, e)))),
    }
}

pub fn load_file(x: i32, y: i32, group: u64) -> Result<MapData, AscendingError> {
    if !is_map_exist(x, y, group) {
        let data = MapData::default(x,y,group);
        match create_file(x, y, group, &MapData::default(x,y,group)) {
            Ok(()) => return Ok(data),
            Err(e) => return Err(e),
        }
    }

    let name = format!("./data/maps/{}_{}_{}.json", x, y, group);
    match OpenOptions::new().read(true).open(&name) {
        Ok(file) => {
            let reader = BufReader::new(file);

            match serde_json::from_reader(reader) {
                Ok(data) => Ok(data),
                Err(e) => {
                    println!("Error {:?}", e);
                    Ok(MapData::default(x,y,group))
                }
            }
        }
        Err(e) => Err(AscendingError::Other(OtherError::new(&format!("Failed to open {}, Err {:?}", name, e)))),
    }
}

pub fn is_map_exist(x: i32, y: i32, group: u64) -> bool {
    let name = format!("./data/maps/{}_{}_{}.json", x, y, group);
    Path::new(&name).exists()
}

pub fn get_tile_pos(x: i32, y: i32) -> usize {
    (x + (y * 32 as i32)) as usize
}

pub fn convert_to_dir(dir: usize) -> Direction {
    match dir {
        1 => { Direction::North },
        2 => { Direction::NorthEast },
        3 => { Direction::West },
        4 => { Direction::East },
        5 => { Direction::SouthWest },
        6 => { Direction::South },
        7 => { Direction::SouthEast },
        _ => { Direction::NorthWest },
    }
}