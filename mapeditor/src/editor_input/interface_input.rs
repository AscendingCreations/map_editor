use crate::{
    editor_input::*,
    collection::*,
};

pub fn interface_input(
    systems: &mut DrawSetting,
    inputtype: &InputType,
    screen_pos: Vec2,
    gameinput: &mut GameInput,
    gui: &mut Interface,
    tileset: &mut Tileset,
    mapview: &mut MapView,
    database: &mut EditorData,
    config_data: &mut ConfigData,
) {
    match inputtype {
        InputType::MouseLeftDown => {
            if gui.tileset_list.scrollbar.in_scrollbar(screen_pos) {
                gui.tileset_list.scrollbar.hold_scrollbar(screen_pos.y);
            } else if gui.scrollbar.in_scrollbar(screen_pos) {
                gui.scrollbar.hold_scrollbar(screen_pos.y);
            } else if gui.current_tab == TAB_PROPERTIES && gui.selected_dropbox >= 0 {
                if gui.editor_selectionbox[gui.selected_dropbox as usize].scrollbar.in_scrollbar(screen_pos) {
                    gui.editor_selectionbox[gui.selected_dropbox as usize].scrollbar.hold_scrollbar(screen_pos.y);
                }
            }

            if !is_scrollbar_in_hold(gui) {
                // Tools
                let click_button = gui.click_tool_button(screen_pos);
                if let Some(button_index) = click_button {
                    gui_button_select(button_index,
                                    systems,
                                    gameinput,
                                    gui,
                                    tileset,
                                    mapview,
                                    database,
                                    config_data,);
                }

                // Tab Options
                let click_tab_option = gui.click_tab_option(screen_pos);
                if let Some(tab_index) = click_tab_option {
                    gui.select_tab_option(tab_index);
                    // Open
                    match gui.current_tab {
                        TAB_ATTRIBUTE => open_attribute_settings(systems, gui, gui.current_tab_data + 1, vec![]),
                        TAB_ZONE => {
                            mapview.update_map_zone(gui.current_tab_data as usize);
                            gui.open_zone_settings(systems, mapview);
                        },
                        _ => {},
                    }
                }
                
                // Textbox / Buttons
                match gui.current_tab {
                    TAB_ATTRIBUTE | TAB_ZONE => gui.select_textbox(screen_pos),
                    TAB_PROPERTIES => {
                        // Buttons
                        let click_button = gui.click_buttons(screen_pos);
                        if let Some(button_index) = click_button {
                            match button_index {
                                0 => database.save_all_maps(mapview),
                                1 => {
                                    database.reset_all_map();
                                    database.load_map_data(&mut systems.renderer, mapview);
                                    database.load_link_maps(mapview);
                                    update_map_name(systems, gui, database);
                                },
                                2 => {
                                    gui.preference.open();
                                    open_preference_tab(&mut gui.preference, systems, config_data);
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
                                        gui.editor_selectionbox[selection_index].show_list(&mut systems.renderer);
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
                                gui.editor_selectionbox[gui.selected_dropbox as usize].switch_list(&mut systems.renderer, selection_index);

                                match gui.selected_dropbox {
                                    0 => {
                                        mapview.fixed_weather = gui.editor_selectionbox[gui.selected_dropbox as usize].selected_index as u8;
                                        database.set_map_change(mapview);
                                        update_map_name(systems, gui, database);
                                    }
                                    _ => {},
                                }

                                gui.editor_selectionbox[gui.selected_dropbox as usize].hide_list();
                            }
                        }
                    },
                    _ => {},
                }
            }
        }
        InputType::MouseLeftDownMove => {
            if gui.scrollbar.in_hold {
                gui.scrollbar.move_scrollbar(screen_pos.y, false);
                gui.update_scroll(&mut systems.renderer, gui.scrollbar.cur_value);
                gui.scrollbar.set_hover(screen_pos);
            } else if gui.current_tab == TAB_PROPERTIES && gui.selected_dropbox >= 0 {
                if gui.editor_selectionbox[gui.selected_dropbox as usize].scrollbar.in_hold {
                    gui.editor_selectionbox[gui.selected_dropbox as usize].scrollbar.move_scrollbar(screen_pos.y, false);
                    let scrollbar_value = gui.editor_selectionbox[gui.selected_dropbox as usize].scrollbar.cur_value;
                    gui.editor_selectionbox[gui.selected_dropbox as usize].update_list(&mut systems.renderer, scrollbar_value);
                    gui.editor_selectionbox[gui.selected_dropbox as usize].scrollbar.set_hover(screen_pos);
                }
            }
        }
        InputType::MouseMove => {
            gui.hover_tool_button(screen_pos);
            gui.hover_buttons(screen_pos);
            gui.hover_selectionbox(screen_pos);
            gui.hover_tab_option(screen_pos);
            gui.scrollbar.set_hover(screen_pos);
            if gui.current_tab == TAB_PROPERTIES && gui.selected_dropbox >= 0 {
                gui.editor_selectionbox[gui.selected_dropbox as usize].hover_list(screen_pos);
                gui.editor_selectionbox[gui.selected_dropbox as usize].scrollbar.set_hover(screen_pos);
            }
        }
        InputType::MouseRelease => {
            gui.reset_tool_button_click();
            gui.release_click();
            gui.release_selectionbox_click();
            gui.scrollbar.release_scrollbar();
            if gui.current_tab == TAB_PROPERTIES && gui.selected_dropbox >= 0 {
                gui.editor_selectionbox[gui.selected_dropbox as usize].scrollbar.release_scrollbar();
            }
        },
    }
}

pub fn interface_key_input(
    event: &KeyEvent,
    gui: &mut Interface,
    mapview: &mut MapView,
    database: &mut EditorData,
    systems: &mut DrawSetting,
) -> bool {
    let mut result = false;
    match gui.current_tab {
        TAB_ATTRIBUTE => {
            let attribute = MapAttribute::convert_to_plain_enum(gui.current_tab_data + 1);
            match attribute {
                MapAttribute::Warp(_, _, _, _, _) => {
                    if gui.selected_textbox >= 0 {
                        if gui.selected_textbox < 2 {
                            gui.editor_textbox[gui.selected_textbox as usize].enter_numeric(&mut systems.renderer, event, 5, true);
                        } else {
                            gui.editor_textbox[gui.selected_textbox as usize].enter_numeric(&mut systems.renderer, event, 5, false);
                        }
                        result = true;
                    }
                },
                MapAttribute::Sign(_) => {
                    if gui.selected_textbox >= 0 {
                        gui.editor_textbox[gui.selected_textbox as usize].enter_text(&mut systems.renderer, event, 100);
                        result = true;
                    }
                },
                _ => {},
            }
        },
        TAB_ZONE => {
            if gui.selected_textbox >= 0 {
                gui.editor_textbox[gui.selected_textbox as usize].enter_numeric(&mut systems.renderer, event, 5, false);
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
                database.set_map_change(mapview);
                update_map_name(systems, gui, database);
                result = true;
            }
        },
        _ => {},
    }
    result
}

// This function help us switch the map setting tab that the editor is using
pub fn set_tab(systems: &mut DrawSetting, gui: &mut Interface, tab_index: usize, mapview: &mut MapView, tileset: &mut Tileset, gameinput: &mut GameInput) {
    if gui.current_tab != tab_index {
        // Set tab data to default
        for index in 0..MAX_TAB_LABEL {
            gui.tab_labels[index].close(&mut systems.renderer);
            gui.tab_labels[index].set_select(false);
        }
        gui.editor_label = vec![];
        gui.editor_textbox = vec![];
        gui.editor_button = vec![];
        gui.editor_selectionbox = vec![];
        gui.scrollbar.hide();
        gui.current_tab_data = 0;
        gui.current_selected_area = 0;

        // Switch selected tab
        gui.buttons[gui.current_tab].set_state(ButtonState::Normal);
        gui.buttons[tab_index].set_state(ButtonState::Selected);
        gui.current_tab = tab_index;

        // Load tab data
        match gui.current_tab {
            TAB_LAYER => {
                for index in 0..MAX_TAB_LABEL {
                    if index < MapLayers::Count as usize {
                        gui.tab_labels[index].init(&mut systems.renderer, MapLayers::as_str(index as u32), 194.0);
                    }
                }
                gui.tab_labels[0].set_select(true);

                tileset.map.changed = true;
                tileset.selection.changed = true;
                gui.labels[LABEL_TILESET].changed = true;

                mapview.change_selection_preview_size(gameinput.selected_size);
            },
            TAB_ATTRIBUTE => {
                gui.start_view = 0;
                for index in 0..MAX_TAB_LABEL {
                    let sel_index = gui.start_view + index;
                    if sel_index < MAX_ATTRIBUTE as usize - 1 {
                        gui.tab_labels[index].init(&mut systems.renderer, MapAttribute::as_str(sel_index as u32 + 1), 180.0);
                    }
                }
                gui.tab_labels[0].set_select(true);

                mapview.map_attributes.iter_mut().for_each(|attribute| {
                    attribute.text.changed = true;
                    attribute.image.changed = true;
                });

                reset_scrollbar(&mut gui.scrollbar);
                gui.scrollbar_bg.changed = true;
                gui.scrollbar.show();
                gui.scrollbar.images.iter_mut().for_each(|image| {
                    image.changed = true;
                });

                gui.tab_opt_bg[0].changed = true;
                gui.tab_opt_bg[1].changed = true;
                gui.labels[LABEL_OPT_HEADER_TEXT].set_text(&mut systems.renderer, "Attribute Properties", Attrs::new());
                center_text(&mut gui.labels[LABEL_OPT_HEADER_TEXT]);

                mapview.change_selection_preview_size(Vec2::new(1.0, 1.0));
            },
            TAB_ZONE => {
                for index in 0..MAX_TAB_LABEL {
                    if index < 5 {
                        gui.tab_labels[index].init(&mut systems.renderer, &format!("Zone {}", index + 1), 194.0);
                    }
                }
                gui.tab_labels[0].set_select(true);

                mapview.map_zone.iter_mut().for_each(|zone| {
                    zone.changed = true;
                });

                gui.tab_opt_bg[0].changed = true;
                gui.tab_opt_bg[1].changed = true;
                gui.labels[LABEL_OPT_HEADER_TEXT].set_text(&mut systems.renderer, "Zone Settings", Attrs::new());
                center_text(&mut gui.labels[LABEL_OPT_HEADER_TEXT]);

                mapview.update_map_zone(0);

                mapview.change_selection_preview_size(Vec2::new(1.0, 1.0));

                let text_start_pos = Vec2::new(gui.tab_opt_bg[0].position.x, gui.tab_opt_bg[0].position.y);
                for i in 0..7 {
                    let mut ajdust_pos = Vec2::new(text_start_pos.x, text_start_pos.y);
                    let msg: String;
                    match i {
                        1 => {
                            ajdust_pos += Vec2::new(10.0, 338.0);
                            msg = "NPC ID".to_string();
                        },
                        2 | 3 | 4 | 5 | 6=> {
                            ajdust_pos += Vec2::new(10.0, 312.0 - ((i - 2) * 23) as f32);
                            msg = format!("{}", i - 1);
                        },
                        _ => {
                            ajdust_pos += Vec2::new(10.0, 368.0);
                            msg = "Max NPC".to_string();
                        },
                    }
                    let mut text = create_basic_label(systems, 
                        Vec3::new(ajdust_pos.x, ajdust_pos.y, ORDER_ATTRIBUTE_LABEL),
                        Vec2::new(100.0, 20.0),
                        Color::rgba(180, 180, 180, 255));
                    text.set_text(&mut systems.renderer, &msg, Attrs::new());
                    gui.editor_label.push(text);

                    if i != 1 {
                        let add_pos = match i {
                            0 => 85.0,
                            _ => 35.0,
                        };
                        let text_box = Textbox::new(systems,
                            Vec3::new(ajdust_pos.x + add_pos, ajdust_pos.y, ORDER_ATTRIBUTE_TEXTBOX),
                            Vec2::new(60.0, 22.0), false);
                        gui.editor_textbox.push(text_box);
                    }
                }
                gui.editor_textbox[0].input_text(&mut systems.renderer, mapview.map_zone_setting[0].max_npc.to_string()); // Max Npc
                for i in 0..5 {
                    if mapview.map_zone_setting[0].npc_id[i].is_some() {
                        gui.editor_textbox[i + 1].input_text(&mut systems.renderer, mapview.map_zone_setting[0].npc_id[i].unwrap().to_string());
                    }
                }
            },
            TAB_PROPERTIES => {
                gui.tab_opt_bg[0].changed = true;

                gui.editor_button = vec![
                    Button::new(systems, systems.resource.option_button.allocation, "Save All Map",
                            Vec2::new(gui.tab_opt_bg[0].position.x + 14.0, gui.tab_opt_bg[0].position.y + 372.0), Vec2::new(172.0, 36.0),
                            [ORDER_OPTION_BUTTON, ORDER_OPTION_BUTTON_TEXT], 8.0),
                    Button::new(systems, systems.resource.option_button.allocation, "Reset All Map",
                            Vec2::new(gui.tab_opt_bg[0].position.x + 14.0, gui.tab_opt_bg[0].position.y + 332.0), Vec2::new(172.0, 36.0),
                            [ORDER_OPTION_BUTTON, ORDER_OPTION_BUTTON_TEXT], 8.0),
                    Button::new(systems, systems.resource.option_button.allocation, "Preference",
                            Vec2::new(gui.tab_opt_bg[0].position.x + 14.0, gui.tab_opt_bg[0].position.y + 292.0), Vec2::new(172.0, 36.0),
                            [ORDER_OPTION_BUTTON, ORDER_OPTION_BUTTON_TEXT], 8.0),
                ];

                let content_pos = Vec2::new(25.0, 295.0);
                let mut text = create_basic_label(systems, 
                    Vec3::new(content_pos.x, content_pos.y, ORDER_ATTRIBUTE_LABEL),
                    Vec2::new(100.0, 20.0),
                    Color::rgba(180, 180, 180, 255));
                text.set_text(&mut systems.renderer, "Weather", Attrs::new());
                gui.editor_label.push(text);

                let mut selectionbox = SelectionBox::new(systems, 
                    Vec2::new(content_pos.x, content_pos.y - 24.0), 
                    [ORDER_PROPERTIES_BUTTON, 
                                ORDER_PROPERTIES_BUTTON_TEXT,
                                ORDER_DROPDOWN_WINDOW,
                                ORDER_DROPDOWN_SELECTION,
                                ORDER_DROPDOWN_TEXT,
                                ORDER_DROPDOWN_SCROLLBAR], 
                    168.0,
                    vec![
                        "None".to_string(),
                        "Rain".to_string(),
                        "Snow".to_string(),
                    ]);
                selectionbox.switch_list(&mut systems.renderer, mapview.fixed_weather as usize);
                gui.editor_selectionbox.push(selectionbox);
            },
            _ => {},
        }
    }
}

pub fn open_attribute_settings(systems: &mut DrawSetting,
                                gui: &mut Interface,
                                attribute: u32,
                                data: Vec<InsertTypes>)
{
    let attr = MapAttribute::convert_to_plain_enum(attribute);
    // We will make it default that no textbox is selected
    gui.selected_textbox = -1;
    gui.selected_dropbox = -1;
    match attr {
        MapAttribute::Warp(_, _, _, _, _) => {
            gui.editor_label = Vec::with_capacity(7);
            for i in 0..7 {
                let mut ajdust_pos = Vec2::new(gui.tab_opt_bg[0].position.x, gui.tab_opt_bg[0].position.y);
                let msg;
                match i {
                    1 => {
                        ajdust_pos += Vec2::new(45.0, 340.0);
                        msg = "X";
                    },
                    2 => {
                        ajdust_pos += Vec2::new(45.0, 316.0);
                        msg = "Y";
                    },
                    3 => {
                        ajdust_pos += Vec2::new(10.0, 292.0);
                        msg = "Group";
                    },
                    4 => {
                        ajdust_pos += Vec2::new(10.0, 260.0);
                        msg = "Tile Location";
                    },
                    5 => {
                        ajdust_pos += Vec2::new(45.0, 232.0);
                        msg = "X";
                    },
                    6 => {
                        ajdust_pos += Vec2::new(45.0, 208.0);
                        msg = "Y";
                    },
                    _ => {
                        ajdust_pos += Vec2::new(10.0, 368.0);
                        msg = "Map Location";
                    },
                }
                let mut text = create_basic_label(systems, 
                    Vec3::new(ajdust_pos.x, ajdust_pos.y, ORDER_ATTRIBUTE_LABEL),
                    Vec2::new(100.0, 20.0),
                    Color::rgba(180, 180, 180, 255));
                text.set_text(&mut systems.renderer, msg, Attrs::new());
                gui.editor_label.push(text);
            }

            gui.editor_textbox = Vec::with_capacity(5);
            for i in 0..5 {
                let textbox_pos = match i {
                    1 => { Vec3::new(
                        gui.tab_opt_bg[0].position.x + 65.0, 
                        gui.tab_opt_bg[0].position.y + 316.0, ORDER_ATTRIBUTE_TEXTBOX) },
                    2 => { Vec3::new(
                        gui.tab_opt_bg[0].position.x + 65.0, 
                        gui.tab_opt_bg[0].position.y + 292.0, ORDER_ATTRIBUTE_TEXTBOX) },
                    3 => { Vec3::new(
                        gui.tab_opt_bg[0].position.x + 65.0, 
                        gui.tab_opt_bg[0].position.y + 232.0, ORDER_ATTRIBUTE_TEXTBOX) },
                    4 => { Vec3::new(
                        gui.tab_opt_bg[0].position.x + 65.0, 
                        gui.tab_opt_bg[0].position.y + 208.0, ORDER_ATTRIBUTE_TEXTBOX) },
                    _ => { Vec3::new(
                        gui.tab_opt_bg[0].position.x + 65.0, 
                        gui.tab_opt_bg[0].position.y + 340.0, ORDER_ATTRIBUTE_TEXTBOX) },
                };
                gui.editor_textbox.push(Textbox::new(systems,
                        textbox_pos, Vec2::new(60.0, 22.0), false));
            }
            // If data exist, place the data on textbox
            if !data.is_empty() {
                gui.editor_textbox[0].input_text(&mut systems.renderer, data[0].get_int().to_string());
                gui.editor_textbox[1].input_text(&mut systems.renderer, data[1].get_int().to_string());
                gui.editor_textbox[2].input_text(&mut systems.renderer, data[2].get_uint().to_string());
                gui.editor_textbox[3].input_text(&mut systems.renderer, data[3].get_uint().to_string());
                gui.editor_textbox[4].input_text(&mut systems.renderer, data[4].get_uint().to_string());
            } else {
                gui.editor_textbox[0].input_text(&mut systems.renderer, "0".to_string());
                gui.editor_textbox[1].input_text(&mut systems.renderer, "0".to_string());
                gui.editor_textbox[2].input_text(&mut systems.renderer, "0".to_string());
                gui.editor_textbox[3].input_text(&mut systems.renderer, "0".to_string());
                gui.editor_textbox[4].input_text(&mut systems.renderer, "0".to_string());
            }
        }
        MapAttribute::Sign(_data_string) => {
            gui.editor_label = vec![
                                    create_basic_label(systems, 
                                        Vec3::new(gui.tab_opt_bg[0].position.x + 10.0,
                                            gui.tab_opt_bg[0].position.y + 368.0, ORDER_ATTRIBUTE_LABEL),
                                        Vec2::new(100.0, 20.0),
                                        Color::rgba(180, 180, 180, 255)),
            ];
            gui.editor_label[0].set_text(&mut systems.renderer, "Sign Text", Attrs::new());
            gui.editor_textbox = vec![
                Textbox::new(systems,
                    Vec3::new(gui.tab_opt_bg[0].position.x + 10.0, 
                        gui.tab_opt_bg[0].position.y + 115.0, ORDER_ATTRIBUTE_TEXTBOX),
                    Vec2::new(180.0, 250.0), true),
            ];
            // If data exist, place the data on textbox
            if !data.is_empty() {
                gui.editor_textbox[0].input_text(&mut systems.renderer, data[0].get_string());
            } else {
                gui.editor_textbox[0].input_text(&mut systems.renderer, String::new());
            }
        }
        _ => {
            gui.editor_label = vec![];
            gui.editor_textbox = vec![];
        }
    }
}