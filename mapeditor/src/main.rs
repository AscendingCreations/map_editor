#![allow(dead_code, clippy::collapsible_match, unused_imports)]
use backtrace::Backtrace;
use camera::{
    controls::{Controls, FlatControls, FlatSettings},
    Projection,
};
use cosmic_text::{Attrs, Metrics};
use graphics::*;
use input::{Bindings, FrameTime, InputHandler, Key};
use log::{error, info, warn, Level, LevelFilter, Metadata, Record};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{prelude::*, Read, Write},
    iter, panic,
    sync::Arc,
    time::{Duration, Instant},
};
use wgpu::{Backends, Dx12Compiler, InstanceDescriptor, InstanceFlags};
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, WindowButtons},
};

mod renderer;
mod interface;
mod resource;
mod collection;
mod tileset;
mod game_input;
mod map;
mod map_data;
mod gfx_order;

use renderer::*;
use interface::*;
use resource::*;
use collection::*;
use tileset::*;
use game_input::*;
use map::*;
use map_data::*;
use gfx_order::*;

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
enum Axis {
    Forward,
    Sideward,
    Yaw,
    Pitch,
}

pub struct Content {
    pub gui: Interface,
    pub tileset: Tileset,
}

// creates a static global logger type for setting the logger
static MY_LOGGER: MyLogger = MyLogger(Level::Debug);

struct MyLogger(pub Level);

impl log::Log for MyLogger {
    // checks if it can log these types of events.
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.0
    }

    // This logs to a panic file. This is so we can see
    // Errors and such if a program crashes in full render mode.
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let msg = format!("{} - {}\n", record.level(), record.args());
            println!("{}", &msg);

            let mut file = match File::options()
                .append(true)
                .create(true)
                .open("paniclog.txt")
            {
                Ok(v) => v,
                Err(_) => return,
            };

            let _ = file.write(msg.as_bytes());
        }
    }
    fn flush(&self) {}
}

