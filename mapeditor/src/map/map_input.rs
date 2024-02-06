use graphics::*;
use crate::{
    map::*,
    tileset::*,
    interface::*,
    game_input::*,
    DrawSetting,
};

pub fn interact_with_map(
    draw_setting: &mut DrawSetting,
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
                    mapview.set_tile_group(tile_pos,gui.get_tab_option_data(),&tileset.map,tileset.select_start,tileset.select_size);
                    if editor_data.set_map_change() {
                        update_map_name(draw_setting, gui, editor_data);
                    };
                    mapview.record.clear_redo();
                }
                TOOL_ERASE => {
                    mapview.delete_tile_group(tile_pos,gui.get_tab_option_data(),tileset.select_size);
                    if editor_data.set_map_change() {
                        update_map_name(draw_setting, gui, editor_data);
                    };
                    mapview.record.clear_redo();
                }
                TOOL_FILL => {
                    mapview.set_tile_fill(tile_pos,gui.get_tab_option_data(),&tileset.map,tileset.select_start);
                    if editor_data.set_map_change() {
                        update_map_name(draw_setting, gui, editor_data);
                    };
                    mapview.record.clear_redo();
                }
                TOOL_EYEDROP => {
                    let tiledata = mapview.get_tile_data(tile_pos);
                    let id = tiledata.id;
                    if let Some((x, y, tile)) = draw_setting.resource.tile_location.get(&id) {
                        // Change the loaded tileset
                        gui.tileset_list.selected_tileset = tile.clone() as usize;
                        gui.labels[LABEL_TILESET].set_text(&mut draw_setting.renderer,&draw_setting.resource.tilesheet[gui.tileset_list.selected_tileset].name, Attrs::new());
                        tileset.change_tileset(&draw_setting.resource, gui.tileset_list.selected_tileset);
                        gui.tileset_list.update_list(&mut draw_setting.renderer, &draw_setting.resource);

                        // Set the selected tile position
                        let (posx, posy) = (x / TEXTURE_SIZE, (MAX_TILE_Y - (y / TEXTURE_SIZE) - 1));
                        gameinput.tileset_start = Vec2::new(posx as f32, posy as f32);
                        gameinput.tileset_end = Vec2::new(posx as f32, posy as f32);
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
        TAB_ATTRIBUTE => {
            match gui.current_tool {
                TOOL_DRAW => {
                    let attribute = gui.get_attribute();
                    mapview.set_attribute(&mut draw_setting.renderer, tile_pos, attribute);
                    if editor_data.set_map_change() {
                        update_map_name(draw_setting, gui, editor_data);
                    };
                    mapview.record.clear_redo();
                },
                TOOL_ERASE => {
                    mapview.set_attribute(&mut draw_setting.renderer, tile_pos, MapAttribute::Walkable);
                    if editor_data.set_map_change() {
                        update_map_name(draw_setting, gui, editor_data);
                    };
                    mapview.record.clear_redo();
                },
                TOOL_EYEDROP => {
                    let attribute = mapview.get_attribute(tile_pos);
                    if attribute != MapAttribute::Walkable {
                        let attribute_index = MapAttribute::convert_to_num(&attribute);
                        let data = match attribute {
                            MapAttribute::Warp(mx, my, mg, tx, ty) => {
                                vec![InsertTypes::Int(mx as i64), 
                                    InsertTypes::Int(my as i64), 
                                    InsertTypes::UInt(mg as u64), 
                                    InsertTypes::UInt(tx as u64), 
                                    InsertTypes::UInt(ty as u64)]
                            }
                            MapAttribute::Sign(text) => {
                                vec![InsertTypes::Str(text)]
                            }
                            _ => vec![],
                        };
                        gui.select_tab_option(attribute_index as usize - 1);
                        gui.open_attribute_settings(draw_setting, attribute_index, data)
                    }
                },
                TOOL_FILL => {
                    let attribute = gui.get_attribute();
                    mapview.set_attribute_fill(&mut draw_setting.renderer, tile_pos,attribute);
                    if editor_data.set_map_change() {
                        update_map_name(draw_setting, gui, editor_data);
                    };
                    mapview.record.clear_redo();
                },
                _ => {},
            }
        }
        TAB_ZONE => {
            match gui.current_tool {
                TOOL_DRAW => {
                    mapview.add_map_zone(gui.current_tab_data as usize, tile_pos);
                    if editor_data.set_map_change() {
                        update_map_name(draw_setting, gui, editor_data);
                    };
                    mapview.record.clear_redo();
                }
                TOOL_ERASE => {
                    mapview.delete_map_zone(gui.current_tab_data as usize, tile_pos);
                    if editor_data.set_map_change() {
                        update_map_name(draw_setting, gui, editor_data);
                    };
                    mapview.record.clear_redo();
                }
                TOOL_FILL => {
                    mapview.set_zone_fill(tile_pos, gui.current_tab_data as usize);
                    if editor_data.set_map_change() {
                        update_map_name(draw_setting, gui, editor_data);
                    };
                    mapview.record.clear_redo();
                }
                _ => {}
            }
        }
        TAB_PROPERTIES => {}
        _ => {}
    }
}