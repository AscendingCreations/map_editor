use crate::editor_input::*;

const GENERAL_HIDEFPS: usize = 0;
const GENERAL_HIDETILEBG: usize = 1;
const GENERAL_HIDEMAPBG: usize = 2;
const GENERAL_MAPCOLOR: usize = 3;
const GENERAL_TILECOLOR: usize = 4;

pub fn preference_input(
    systems: &mut DrawSetting,
    inputtype: &InputType,
    screen_pos: Vec2,
    gui: &mut Interface,
    tileset: &mut Tileset,
    mapview: &mut MapView,
    config_data: &mut ConfigData,
) {
    // If preference is open, cancel all other inputs
    if !gui.preference.is_open {
        return;
    }

    match inputtype {
        InputType::MouseLeftDown => {
            if gui.preference.keywindow.is_open {
                let click_button = gui.preference.keywindow.click_buttons(screen_pos);
                if let Some(index) = click_button {
                    match index {
                        0 => gui.preference.keywindow.close_key(), // Cancel
                        _ => {
                            if let Some(keycode) = &gui.preference.keywindow.key_code {
                                let index = gui.preference.keywindow.key_index;
                                config_data.key_code[index] = keycode.clone();
                                config_data.key_code_modifier[index] = gui.preference.keywindow.key_modifier.clone();
                                gui.preference.update_key_list(systems, index, config_data);
                            }
                            gui.preference.keywindow.close_key()
                        }, // Save
                    }
                }
                return;
            }

            if gui.preference.scrollbar.in_scrollbar(screen_pos) {
                gui.preference.scrollbar.hold_scrollbar(screen_pos.y);
            }

            if !gui.preference.scrollbar.in_hold {
                let click_button = gui.preference.click_buttons(screen_pos);
                if let Some(index) = click_button {
                    match index {
                        0 => {
                            config_data.set_data(load_config());
                            gui.preference.close()
                        }, // Cancel
                        1 => {
                            gui.preference.reset_preference(systems, config_data);
                        }, // Reset
                        _ => {
                            config_data.save_config().unwrap();
                            // Apply settings
                            mapview.selection_preview.set_color(Color::rgba(
                                config_data.map_selection_color[0],
                                config_data.map_selection_color[1],
                                config_data.map_selection_color[2],
                                150,
                            ));
                            tileset.selection.set_color(Color::rgba(
                                config_data.tile_selection_color[0],
                                config_data.tile_selection_color[1],
                                config_data.tile_selection_color[2],
                                150,
                            ));
                            gui.preference.close()
                        }, // Save
                    }
                }

                if gui.preference.select_menu_button(screen_pos) {
                    open_preference_tab(&mut gui.preference, systems, config_data);
                }

                match gui.preference.selected_menu {
                    PREF_TAB_GENERAL => {
                        if let Some(index) = gui.preference.is_coloreditor_open {
                            if gui.preference.in_color_selection(screen_pos) {
                                gui.preference.select_text(screen_pos);
                                if gui.preference.click_color_selection_button(screen_pos) {
                                    if let SettingData::ColorSelection(colorselection) = &mut gui.preference.setting_data[index] {
                                        let data = colorselection.color_editor.data.clone();
                                        colorselection.image.set_color(Color::rgba(data[0], data[1], data[2], data[3]));
                                        match index {
                                            GENERAL_MAPCOLOR => config_data.map_selection_color = data,
                                            GENERAL_TILECOLOR => config_data.tile_selection_color = data,
                                            _ => {}
                                        }
                                    }
                                    gui.preference.hide_color_selection();
                                }
                                return;
                            }
                        }

                        if let Some(config_index) = gui.preference.select_config(screen_pos) {
                            match &mut gui.preference.setting_data[config_index] {
                                SettingData::Checkbox(checkbox) => {
                                    if checkbox.is_select {
                                        checkbox.set_select(false);
                                    } else {
                                        checkbox.set_select(true);
                                    }
                                    match config_index {
                                        GENERAL_HIDEFPS => {
                                            config_data.hide_fps = checkbox.is_select;
                                            gui.labels[LABEL_FPS].changed = true;
                                        },
                                        GENERAL_HIDETILEBG => {
                                            config_data.hide_tileset_bg = checkbox.is_select;
                                            gui.bg_layout[2].changed = true;
                                        },
                                        GENERAL_HIDEMAPBG => {
                                            config_data.hide_mapview_bg = checkbox.is_select;
                                            gui.bg_layout[1].changed = true;
                                        },
                                        _ => {},
                                    }
                                    gui.preference.hide_color_selection();
                                },
                                SettingData::ColorSelection(colorselection) => {
                                    if gui.preference.is_coloreditor_open.is_none() {
                                        colorselection.open_color_editor();
                                        gui.preference.is_coloreditor_open = Some(config_index);
                                    } else {
                                        gui.preference.hide_color_selection();
                                    }
                                },
                                _ => {},
                            }
                        } else {
                            gui.preference.hide_color_selection();
                        }
                    },
                    PREF_TAB_KEYBIND => {
                        if let Some(key_index) = gui.preference.select_keylist(screen_pos) {
                            gui.preference.keywindow.open_key(systems, key_index);
                        }
                    },
                    _ => {},
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
        InputType::MouseRelease => {
            gui.preference.release_click();
            gui.preference.keywindow.release_click();
            gui.preference.scrollbar.release_scrollbar();
        }
    }
}

pub fn preference_key_input(
    renderer: &mut GpuRenderer,
    event: &KeyEvent,
    gui: &mut Interface,
) {
    if !event.state.is_pressed() {
        return;
    }

    match gui.preference.selected_menu {
        PREF_TAB_KEYBIND => {
            if gui.preference.keywindow.is_open {
                gui.preference.keywindow.edit_key(event, renderer);
            }
        },
        PREF_TAB_GENERAL => {
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
        },
        _ => {},
    }
}