use indexmap::IndexMap;
use graphics::*;
use crate::collection::TEXTURE_SIZE;

// Modify this based on how many tilesheet image
pub const MAX_TILESHEET: u32 = 4;

pub struct TextureData {
    pub name: String,
    pub allocation: usize,
}

pub struct TilesheetData {
    pub name: String,
    pub tile: TileSheet,
}

pub struct TextureAllocation {
    pub bg_layout: TextureData,
    pub tool_icon: TextureData,
    pub tab_icon: TextureData,
    pub tileset_button: TextureData,
    pub tileset_list_bg: TextureData,
    pub tileset_list_select: TextureData,
    pub scrollbar: TextureData,
    pub tab_option: TextureData,
    pub dialog_button: TextureData,
    pub option_button: TextureData,
    pub preference_button: TextureData,
    pub selection_drop_button: TextureData,
    pub tileset_bg: TextureData,
    pub mapview_bg: TextureData,
    pub tilesheet: Vec<TilesheetData>,
    // This will be used for eyedropper tool
    pub tile_location: IndexMap<usize, (u32, u32, u32)>,
}

impl TextureAllocation {
    pub fn new(
        atlases: &mut Vec<AtlasSet>,
        renderer: &GpuRenderer,
    ) -> Result<Self, AscendingError> {
        // This is how we load a image into a atlas/Texture. It returns the location of the image
        // within the texture. its x, y, w, h.  Texture loads the file. group_uploads sends it to the Texture
        // renderer is used to upload it to the GPU when done.
        let bg_layout = TextureData {
            name: "layout.png".to_string(),
            allocation: Texture::from_file("images/gui/layout.png")?
                .upload(&mut atlases[0], renderer)
                .ok_or_else(|| OtherError::new("failed to upload image"))?,
        };

        let tool_icon = TextureData {
            name: "tool_buttons.png".to_string(),
            allocation: Texture::from_file("images/gui/tool_buttons.png")?
                .upload(&mut atlases[0], renderer)
                .ok_or_else(|| OtherError::new("failed to upload image"))?,
        };

        let tab_option = TextureData {
            name: "tab_option_button.png".to_string(),
            allocation: Texture::from_file("images/gui/tab_option_button.png")?
                .upload(&mut atlases[0], renderer)
                .ok_or_else(|| OtherError::new("failed to upload image"))?,
        };

        let tab_icon = TextureData {
            name: "map_setting_buttons.png".to_string(),
            allocation: Texture::from_file(
                "images/gui/map_setting_buttons.png",
            )?
            .upload(&mut atlases[0], renderer)
            .ok_or_else(|| OtherError::new("failed to upload image"))?,
        };

        let tileset_button = TextureData {
            name: "tileset_selection_button.png".to_string(),
            allocation: Texture::from_file(
                "images/gui/tileset_selection_button.png",
            )?
            .upload(&mut atlases[0], renderer)
            .ok_or_else(|| OtherError::new("failed to upload image"))?,
        };

        let tileset_list_bg = TextureData {
            name: "tileset_list_bg.png".to_string(),
            allocation: Texture::from_file("images/gui/tileset_list_bg.png")?
                .upload(&mut atlases[0], renderer)
                .ok_or_else(|| OtherError::new("failed to upload image"))?,
        };

        let tileset_list_select = TextureData {
            name: "tileset_list_select.png".to_string(),
            allocation: Texture::from_file(
                "images/gui/tileset_list_select.png",
            )?
            .upload(&mut atlases[0], renderer)
            .ok_or_else(|| OtherError::new("failed to upload image"))?,
        };

        let scrollbar = TextureData {
            name: "scrollbar.png".to_string(),
            allocation: Texture::from_file("images/gui/scrollbar.png")?
                .upload(&mut atlases[0], renderer)
                .ok_or_else(|| OtherError::new("failed to upload image"))?,
        };

        let dialog_button = TextureData {
            name: "dialog_button.png".to_string(),
            allocation: Texture::from_file("images/gui/dialog_button.png")?
                .upload(&mut atlases[0], renderer)
                .ok_or_else(|| OtherError::new("failed to upload image"))?,
        };

        let option_button = TextureData {
            name: "option_button.png".to_string(),
            allocation: Texture::from_file("images/gui/option_button.png")?
                .upload(&mut atlases[0], renderer)
                .ok_or_else(|| OtherError::new("failed to upload image"))?,
        };

        let preference_button = TextureData {
            name: "preference_button.png".to_string(),
            allocation: Texture::from_file("images/gui/preference_button.png")?
                .upload(&mut atlases[0], renderer)
                .ok_or_else(|| OtherError::new("failed to upload image"))?,
        };

        let selection_drop_button = TextureData {
            name: "selection_drop_button.png".to_string(),
            allocation: Texture::from_file("images/gui/selection_drop_button.png")?
                .upload(&mut atlases[0], renderer)
                .ok_or_else(|| OtherError::new("failed to upload image"))?,
        };

        let tileset_bg = TextureData {
            name: "tileset_bg.png".to_string(),
            allocation: Texture::from_file("images/gui/tileset_bg.png")?
                .upload(&mut atlases[0], renderer)
                .ok_or_else(|| OtherError::new("failed to upload image"))?,
        };

        let mapview_bg = TextureData {
            name: "mapview_bg.png".to_string(),
            allocation: Texture::from_file("images/gui/mapview_bg.png")?
                .upload(&mut atlases[0], renderer)
                .ok_or_else(|| OtherError::new("failed to upload image"))?,
        };

        let mut tile_location = IndexMap::new();
        let mut tilesheet = Vec::with_capacity(MAX_TILESHEET as usize);
        for index in 0..MAX_TILESHEET {
            let res = TilesheetData {
                name: format!("tile_{}.png", index),
                tile: Texture::from_file(format!(
                    "images/tiles/tile_{}.png",
                    index
                ))?
                .new_tilesheet(&mut atlases[1], &renderer, TEXTURE_SIZE)
                .ok_or_else(|| OtherError::new("failed to upload tiles"))?,
            };

            // Store the tile location
            for tile in &res.tile.tiles {
                if tile.tex_id > 0 {
                    tile_location.insert(tile.tex_id, (tile.x, tile.y, index));
                }
            }

            tilesheet.push(res);
        }

        // Complete! We can now pass the result
        Ok(Self {
            bg_layout,
            tool_icon,
            tab_icon,
            tileset_button,
            tileset_list_bg,
            tileset_list_select,
            scrollbar,
            tab_option,
            dialog_button,
            option_button,
            preference_button,
            selection_drop_button,
            tileset_bg,
            mapview_bg,
            tilesheet,
            tile_location,
        })
    }
}
