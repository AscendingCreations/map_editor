use graphics::*;
use crate::collection::TEXTURE_SIZE;

// Modify this based on how many tilesheet image
const MAX_TILESHEET: u32 = 4;

pub struct TextureAllocation {
    pub bg_layout: Allocation,
    pub white: Allocation,
    pub tilesheet: Vec<TileSheet>,
}

impl TextureAllocation {
    pub fn new(atlases: &mut Vec<AtlasGroup>, renderer: &GpuRenderer) -> Result<Self, AscendingError> {
        // This is how we load a image into a atlas/Texture. It returns the location of the image
        // within the texture. its x, y, w, h.  Texture loads the file. group_uploads sends it to the Texture
        // renderer is used to upload it to the GPU when done.
        let bg_layout = Texture::from_file("images/gui/layout.png")?
            .group_upload(&mut atlases[0], renderer)
            .ok_or_else(|| OtherError::new("failed to upload image"))?;

        let white = Texture::from_file("images/gui/white.png")?
            .group_upload(&mut atlases[0], renderer)
            .ok_or_else(|| OtherError::new("failed to upload image"))?;

        let mut tilesheet = Vec::with_capacity(MAX_TILESHEET as usize);
        for index in 0..MAX_TILESHEET {
            let res = Texture::from_file(format!("images/tiles/tile_{}.png", index))?
                .new_tilesheet(&mut atlases[1], &renderer, TEXTURE_SIZE)
                .ok_or_else(|| OtherError::new("failed to upload tiles"))?;
            tilesheet.push(res);
        }

        // Complete! We can now pass the result
        Ok(Self {
            bg_layout,
            white,
            tilesheet,
        })
    }
}