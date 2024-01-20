use graphics::*;
use cosmic_text::{Attrs, Metrics};
use winit::dpi::PhysicalSize;
use crate::resource::*;

pub const LABEL_FPS: usize = 0;

pub struct Interface {
    pub bg_layout: Image,
    pub labels: Vec<Text>,
}

impl Interface {
    pub fn new(resource: &TextureAllocation, renderer: &mut GpuRenderer, size: &PhysicalSize<f32>, scale: f64) -> Self {
        // Load the texture
        let mut bg_layout = Image::new(Some(resource.bg_layout), renderer, 1);

        // Setup the interface position, height, width, color and texture coordinate
        bg_layout.pos = Vec3::new(0.0, 0.0, 11.0);
        bg_layout.hw = Vec2::new(949.0, 802.0);
        bg_layout.uv = Vec4::new(0.0, 0.0, 949.0, 802.0);
        bg_layout.color = Color::rgba(255, 255, 255, 255);

        // Preparing labels
        let mut labels = Vec::with_capacity(1);

        // Prepare all labels that will be drawn in the interface
        let fps_text = create_label(renderer, size, scale,
                            Vec3::new(221.0, 16.0, 1.0), 
                            Vec2::new(100.0, 16.0),
                            Bounds::new(221.0, 16.0, 321.0, 32.0),
                            Color::rgba(120, 120, 120, 255));
        
        labels.push(fps_text);

        // Completed! We can now pass the struct
        Self {
            bg_layout,
            labels,
        }
    }
}

fn create_label(renderer: &mut GpuRenderer, 
                size: &PhysicalSize<f32>, 
                scale: f64,
                pos: Vec3,
                label_size: Vec2,
                bound: Bounds,
                color: Color,
) -> Text {
    let mut text = Text::new(
        renderer,
        Some(Metrics::new(16.0, 16.0).scale(scale as f32)),
        pos, label_size,
    );
    text.set_buffer_size(renderer, size.width as i32, size.height as i32)
        .set_bounds(Some(bound))
        .set_default_color(color);
    text
}