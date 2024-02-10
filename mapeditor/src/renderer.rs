use graphics::*;
use winit::dpi::PhysicalSize;

use crate::{
    ConfigData, 
    interface::*,
    MapView, 
    TextureAllocation, 
    Tileset,
};

pub struct DrawSetting {
    pub renderer: GpuRenderer,
    pub size: PhysicalSize<f32>,
    pub scale: f64,
    pub resource: TextureAllocation,
}

pub struct Graphics<Controls>
where
    Controls: camera::controls::Controls,
{
    /// World Camera Controls and time. Deturmines how the world is looked at.
    pub system: System<Controls>,
    /// Atlas Groups for Textures in GPU
    pub image_atlas: AtlasSet,
    pub map_atlas: AtlasSet,
    pub text_atlas: TextAtlas,
    pub ui_atlas: AtlasSet,
    /// Rendering Buffers and other shared data.
    pub text_renderer: TextRenderer,
    pub image_renderer: ImageRenderer,
    pub map_renderer: MapRenderer,
    pub ui_renderer: RectRenderer,
}

impl<Controls> Pass for Graphics<Controls>
where
    Controls: camera::controls::Controls,
{
    fn render(
        &mut self,
        renderer: &GpuRenderer,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: renderer.frame_buffer().as_ref().expect("no frame view?"),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.3,
                        g: 0.3,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(
                wgpu::RenderPassDepthStencilAttachment {
                    view: renderer.depth_buffer(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0),
                        store: wgpu::StoreOp::Store,
                    }),
                },
            ),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Lets set the System's Shader information here, mostly Camera, Size and Time
        pass.set_bind_group(0, self.system.bind_group(), &[]);
        // Lets set the Reusable Vertices and Indicies here.
        // This is used for each Renderer, Should be more performant since it is shared.
        pass.set_vertex_buffer(0, renderer.buffer_object.vertices());
        pass.set_index_buffer(
            renderer.buffer_object.indices(),
            wgpu::IndexFormat::Uint32,
        );

        pass.render_map(renderer, &self.map_renderer, &self.map_atlas, 0);
        pass.render_image(renderer,&self.image_renderer, &self.image_atlas, 0);
        pass.render_rects(renderer, &self.ui_renderer, &self.ui_atlas, 0);
        pass.render_text(renderer, &self.text_renderer, &self.text_atlas, 1);

        pass.render_image(renderer,&self.image_renderer, &self.image_atlas, 2);
        pass.render_rects(renderer, &self.ui_renderer, &self.ui_atlas, 2);
        pass.render_text(renderer, &self.text_renderer, &self.text_atlas, 3);

        pass.render_image(renderer,&self.image_renderer, &self.image_atlas, 4);
        pass.render_rects(renderer, &self.ui_renderer, &self.ui_atlas, 4);
        pass.render_text(renderer, &self.text_renderer, &self.text_atlas, 5);
    }
}

