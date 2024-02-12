use crate::editor_input::*;

// We will handle the dialog input upon the release state of the selected Input
fn dialog_release_input(
    systems: &mut DrawSetting,
    gameinput: &mut GameInput,
    gui: &mut Interface,
    database: &mut EditorData,
    mapview: &mut MapView,
    elwt: &winit::event_loop::EventLoopWindowTarget<()>,
) {
    if !gameinput.dialog_button_press || gui.dialog.is_none() {
        return;
    }

    gameinput.dialog_button_press = false;
    
    if let Some(dialog) = &mut gui.dialog {
        match gameinput.selected_dialog_type {
            DialogButtonType::ButtonConfirm => match &dialog.dialog_type {
                DialogType::TypeExitConfirm => elwt.exit(),
                DialogType::TypeMapLoad => {
                    let (mut x, mut y, mut group) = (0 as i32, 0 as i32, 0 as u64);
                    for (index, textbox) in dialog.editor_textbox.iter().enumerate() {
                        let value = textbox.data.parse::<i64>().unwrap_or_default();
                        match index {
                            1 => {y = value as i32;}
                            2 => {group = value as u64;}
                            _ => {x = value as i32;}
                        }
                    }

                    database.init_map(x, y, group);
                    database.load_map_data(systems, mapview);
                    database.load_link_maps(mapview);
                    update_map_name(systems, gui, database);
                    gui.close_dialog(systems);
                }
                DialogType::TypeMapSave => {
                    database.save_all_maps(mapview);
                    elwt.exit()
                }
                _ => {}
            }
            DialogButtonType::ButtonDecline => match &dialog.dialog_type {
                DialogType::TypeMapSave => elwt.exit(),
                _ => {}
            }
            DialogButtonType::ButtonCancel => gui.close_dialog(systems),
            _ => {}
        }
    }
}

pub fn dialog_input(
    systems: &mut DrawSetting,
    inputtype: &InputType,
    screen_pos: Vec2,
    gameinput: &mut GameInput,
    gui: &mut Interface,
    database: &mut EditorData,
    mapview: &mut MapView,
    elwt: &winit::event_loop::EventLoopWindowTarget<()>,
) {
    if let Some(dialog) = &mut gui.dialog {
        match inputtype {
            InputType::MouseLeftDown => {
                // Check if we are clicking the scrollbar
                if dialog.dialog_type == DialogType::TypeMapSave {
                    if dialog.scrollbar.in_scrollbar(systems, screen_pos) {
                        dialog.scrollbar.hold_scrollbar(systems, screen_pos.y);
                    }
                }

                // Prevent other inputs when we are holding the scrollbar
                if !dialog.scrollbar.in_hold {
                    gameinput.selected_dialog_type =
                        dialog.click_buttons(systems, screen_pos);
                    gameinput.dialog_button_press = true;
                    dialog.select_text(systems, screen_pos);
                }
            }
            InputType::MouseLeftDownMove => {
                if dialog.dialog_type == DialogType::TypeMapSave {
                    dialog.scrollbar.move_scrollbar(systems, screen_pos.y, false);
                    if dialog.update_scroll(dialog.scrollbar.cur_value) {
                        dialog.update_list(systems);
                    }
                    dialog.scrollbar.set_hover(systems, screen_pos);
                }
            }
            InputType::MouseMove => {
                dialog.hover_buttons(systems, screen_pos);
                dialog.scrollbar.set_hover(systems, screen_pos);
            }
            InputType::MouseRelease => {
                dialog.release_click(systems);
                dialog.scrollbar.release_scrollbar(systems);
                if gameinput.dialog_button_press {
                    dialog_release_input(systems,
                                        gameinput, 
                                        gui,
                                        database,
                                        mapview,
                                        elwt,);
                }
            }
        }
    }
}

pub fn dialog_key_input(
    systems: &mut DrawSetting,
    event: &KeyEvent,
    dialog: &mut Dialog,
) {
    if dialog.dialog_type == DialogType::TypeMapLoad {
        if dialog.editing_index < 2 {
            dialog.editor_textbox[dialog.editing_index].enter_numeric(systems, event, 5, true);
        } else {
            dialog.editor_textbox[dialog.editing_index].enter_numeric(systems, event, 5, false);
        }
    }
}