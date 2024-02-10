use crate::editor_input::*;

fn interact_with_map(
    systems: &mut DrawSetting,
    tile_pos: Vec2,
    gui: &mut Interface,
    tileset: &mut Tileset,
    mapview: &mut MapView,
    database: &mut EditorData,
    gameinput: &mut GameInput,
) {
    match gui.current_tab {
        TAB_LAYER => {
            match gui.current_tool {
                TOOL_DRAW => {
                    mapview.set_tile_group(tile_pos,gui.get_tab_option_data(),&tileset.map,tileset.select_start,tileset.select_size);
                    database.set_map_change(mapview);
                    update_map_name(systems, gui, database);
                }
                TOOL_ERASE => {
                    mapview.delete_tile_group(tile_pos,gui.get_tab_option_data(),tileset.select_size);
                    database.set_map_change(mapview);
                    update_map_name(systems, gui, database);
                }
                TOOL_FILL => {
                    mapview.set_tile_fill(tile_pos,gui.get_tab_option_data(),&tileset.map,tileset.select_start);
                    database.set_map_change(mapview);
                    update_map_name(systems, gui, database);
                }
                TOOL_EYEDROP => {
                    let tiledata = mapview.get_tile_data(tile_pos);
                    let id = tiledata.id;
                    if let Some((x, y, tile)) = systems.resource.tile_location.get(&id) {
                        // Change the loaded tileset
                        gui.tileset_list.selected_tileset = tile.clone() as usize;
                        gui.labels[LABEL_TILESET].set_text(&mut systems.renderer,&systems.resource.tilesheet[gui.tileset_list.selected_tileset].name, Attrs::new());
                        tileset.change_tileset(&systems.resource, gui.tileset_list.selected_tileset);
                        gui.tileset_list.update_list(&mut systems.renderer, &systems.resource);

                        // Set the selected tile position
                        let (posx, posy) = (x / TEXTURE_SIZE, (MAX_TILE_Y - (y / TEXTURE_SIZE) - 1));
                        gameinput.tileset_start = Vec2::new(posx as f32, posy as f32);
                        gameinput.tileset_end = Vec2::new(posx as f32, posy as f32);
                        gameinput.selected_size = tileset.set_selection(
                            gameinput.tileset_start,
                            gameinput.tileset_end,
                        );
                        mapview.change_selection_preview_size(
                            gameinput.selected_size,
                        );
                    }
                }
                _ => {}
            }
        }
        TAB_ATTRIBUTE => {
            match gui.current_tool {
                TOOL_DRAW => {
                    let attribute = gui.get_attribute_setting();
                    mapview.set_attribute(&mut systems.renderer, tile_pos, attribute);
                    database.set_map_change(mapview);
                    update_map_name(systems, gui, database);
                }
                TOOL_ERASE => {
                    mapview.set_attribute(&mut systems.renderer, tile_pos, MapAttribute::Walkable);
                    database.set_map_change(mapview);
                    update_map_name(systems, gui, database);
                }
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
                        open_attribute_settings(systems, gui, attribute_index, data);
                    }
                }
                TOOL_FILL => {
                    let attribute = gui.get_attribute_setting();
                    mapview.set_attribute_fill(&mut systems.renderer, tile_pos,attribute);
                    database.set_map_change(mapview);
                    update_map_name(systems, gui, database);
                }
                _ => {}
            }
        }
        TAB_ZONE => {
            match gui.current_tool {
                TOOL_DRAW => {
                    mapview.add_map_zone(gui.current_tab_data as usize, tile_pos);
                    database.set_map_change(mapview);
                    update_map_name(systems, gui, database);
                }
                TOOL_ERASE => {
                    mapview.delete_map_zone(gui.current_tab_data as usize, tile_pos);
                    database.set_map_change(mapview);
                    update_map_name(systems, gui, database);
                }
                TOOL_FILL => {
                    mapview.set_zone_fill(tile_pos, gui.current_tab_data as usize);
                    database.set_map_change(mapview);
                    update_map_name(systems, gui, database);
                }
                _ => {}
            }
        }
        TAB_PROPERTIES => {}
        _ => {}
    }
}

pub fn mapview_input(
    systems: &mut DrawSetting,
    inputtype: &InputType,
    screen_pos: Vec2,
    gameinput: &mut GameInput,
    gui: &mut Interface,
    tileset: &mut Tileset,
    mapview: &mut MapView,
    database: &mut EditorData,
) {
    match inputtype {
        InputType::MouseLeftDown => {
            if !is_scrollbar_in_hold(gui) {
                // Check if mouse position is pointing to our map view
                if in_map(screen_pos, mapview) {
                    mapview.record.set_undo_record();
                    interact_with_map(
                        systems,
                        get_map_pos(screen_pos, mapview),
                        gui,
                        tileset,
                        mapview,
                        database,
                        gameinput,
                    );
                    gameinput.presstype = PressType::PressMap;
                }

                // Linked Map
                if gameinput.selected_link_map.is_some() {
                    let direction =
                        convert_to_dir(gameinput.selected_link_map.unwrap());
                    let temp_key = database.move_map(direction);
                    if temp_key.is_some() {
                        // We will store a temporary map data when changes happen
                        database.save_map_data(&mapview, temp_key);
                    };
                    // Load the initial map
                    database.load_map_data(&mut systems.renderer, mapview);
                    database.load_link_maps(mapview);
                    update_map_name(systems, gui, database);

                    match gui.current_tab {
                        TAB_ZONE => {
                            mapview.update_map_zone(gui.current_tab_data as usize);
                            gui.open_zone_settings(systems, mapview);
                        },
                        TAB_PROPERTIES => {
                            gui.editor_selectionbox[0].switch_list(&mut systems.renderer, mapview.fixed_weather as usize);
                        },
                        _ => {},
                    }
                }
            }
        }
        InputType::MouseLeftDownMove => {
            if !is_scrollbar_in_hold(gui) {
                // Check if mouse position is pointing to our map view
                if in_map(screen_pos, mapview)
                    && gameinput.presstype == PressType::PressMap
                {
                    // Calculate the tile position on the map based on mouse position
                    let tile_map_pos = get_map_pos(screen_pos, mapview);

                    gui.labels[LABEL_TILEPOS].set_text(&mut systems.renderer,&format!("Tile [ X: {} Y: {} ]", tile_map_pos.x, tile_map_pos.y), Attrs::new());

                    interact_with_map(
                        systems,
                        tile_map_pos,
                        gui,
                        tileset,
                        mapview,
                        database,
                        gameinput,
                    );

                    mapview.hover_selection_preview(tile_map_pos);
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
                gui.labels[LABEL_TILEPOS].set_text(&mut systems.renderer,&format!("Tile [ X: {} Y: {} ]",tile_map_pos.x, tile_map_pos.y),Attrs::new());
                mapview.hover_selection_preview(tile_map_pos);
            }
        }
        InputType::MouseRelease => {
            mapview.record.stop_record();
        },
    }
}