pub fn add_image_to_buffer<Controls>(
        systems: &mut DrawSetting, 
        graphics: &mut Graphics<Controls>,
        config_data: &mut ConfigData,
        mapview: &mut MapView,
        gui: &mut Interface,
        tileset: &mut Tileset,
) where
    Controls: camera::controls::Controls, 
{
    // Map View
    mapview.maps.iter_mut().for_each(|map| {
        graphics.map_renderer.map_update(map, &mut systems.renderer, &mut graphics.map_atlas, [0, 0]);
    });
    mapview.link_map_selection.iter_mut().for_each(|image| {
        graphics.ui_renderer.rect_update(image, &mut systems.renderer, &mut graphics.ui_atlas, 0);
    });
    graphics.ui_renderer.rect_update(&mut mapview.selection_preview, &mut systems.renderer, &mut graphics.ui_atlas, 0);
    // GUI
    graphics.image_renderer.image_update(&mut gui.bg_layout[0], &mut systems.renderer, &mut graphics.image_atlas, 0);
    if !config_data.hide_mapview_bg {
        graphics.image_renderer.image_update(&mut gui.bg_layout[1], &mut systems.renderer, &mut graphics.image_atlas, 0);
    }
    if !config_data.hide_tileset_bg {
        graphics.image_renderer.image_update(&mut gui.bg_layout[2], &mut systems.renderer, &mut graphics.image_atlas, 0);
    }
    gui.buttons.iter_mut().for_each(|button| {
        graphics.image_renderer.image_update(&mut button.image, &mut systems.renderer, &mut graphics.image_atlas, 0);
    });
    // Tab labels
    gui.tab_labels.iter_mut().for_each(|tab_label| {
        if tab_label.is_visible {
            graphics.image_renderer.image_update(&mut tab_label.button, &mut systems.renderer, &mut graphics.image_atlas, 0);
            graphics.text_renderer
                .text_update(&mut tab_label.text, &mut graphics.text_atlas, &mut systems.renderer, 1)
                .unwrap();
        }
    });
    // Other Tab settings
    match gui.current_tab {
        TAB_LAYER => {
            graphics.map_renderer.map_update(&mut tileset.map, &mut systems.renderer, &mut graphics.map_atlas, [0, 0]); // Tileset
            graphics.ui_renderer.rect_update(&mut tileset.selection, &mut systems.renderer, &mut graphics.map_atlas, 0); // Tileset Selection
            // Tileset List
            if gui.tileset_list.visible {
                graphics.ui_renderer.rect_update(&mut gui.tileset_list.bg[0], &mut systems.renderer, &mut graphics.ui_atlas, 0);
                graphics.ui_renderer.rect_update(&mut gui.tileset_list.bg[1], &mut systems.renderer, &mut graphics.ui_atlas, 0);
                gui.tileset_list.texts.iter_mut().for_each(|text| {
                    graphics.text_renderer
                                .text_update(text, &mut graphics.text_atlas, &mut systems.renderer, 1)
                                .unwrap();
                });
                gui.tileset_list.selection_buttons.iter_mut().for_each(|button| {
                    graphics.image_renderer.image_update(&mut button.image, &mut systems.renderer, &mut graphics.image_atlas, 0);
                });
                gui.tileset_list.scrollbar.images.iter_mut().for_each(|image| {
                    graphics.image_renderer.image_update(image, &mut systems.renderer, &mut graphics.image_atlas, 0);
                });
            }
        },
        TAB_ATTRIBUTE => {
            graphics.ui_renderer.rect_update(&mut gui.scrollbar_bg, &mut systems.renderer, &mut graphics.ui_atlas, 0);
            gui.scrollbar.images.iter_mut().for_each(|image| {
                graphics.image_renderer.image_update(image, &mut systems.renderer, &mut graphics.image_atlas, 0);
            });
            mapview.map_attributes.iter_mut().for_each(|attribute| {
                graphics.text_renderer
                        .text_update(&mut attribute.text, &mut graphics.text_atlas, &mut systems.renderer, 1)
                        .unwrap();
                graphics.ui_renderer.rect_update(&mut attribute.image, &mut systems.renderer, &mut graphics.ui_atlas, 0);
            });
            graphics.ui_renderer.rect_update(&mut gui.tab_opt_bg[0], &mut systems.renderer, &mut graphics.ui_atlas, 0);
            graphics.ui_renderer.rect_update(&mut gui.tab_opt_bg[1], &mut systems.renderer, &mut graphics.ui_atlas, 0);
            
            // Attribute Properties
            gui.editor_label.iter_mut().for_each(|text| {
                graphics.text_renderer
                                .text_update(text, &mut graphics.text_atlas, &mut systems.renderer, 1)
                                .unwrap();
            });
            gui.editor_textbox.iter_mut().for_each(|textbox| {
                graphics.ui_renderer.rect_update(&mut textbox.image, &mut systems.renderer, &mut graphics.ui_atlas, 0);
                graphics.text_renderer
                                .text_update(&mut textbox.text, &mut graphics.text_atlas, &mut systems.renderer, 1)
                                .unwrap();
            });
        },
        TAB_PROPERTIES => {
            graphics.ui_renderer.rect_update(&mut gui.tab_opt_bg[0], &mut systems.renderer, &mut graphics.ui_atlas, 0);
            gui.editor_button.iter_mut().for_each(|button| {
                graphics.image_renderer.image_update(&mut button.image, &mut systems.renderer, &mut graphics.image_atlas, 0);
                graphics.text_renderer
                    .text_update(&mut button.text, &mut graphics.text_atlas, &mut systems.renderer, 1)
                    .unwrap();
            });

            // Settings
            gui.editor_label.iter_mut().for_each(|text| {
                graphics.text_renderer
                                .text_update(text, &mut graphics.text_atlas, &mut systems.renderer, 1)
                                .unwrap();
            });
            gui.editor_selectionbox.iter_mut().for_each(|selection_box| {
                graphics.ui_renderer.rect_update(&mut selection_box.rect[0], &mut systems.renderer, &mut graphics.ui_atlas, 0);
                if selection_box.is_list_visible {
                    graphics.ui_renderer.rect_update(&mut selection_box.rect[1], &mut systems.renderer, &mut graphics.ui_atlas, 0);
                    selection_box.list_text.iter_mut().for_each(|list_text| {
                        graphics.ui_renderer.rect_update(&mut list_text.rect, &mut systems.renderer, &mut graphics.ui_atlas, 0);
                        graphics.text_renderer
                                    .text_update(&mut list_text.text, &mut graphics.text_atlas, &mut systems.renderer, 1)
                                    .unwrap();
                    });
                    if selection_box.scrollbar.visible {
                        selection_box.scrollbar.images.iter_mut().for_each(|image| {
                            graphics.image_renderer.image_update(image, &mut systems.renderer, &mut graphics.image_atlas, 0);
                        });
                    }
                }
                graphics.image_renderer.image_update(&mut selection_box.button, &mut systems.renderer, &mut graphics.image_atlas, 0);
                graphics.text_renderer
                                .text_update(&mut selection_box.text, &mut graphics.text_atlas, &mut systems.renderer, 1)
                                .unwrap();
            });
        },
        TAB_ZONE => {
            mapview.map_zone.iter_mut().for_each(|zone| {
                graphics.ui_renderer.rect_update(zone, &mut systems.renderer, &mut graphics.ui_atlas, 0);
            });
            graphics.ui_renderer.rect_update(&mut gui.tab_opt_bg[0], &mut systems.renderer, &mut graphics.ui_atlas, 0);
            graphics.ui_renderer.rect_update(&mut gui.tab_opt_bg[1], &mut systems.renderer, &mut graphics.ui_atlas, 0);
        
            // Zone Properties
            gui.editor_label.iter_mut().for_each(|text| {
                graphics.text_renderer
                                .text_update(text, &mut graphics.text_atlas, &mut systems.renderer, 1)
                                .unwrap();
            });
            gui.editor_textbox.iter_mut().for_each(|textbox| {
                graphics.ui_renderer.rect_update(&mut textbox.image, &mut systems.renderer, &mut graphics.ui_atlas, 0);
                graphics.text_renderer
                                .text_update(&mut textbox.text, &mut graphics.text_atlas, &mut systems.renderer, 1)
                                .unwrap();
            });
        },
        _ => {},
    }
    // Labels
    for index in 0..MAX_LABEL {
        let mut can_render = true;
        match index {
            LABEL_OPT_HEADER_TEXT => {
                if gui.current_tab == TAB_LAYER || 
                    gui.current_tab == TAB_PROPERTIES { can_render = false; }
            },
            LABEL_TILESET => {
                if gui.current_tab != TAB_LAYER { can_render = false }
            },
            LABEL_FPS => {
                if config_data.hide_fps { can_render = false }
            },
            _ => {},
        }
        if can_render {
            graphics.text_renderer
                .text_update(&mut gui.labels[index], &mut graphics.text_atlas, &mut systems.renderer, 1)
                .unwrap();
        }
    }

    // Dialog
    if let Some(dialog) = &mut gui.dialog {
        graphics.ui_renderer.rect_update(&mut dialog.bg, &mut systems.renderer, &mut graphics.ui_atlas, 2);
        graphics.ui_renderer.rect_update(&mut dialog.window, &mut systems.renderer, &mut graphics.ui_atlas, 2);
        graphics.text_renderer
            .text_update(&mut dialog.message, &mut graphics.text_atlas, &mut systems.renderer, 3)
            .unwrap();
        dialog.buttons.iter_mut().for_each(|dialogbutton| {
            graphics.image_renderer.image_update(&mut dialogbutton.image, &mut systems.renderer, &mut graphics.image_atlas, 2);
            graphics.text_renderer
                .text_update(&mut dialogbutton.text, &mut graphics.text_atlas, &mut systems.renderer, 3)
                .unwrap();
        });
        dialog.content_image.iter_mut().for_each(|rect| {
            graphics.ui_renderer.rect_update(rect, &mut systems.renderer, &mut graphics.ui_atlas, 2);
        });
        dialog.content_text.iter_mut().for_each(|text| {
            graphics.text_renderer
                        .text_update(text, &mut graphics.text_atlas, &mut systems.renderer, 3)
                        .unwrap();
        });
        dialog.editor_textbox.iter_mut().for_each(|textbox| {
            graphics.ui_renderer.rect_update(&mut textbox.image, &mut systems.renderer, &mut graphics.ui_atlas, 2);
            graphics.text_renderer
                        .text_update(&mut textbox.text, &mut graphics.text_atlas, &mut systems.renderer, 3)
                        .unwrap();
        });
        if dialog.dialog_type == DialogType::TypeMapSave {
            dialog.scrollbar.images.iter_mut().for_each(|image| {
                graphics.image_renderer.image_update(image, &mut systems.renderer, &mut graphics.image_atlas, 2);
            });
        }
    }
    // Preference
    if gui.preference.is_open {
        graphics.ui_renderer.rect_update(&mut gui.preference.bg, &mut systems.renderer, &mut graphics.ui_atlas, 2);
        gui.preference.window.iter_mut().for_each(|window| {
            graphics.ui_renderer.rect_update(window, &mut systems.renderer, &mut graphics.ui_atlas, 2);
        });
        gui.preference.buttons.iter_mut().for_each(|button| {
            graphics.image_renderer.image_update(&mut button.image, &mut systems.renderer, &mut graphics.image_atlas, 2);
            graphics.text_renderer
                .text_update(&mut button.text, &mut graphics.text_atlas, &mut systems.renderer, 3)
                .unwrap();
        });
        gui.preference.menu_button.iter_mut().for_each(|button| {
            graphics.ui_renderer.rect_update(&mut button.image, &mut systems.renderer, &mut graphics.ui_atlas, 2);
            graphics.text_renderer
                .text_update(&mut button.text, &mut graphics.text_atlas, &mut systems.renderer, 3)
                .unwrap();
        });
        gui.preference.scrollbar.images.iter_mut().for_each(|image| {
            graphics.image_renderer.image_update(image, &mut systems.renderer, &mut graphics.image_atlas, 2);
        });
        match gui.preference.selected_menu {
            PREF_TAB_GENERAL => {
                gui.preference.setting_data.iter_mut().for_each(|setting| {
                    match setting {
                        SettingData::Checkbox(checkbox) => {
                            checkbox.window.iter_mut().for_each(|rect| {
                                graphics.ui_renderer.rect_update(rect, &mut systems.renderer, &mut graphics.ui_atlas, 2);
                            });
                            graphics.text_renderer
                                .text_update(&mut checkbox.text, &mut graphics.text_atlas, &mut systems.renderer, 3)
                                .unwrap();
                        },
                        SettingData::ColorSelection(colorselection) => {
                            graphics.ui_renderer.rect_update(&mut colorselection.image, &mut systems.renderer, &mut graphics.ui_atlas, 2);
                            graphics.text_renderer
                                .text_update(&mut colorselection.text, &mut graphics.text_atlas, &mut systems.renderer, 3)
                                .unwrap();
                            if colorselection.color_editor.is_open {
                                graphics.ui_renderer.rect_update(&mut colorselection.color_editor.window, &mut systems.renderer, &mut graphics.ui_atlas, 2);
                                colorselection.color_editor.label.iter_mut().for_each(|label| {
                                    graphics.text_renderer
                                        .text_update(label, &mut graphics.text_atlas, &mut systems.renderer, 3)
                                        .unwrap();
                                });
                                colorselection.color_editor.textbox.iter_mut().for_each(|textbox| {
                                    graphics.ui_renderer.rect_update(&mut textbox.image, &mut systems.renderer, &mut graphics.ui_atlas, 2);
                                    graphics.text_renderer
                                                .text_update(&mut textbox.text, &mut graphics.text_atlas, &mut systems.renderer, 3)
                                                .unwrap();
                                });
                                graphics.image_renderer.image_update(&mut colorselection.color_editor.button.image, &mut systems.renderer, &mut graphics.image_atlas, 2);
                                graphics.text_renderer
                                    .text_update(&mut colorselection.color_editor.button.text, &mut graphics.text_atlas, &mut systems.renderer, 3)
                                    .unwrap();
                            }
                        },
                        _ => {},
                    }
                })
            },
            PREF_TAB_KEYBIND => {
                gui.preference.key_list.iter_mut().for_each(|keylist| {
                    graphics.text_renderer
                        .text_update(&mut keylist.text, &mut graphics.text_atlas, &mut systems.renderer, 3)
                        .unwrap();
                    graphics.text_renderer
                        .text_update(&mut keylist.key_string, &mut graphics.text_atlas, &mut systems.renderer, 3)
                        .unwrap();
                    graphics.ui_renderer.rect_update(&mut keylist.key_button, &mut systems.renderer, &mut graphics.ui_atlas, 2);
                });

                if gui.preference.keywindow.is_open {
                    graphics.ui_renderer.rect_update(&mut gui.preference.keywindow.window, &mut systems.renderer, &mut graphics.ui_atlas, 4);
                    graphics.text_renderer
                        .text_update(&mut gui.preference.keywindow.text, &mut graphics.text_atlas, &mut systems.renderer, 5)
                        .unwrap();
                    gui.preference.keywindow.buttons.iter_mut().for_each(|button| {
                        graphics.image_renderer.image_update(&mut button.image, &mut systems.renderer, &mut graphics.image_atlas, 4);
                        graphics.text_renderer
                            .text_update(&mut button.text, &mut graphics.text_atlas, &mut systems.renderer, 5)
                            .unwrap();
                    });
                }
            },
            _ => {},
        }
    }
}