#[tokio::main]
async fn main() -> Result<(), AscendingError> {
    // Create logger to output to a File
    log::set_logger(&MY_LOGGER).unwrap();
    // Set the Max level we accept logging to the file for.
    log::set_max_level(LevelFilter::Info);

    // This allows us to take control of panic!() so we can send it to a file via the logger.
    panic::set_hook(Box::new(|panic_info| {
        let bt = Backtrace::new();

        error!("PANIC: {}, BACKTRACE: {:?}", panic_info, bt);
    }));

    // Create the directory for our map data
    fs::create_dir_all("./data/maps/")?;

    // Starts an event gathering type for the window.
    let event_loop = EventLoop::new()?;

    // Builds the Windows that will be rendered too.
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Map Editor")
            .with_inner_size(PhysicalSize::new((949.0 * ZOOM_LEVEL) as u32, 
                                                (802.0 * ZOOM_LEVEL) as u32))
            .with_visible(false)
            .with_enabled_buttons({
                let mut buttons = WindowButtons::all();
                buttons.remove(WindowButtons::MAXIMIZE);
                buttons
            })
            .with_resizable(false)
            .build(&event_loop)
            .unwrap(),
    );

    // Generates an Instance for WGPU. Sets WGPU to be allowed on all possible supported backends
    // These are DX12, DX11, Vulkan, Metal and Gles. if none of these work on a system they cant
    // play the game basically.
    let instance = wgpu::Instance::new(InstanceDescriptor {
        backends: Backends::all(),
        flags: InstanceFlags::default(),
        dx12_shader_compiler: Dx12Compiler::default(),
        gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
    });

    // This is used to ensure the GPU can load the correct.
    let compatible_surface = instance.create_surface(window.clone()).unwrap();
    
    // This creates the Window Struct and Device struct that holds all the rendering information
    // we need to render to the screen. Window holds most of the window information including
    // the surface type. device includes the queue and GPU device for rendering.
    // This then adds gpu_window and gpu_device and creates our renderer type. for easy passing of window, device and font system.
    let mut renderer = instance
        .create_device(
            window,
            &wgpu::RequestAdapterOptions {
                // High performance mode says to use Dedicated Graphics devices first.
                // Low power is APU graphic devices First.
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&compatible_surface),
                // we will never use this as this forces us to use an alternative renderer.
                force_fallback_adapter: false,
            },
            // used to deturmine if we need special limits or features for our backends.
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::default(),
                required_limits: wgpu::Limits::default(),
                label: None,
            },
            None,
            // How we are presenting the screen which causes it to either clip to a FPS limit or be unlimited.
            wgpu::PresentMode::AutoVsync,
        )
        .await
        .unwrap();

    // get the screen size.
    let size = renderer.size();

    // get the Scale factor the pc currently is using for upscaling or downscaling the rendering.
    let scale = renderer.window().current_monitor().unwrap().scale_factor();

    // We generate Texture atlases to use with out types.
    let mut atlases: Vec<AtlasSet> = iter::from_fn(|| {
        Some(AtlasSet::new(
            &mut renderer,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            true,
        ))
    })
    .take(4)
    .collect();

    // we generate the Text atlas seperatly since it contains a special texture that only has the red color to it.
    // and another for emojicons.
    let text_atlas = TextAtlas::new(&mut renderer).unwrap();

    // Load textures image
    let resource = TextureAllocation::new(&mut atlases, &renderer)?;

    // Compile all rendering data in one type for quick access and passing
    let mut draw_setting = DrawSetting {
        renderer,
        size,
        scale,
        resource,
    };

    // Initiate map editor data
    let mut gui = Interface::new(&mut draw_setting);
    let mut tileset = Tileset::new(&mut draw_setting);
    let mut gameinput = GameInput::new();
    let mut mapview = MapView::new(&mut draw_setting);
    let mut editor_data = EditorData::new()?;

    // Load the initial map
    editor_data.load_map_data(&mut draw_setting.renderer, &mut mapview);
    editor_data.load_link_maps(&mut mapview);

    // setup our system which includes Camera and projection as well as our controls.
    // for the camera.
    let system = System::new(
        &mut draw_setting.renderer,
        Projection::Orthographic {
            left: 0.0,
            right: size.width,
            bottom: 0.0,
            top: size.height,
            near: 1.0,
            far: -100.0,
        },
        FlatControls::new(FlatSettings { zoom: ZOOM_LEVEL }),
        [size.width, size.height],
    );

    // We establish the different renderers here to load their data up to use them.
    let text_renderer = TextRenderer::new(&draw_setting.renderer).unwrap();
    let image_renderer = ImageRenderer::new(&draw_setting.renderer).unwrap();
    let map_renderer = MapRenderer::new(&mut draw_setting.renderer, 81).unwrap();
    let ui_renderer = RectRenderer::new(&mut draw_setting.renderer).unwrap();

    // Allow the window to be seen. hiding it then making visible speeds up
    // load times.
    draw_setting.renderer.window().set_visible(true);

    // add everything into our convience type for quicker access and passing.
    let mut graphics = Graphics {
        system,
        image_atlas: atlases.remove(0),
        map_renderer,
        map_atlas: atlases.remove(0),
        image_renderer,
        text_atlas,
        text_renderer,
        ui_renderer,
        ui_atlas: atlases.remove(0),
    };

    // Create the mouse/keyboard bindings for our stuff.
    let bindings = Bindings::<Action, Axis>::new();

    // set bindings and create our own input handler.
    let mut input_handler = InputHandler::new(bindings);

    let mut frame_time = FrameTime::new();
    let mut time = 0.0f32;
    let mut fps = 0u32;

    // This will prevent key press to trigger the action while holding down the key
    let mut did_key_press = [false; ACTION_SIZE];

    #[allow(deprecated)]
    event_loop.run(move |event, elwt| {
        // we check for the first batch of events to ensure we dont need to stop rendering here first.
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
                ..
            } if window_id == draw_setting.renderer.window().id() => {
                match event {
                    WindowEvent::CloseRequested => {
                        // Close preference window
                        if gui.preference.is_open {
                            if gui.preference.keywindow.is_open {
                                gui.preference.keywindow.close_key();
                            }
                            gui.preference.config_data = match load_config() {
                                Ok(data) => data,
                                Err(_) => KeybindData::default(),
                            };
                            gui.preference.close();
                        }
                        if editor_data.got_changes() {
                            // We found changes on our map, we need to confirm if we would like to proceed to exit the editor
                            gui.open_dialog(&mut draw_setting, DialogType::TypeMapSave, Some(editor_data.did_map_change.clone()));
                        } else {
                            gui.open_dialog(&mut draw_setting, DialogType::TypeExitConfirm, None);
                        }
                    }
                    WindowEvent::KeyboardInput { event, .. } => {
                        if !handle_key_input(&mut draw_setting.renderer,
                                    event,
                                    &mut gui,
                                    &mut mapview) {
                            // Make sure that we only trigger the shortcut keys when we are not on a textbox
                            handle_shortcut(event,
                                    &mut draw_setting,
                                    &mut gameinput,
                                    &mut editor_data,
                                    &mut mapview,
                                    &mut gui);
                        };
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => { draw_setting.renderer.window().request_redraw(); }
            _ => {}
        }

        // get the current window size so we can see if we need to resize the renderer.
        let new_size = draw_setting.renderer.size();
        let inner_size = draw_setting.renderer.window().inner_size();

        // if our rendering size is zero stop rendering to avoid errors.
        if new_size.width == 0.0
            || new_size.height == 0.0
            || inner_size.width == 0
            || inner_size.height == 0
        { return; }

        // update our inputs.
        input_handler.update(draw_setting.renderer.window(), &event, 1.0);

        // update our renderer based on events here
        if !draw_setting.renderer.update(&event).unwrap() { return; }

        if draw_setting.size != new_size {
            draw_setting.size = new_size;

            // Reset screen size for the Surface here.
            graphics.system.set_projection(Projection::Orthographic {
                left: 0.0,
                right: new_size.width,
                bottom: 0.0,
                top: new_size.height,
                near: 1.0,
                far: -100.0,
            });

            draw_setting.renderer.update_depth_texture();
        }

        // check if out close action was hit for esc
        //if input_handler.is_action_down(&Action::Quit) { elwt.exit(); }

        let mouse_pos_result = input_handler.mouse_position();
        let mouse_pos =if mouse_pos_result.is_none() { (0.0, 0.0) } else { mouse_pos_result.unwrap() };

        if input_handler.is_mouse_button_down(MouseButton::Left) {
            if !did_key_press[action_index(Action::Select)] {
                did_key_press[action_index(Action::Select)] = true;

                gameinput.last_mouse_pos = mouse_pos.clone();

                handle_input(&mut draw_setting, InputType::MouseLeftDown, 
                    &Vec2::new(mouse_pos.0, mouse_pos.1),
                    &mut gameinput,
                    &mut gui, 
                    &mut tileset,
                    &mut mapview,
                    &mut editor_data);
            } else {
                if gameinput.last_mouse_pos != mouse_pos {
                    gameinput.last_mouse_pos = mouse_pos.clone();
                    
                    handle_input(&mut draw_setting, InputType::MouseLeftDownMove, 
                        &Vec2::new(mouse_pos.0, mouse_pos.1),
                        &mut gameinput,
                        &mut gui,
                        &mut tileset,
                        &mut mapview,
                        &mut editor_data);
                }
            }
        } else {
            if gameinput.last_mouse_pos != mouse_pos {
                gameinput.last_mouse_pos = mouse_pos.clone();
                
                handle_input(&mut draw_setting, InputType::MouseMove, 
                    &Vec2::new(mouse_pos.0, mouse_pos.1),
                    &mut gameinput,
                    &mut gui,
                    &mut tileset,
                    &mut mapview,
                    &mut editor_data);
            }
            mapview.record.stop_record();
            gui.reset_tool_button_click();
            gui.release_click();
            gui.preference.release_click();
            gui.preference.scrollbar.release_scrollbar();
            if let Some(dialog) = &mut gui.dialog {
                dialog.release_click();
                dialog.scrollbar.release_scrollbar();
            }
            if gui.preference.keywindow.is_open {
                gui.preference.keywindow.release_click();
            }
            if gameinput.dialog_button_press {
                handle_dialog_input(&mut draw_setting,
                                    &mut gameinput, 
                                    &mut gui,
                                    &mut editor_data,
                                    &mut mapview,
                                    elwt,);
            }
            gui.tileset_list.scrollbar.release_scrollbar();
            gui.scrollbar.release_scrollbar();
            did_key_press[action_index(Action::Select)] = false;
        }

        let seconds = frame_time.seconds();
        // update our systems data to the gpu. this is the Camera in the shaders.
        graphics.system.update(&draw_setting.renderer, &frame_time);

        // update our systems data to the gpu. this is the Screen in the shaders.
        graphics.system.update_screen(&draw_setting.renderer, [new_size.width, new_size.height]);

        // This adds the Image data to the Buffer for rendering.
        // Map View
        mapview.maps.iter_mut().for_each(|map| {
            graphics.map_renderer.map_update(map, &mut draw_setting.renderer, &mut graphics.map_atlas, [0, 0]);
        });
        mapview.link_map_selection.iter_mut().for_each(|image| {
            graphics.ui_renderer.rect_update(image, &mut draw_setting.renderer, &mut graphics.ui_atlas, 0);
        });
        graphics.ui_renderer.rect_update(&mut mapview.selection_preview, &mut draw_setting.renderer, &mut graphics.ui_atlas, 0);
        // GUI
        graphics.image_renderer.image_update(&mut gui.bg_layout, &mut draw_setting.renderer, &mut graphics.image_atlas, 0);
        gui.buttons.iter_mut().for_each(|button| {
            graphics.image_renderer.image_update(&mut button.image, &mut draw_setting.renderer, &mut graphics.image_atlas, 0);
        });
        // Tab labels
        gui.tab_labels.iter_mut().for_each(|tab_label| {
            if tab_label.is_visible {
                graphics.image_renderer.image_update(&mut tab_label.button, &mut draw_setting.renderer, &mut graphics.image_atlas, 0);
                graphics.text_renderer
                    .text_update(&mut tab_label.text, &mut graphics.text_atlas, &mut draw_setting.renderer, 1)
                    .unwrap();
            }
        });
        // Other Tab settings
        match gui.current_setting_tab {
            TAB_LAYER => {
                graphics.map_renderer.map_update(&mut tileset.map, &mut draw_setting.renderer, &mut graphics.map_atlas, [0, 0]); // Tileset
                graphics.ui_renderer.rect_update(&mut tileset.selection, &mut draw_setting.renderer, &mut graphics.map_atlas, 0); // Tileset Selection
                // Tileset List
                if gui.tileset_list.visible {
                    graphics.ui_renderer.rect_update(&mut gui.tileset_list.bg[0], &mut draw_setting.renderer, &mut graphics.ui_atlas, 0);
                    graphics.ui_renderer.rect_update(&mut gui.tileset_list.bg[1], &mut draw_setting.renderer, &mut graphics.ui_atlas, 0);
                    gui.tileset_list.texts.iter_mut().for_each(|text| {
                        graphics.text_renderer
                                    .text_update(text, &mut graphics.text_atlas, &mut draw_setting.renderer, 1)
                                    .unwrap();
                    });
                    gui.tileset_list.selection_buttons.iter_mut().for_each(|button| {
                        graphics.image_renderer.image_update(&mut button.image, &mut draw_setting.renderer, &mut graphics.image_atlas, 0);
                    });
                    gui.tileset_list.scrollbar.images.iter_mut().for_each(|image| {
                        graphics.image_renderer.image_update(image, &mut draw_setting.renderer, &mut graphics.image_atlas, 0);
                    });
                }
            },
            TAB_ATTRIBUTE => {
                graphics.ui_renderer.rect_update(&mut gui.scrollbar_bg, &mut draw_setting.renderer, &mut graphics.ui_atlas, 0);
                gui.scrollbar.images.iter_mut().for_each(|image| {
                    graphics.image_renderer.image_update(image, &mut draw_setting.renderer, &mut graphics.image_atlas, 0);
                });
                mapview.map_attributes.iter_mut().for_each(|attribute| {
                    graphics.text_renderer
                            .text_update(&mut attribute.text, &mut graphics.text_atlas, &mut draw_setting.renderer, 1)
                            .unwrap();
                    graphics.ui_renderer.rect_update(&mut attribute.image, &mut draw_setting.renderer, &mut graphics.ui_atlas, 0);
                });
                graphics.ui_renderer.rect_update(&mut gui.tab_opt_bg[0], &mut draw_setting.renderer, &mut graphics.ui_atlas, 0);
                graphics.ui_renderer.rect_update(&mut gui.tab_opt_bg[1], &mut draw_setting.renderer, &mut graphics.ui_atlas, 0);
                
                // Attribute Properties
                gui.editor_label.iter_mut().for_each(|text| {
                    graphics.text_renderer
                                    .text_update(text, &mut graphics.text_atlas, &mut draw_setting.renderer, 1)
                                    .unwrap();
                });
                gui.editor_textbox.iter_mut().for_each(|textbox| {
                    graphics.ui_renderer.rect_update(&mut textbox.image, &mut draw_setting.renderer, &mut graphics.ui_atlas, 0);
                    graphics.text_renderer
                                    .text_update(&mut textbox.text, &mut graphics.text_atlas, &mut draw_setting.renderer, 1)
                                    .unwrap();
                });
            },
            TAB_PROPERTIES => {
                graphics.ui_renderer.rect_update(&mut gui.tab_opt_bg[0], &mut draw_setting.renderer, &mut graphics.ui_atlas, 0);
                gui.editor_button.iter_mut().for_each(|button| {
                    graphics.image_renderer.image_update(&mut button.image, &mut draw_setting.renderer, &mut graphics.image_atlas, 0);
                    graphics.text_renderer
                        .text_update(&mut button.text, &mut graphics.text_atlas, &mut draw_setting.renderer, 1)
                        .unwrap();
                });
            },
            TAB_ZONE => {
                mapview.map_zone.iter_mut().for_each(|zone| {
                    graphics.ui_renderer.rect_update(zone, &mut draw_setting.renderer, &mut graphics.ui_atlas, 0);
                });
                graphics.ui_renderer.rect_update(&mut gui.tab_opt_bg[0], &mut draw_setting.renderer, &mut graphics.ui_atlas, 0);
                graphics.ui_renderer.rect_update(&mut gui.tab_opt_bg[1], &mut draw_setting.renderer, &mut graphics.ui_atlas, 0);
            
                // Zone Properties
                gui.editor_label.iter_mut().for_each(|text| {
                    graphics.text_renderer
                                    .text_update(text, &mut graphics.text_atlas, &mut draw_setting.renderer, 1)
                                    .unwrap();
                });
                gui.editor_textbox.iter_mut().for_each(|textbox| {
                    graphics.ui_renderer.rect_update(&mut textbox.image, &mut draw_setting.renderer, &mut graphics.ui_atlas, 0);
                    graphics.text_renderer
                                    .text_update(&mut textbox.text, &mut graphics.text_atlas, &mut draw_setting.renderer, 1)
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
                    if gui.current_setting_tab == TAB_LAYER || 
                        gui.current_setting_tab == TAB_PROPERTIES { can_render = false; }
                },
                LABEL_TILESET => {
                    if gui.current_setting_tab != TAB_LAYER { can_render = false }
                },
                _ => {},
            }
            if can_render {
                graphics.text_renderer
                    .text_update(&mut gui.labels[index], &mut graphics.text_atlas, &mut draw_setting.renderer, 1)
                    .unwrap();
            }
        }

        // Dialog
        if let Some(dialog) = &mut gui.dialog {
            graphics.ui_renderer.rect_update(&mut dialog.bg, &mut draw_setting.renderer, &mut graphics.ui_atlas, 2);
            graphics.ui_renderer.rect_update(&mut dialog.window, &mut draw_setting.renderer, &mut graphics.ui_atlas, 2);
            graphics.text_renderer
                .text_update(&mut dialog.message, &mut graphics.text_atlas, &mut draw_setting.renderer, 3)
                .unwrap();
            dialog.buttons.iter_mut().for_each(|dialogbutton| {
                graphics.image_renderer.image_update(&mut dialogbutton.image, &mut draw_setting.renderer, &mut graphics.image_atlas, 2);
                graphics.text_renderer
                    .text_update(&mut dialogbutton.text, &mut graphics.text_atlas, &mut draw_setting.renderer, 3)
                    .unwrap();
            });
            dialog.content_image.iter_mut().for_each(|rect| {
                graphics.ui_renderer.rect_update(rect, &mut draw_setting.renderer, &mut graphics.ui_atlas, 2);
            });
            dialog.content_text.iter_mut().for_each(|text| {
                graphics.text_renderer
                            .text_update(text, &mut graphics.text_atlas, &mut draw_setting.renderer, 3)
                            .unwrap();
            });
            dialog.editor_textbox.iter_mut().for_each(|textbox| {
                graphics.ui_renderer.rect_update(&mut textbox.image, &mut draw_setting.renderer, &mut graphics.ui_atlas, 2);
                graphics.text_renderer
                            .text_update(&mut textbox.text, &mut graphics.text_atlas, &mut draw_setting.renderer, 3)
                            .unwrap();
            });
            if dialog.dialog_type == DialogType::TypeMapSave {
                dialog.scrollbar.images.iter_mut().for_each(|image| {
                    graphics.image_renderer.image_update(image, &mut draw_setting.renderer, &mut graphics.image_atlas, 2);
                });
            }
        }

        // Preference
        if gui.preference.is_open {
            graphics.ui_renderer.rect_update(&mut gui.preference.bg, &mut draw_setting.renderer, &mut graphics.ui_atlas, 2);
            gui.preference.window.iter_mut().for_each(|window| {
                graphics.ui_renderer.rect_update(window, &mut draw_setting.renderer, &mut graphics.ui_atlas, 2);
            });
            gui.preference.buttons.iter_mut().for_each(|button| {
                graphics.image_renderer.image_update(&mut button.image, &mut draw_setting.renderer, &mut graphics.image_atlas, 2);
                graphics.text_renderer
                    .text_update(&mut button.text, &mut graphics.text_atlas, &mut draw_setting.renderer, 3)
                    .unwrap();
            });
            gui.preference.menu_button.iter_mut().for_each(|button| {
                graphics.ui_renderer.rect_update(&mut button.image, &mut draw_setting.renderer, &mut graphics.ui_atlas, 2);
                graphics.text_renderer
                    .text_update(&mut button.text, &mut graphics.text_atlas, &mut draw_setting.renderer, 3)
                    .unwrap();
            });
            gui.preference.scrollbar.images.iter_mut().for_each(|image| {
                graphics.image_renderer.image_update(image, &mut draw_setting.renderer, &mut graphics.image_atlas, 2);
            });
            match gui.preference.selected_menu {
                PREF_TAB_GENERAL => {
                    gui.preference.setting_data.iter_mut().for_each(|setting| {
                        match setting {
                            SettingData::Checkbox(checkbox) => {
                                checkbox.window.iter_mut().for_each(|rect| {
                                    graphics.ui_renderer.rect_update(rect, &mut draw_setting.renderer, &mut graphics.ui_atlas, 2);
                                });
                                graphics.text_renderer
                                    .text_update(&mut checkbox.text, &mut graphics.text_atlas, &mut draw_setting.renderer, 3)
                                    .unwrap();
                            }
                            _ => {},
                        }
                    })
                },
                PREF_TAB_KEYBIND => {
                    gui.preference.key_list.iter_mut().for_each(|keylist| {
                        graphics.text_renderer
                            .text_update(&mut keylist.text, &mut graphics.text_atlas, &mut draw_setting.renderer, 3)
                            .unwrap();
                        graphics.text_renderer
                            .text_update(&mut keylist.key_string, &mut graphics.text_atlas, &mut draw_setting.renderer, 3)
                            .unwrap();
                        graphics.ui_renderer.rect_update(&mut keylist.key_button, &mut draw_setting.renderer, &mut graphics.ui_atlas, 2);
                    });

                    if gui.preference.keywindow.is_open {
                        graphics.ui_renderer.rect_update(&mut gui.preference.keywindow.window, &mut draw_setting.renderer, &mut graphics.ui_atlas, 4);
                        graphics.text_renderer
                            .text_update(&mut gui.preference.keywindow.text, &mut graphics.text_atlas, &mut draw_setting.renderer, 5)
                            .unwrap();
                        gui.preference.keywindow.buttons.iter_mut().for_each(|button| {
                            graphics.image_renderer.image_update(&mut button.image, &mut draw_setting.renderer, &mut graphics.image_atlas, 4);
                            graphics.text_renderer
                                .text_update(&mut button.text, &mut graphics.text_atlas, &mut draw_setting.renderer, 5)
                                .unwrap();
                        });
                    }
                },
                _ => {},
            }
        }

        // this cycles all the Image's in the Image buffer by first putting them in rendering order
        // and then uploading them to the GPU if they have moved or changed in any way. clears the
        // Image buffer for the next render pass. Image buffer only holds the ID's and Sortign info
        // of the finalized Indicies of each Image.
        graphics.image_renderer.finalize(&mut draw_setting.renderer);
        graphics.map_renderer.finalize(&mut draw_setting.renderer);
        graphics.text_renderer.finalize(&mut draw_setting.renderer);
        graphics.ui_renderer.finalize(&mut draw_setting.renderer);

        // Start encoding commands. this stores all the rendering calls for execution when
        // finish is called.
        let mut encoder = draw_setting.renderer.device().create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("command encoder"),
            },
        );

        // Run the render pass. for the games renderer
        graphics.render(&draw_setting.renderer, &mut encoder);

        // Submit our command queue. for it to upload all the changes that were made.
        // Also tells the system to begin running the commands on the GPU.
        draw_setting.renderer.queue().submit(std::iter::once(encoder.finish()));

        if time < seconds {
            gui.labels[LABEL_FPS].set_text(&mut draw_setting.renderer, &format!("FPS: {fps}"), Attrs::new());
            fps = 0u32;
            time = seconds + 1.0;
        }
        fps += 1;

        input_handler.end_frame();
        frame_time.update();
        draw_setting.renderer.present().unwrap();

        // These clear the Last used image tags.
        //Can be used later to auto unload things not used anymore if ram/gpu ram becomes a issue.
        graphics.image_atlas.trim();
        graphics.map_atlas.trim();
        graphics.text_atlas.trim();
    })?;

    Ok(())
}