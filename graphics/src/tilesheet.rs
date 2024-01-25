use crate::{AtlasSet, GpuRenderer, Texture};
use image::{self, EncodableLayout, ImageBuffer, RgbaImage};

//used to map the tile in the tilesheet back visually
//this is only needed for the Editor.
#[derive(Debug)]
pub struct Tile {
    pub x: u32,
    pub y: u32,
    pub id: u32,
    pub tex_id: usize,
}

#[derive(Debug, Default)]
//We can use this for editor loading and just as a precursor.
pub struct TileSheet {
    pub tiles: Vec<Tile>,
}

impl TileSheet {
    pub fn new(
        texture: Texture,
        renderer: &GpuRenderer,
        atlas: &mut AtlasSet<String, i32>,
        tilesize: u32,
    ) -> Option<TileSheet> {
        let tilecount =
            (texture.size().0 / tilesize) * (texture.size().1 / tilesize);
        let sheet_width = texture.size().0 / tilesize;
        let atlas_width = atlas.size().x / tilesize;
        let sheet_image: RgbaImage = ImageBuffer::from_raw(
            texture.size().0,
            texture.size().1,
            texture.bytes.to_owned(),
        )
        .unwrap_or(ImageBuffer::new(texture.size().0, texture.size().1));
        let mut tiles = Vec::with_capacity(tilecount as usize);

        // lets check this to add in the empty tile set first if nothing else yet exists.
        // Also lets add the black tile.
        let empty = if let Some(empty) = atlas.lookup(&"Empty".to_owned()) {
            empty
        } else {
            let image: RgbaImage = ImageBuffer::new(tilesize, tilesize);
            atlas.upload(
                "Empty".to_owned(),
                image.as_bytes(),
                tilesize,
                tilesize,
                0,
                renderer,
            )?
        };

        for id in 0..tilecount {
            let mut image: RgbaImage = ImageBuffer::new(tilesize, tilesize);
            // get its location to remap it back visually.
            let (tilex, tiley) = (
                ((id % sheet_width) * tilesize),
                ((id / sheet_width) * tilesize),
            );

            // lets create the tile from the texture.
            for y in 0..tilesize {
                for x in 0..tilesize {
                    let posx = (id % sheet_width) * tilesize + x;
                    let posy = (id / sheet_width) * tilesize + y;
                    let pixel = sheet_image.get_pixel(posx, posy);
                    image.put_pixel(x, y, *pixel);
                }
            }
            // we upload the tile regardless this avoid tilesheet issues later.
            let name: String = format!("{}-{}", texture.name(), id);
            let (tex_id, allocation) = atlas.upload_with_alloc(
                name,
                image.as_bytes(),
                tilesize,
                tilesize,
                0,
                renderer,
            )?;

            if image.enumerate_pixels().all(|p| p.2 .0[3] == 0) {
                // lets use our only Blank tile. this will always be the first loaded.
                // We use this when tiles are empty to avoid issues later when we do use
                // these spots for other tiles.
                tiles.push(Tile {
                    x: tilex,
                    y: tiley,
                    id: 0,
                    tex_id: empty,
                })
            } else {
                let (posx, posy) = allocation.position();
                tiles.push(Tile {
                    x: tilex,
                    y: tiley,
                    id: (posx / tilesize) + ((posy / tilesize) * atlas_width),
                    tex_id,
                })
            }
        }

        // We return as Some(tilesheet) this allows us to check above upon
        // upload if a tile failed to get added or not due to no more room.
        Some(TileSheet { tiles })
    }

    pub fn upload(
        &mut self,
        texture: Texture,
        renderer: &GpuRenderer,
        atlas: &mut AtlasSet<String, i32>,
        tilesize: u32,
    ) -> Option<()> {
        let tilecount =
            (texture.size().0 / tilesize) * (texture.size().1 / tilesize);
        let atlas_width = atlas.size().x / tilesize;
        let sheet_width = texture.size().0 / tilesize;
        let sheet_image: RgbaImage = ImageBuffer::from_raw(
            texture.size().0,
            texture.size().1,
            texture.bytes.to_owned(),
        )
        .unwrap_or(ImageBuffer::new(texture.size().0, texture.size().1));

        // lets check this to add in the empty tile set first if nothing else yet exists.
        // Also lets add the black tile.
        let empty = if let Some(empty) = atlas.lookup(&"Empty".to_owned()) {
            empty
        } else {
            let image: RgbaImage = ImageBuffer::new(tilesize, tilesize);
            atlas.upload(
                "Empty".to_owned(),
                image.as_bytes(),
                tilesize,
                tilesize,
                0,
                renderer,
            )?
        };

        for id in 0..tilecount {
            let (tilex, tiley) = (
                ((id % sheet_width) * tilesize),
                ((id / sheet_width) * tilesize),
            );
            let mut image: RgbaImage = ImageBuffer::new(tilesize, tilesize);

            // lets create the tile from the texture.
            for y in 0..tilesize {
                for x in 0..tilesize {
                    let posx = (id % sheet_width) * tilesize + x;
                    let posy = (id / sheet_width) * tilesize + y;
                    let pixel = sheet_image.get_pixel(posx, posy);
                    image.put_pixel(x, y, *pixel);
                }
            }

            let name: String = format!("{}-{}", texture.name(), id);
            let (tex_id, allocation) = atlas.upload_with_alloc(
                name,
                image.as_bytes(),
                tilesize,
                tilesize,
                0,
                renderer,
            )?;

            if image.enumerate_pixels().all(|p| p.2 .0[3] == 0) {
                // lets use our only Blank tile. this will always be the first loaded.
                // We use this when tiles are empty to avoid issues later when we do use
                // these spots for other tiles.
                self.tiles.push(Tile {
                    x: tilex,
                    y: tiley,
                    id: 0,
                    tex_id: empty,
                })
            } else {
                let (posx, posy) = allocation.position();
                self.tiles.push(Tile {
                    x: tilex,
                    y: tiley,
                    id: (posx / tilesize) + ((posy / tilesize) * atlas_width),
                    tex_id,
                })
            }
        }

        // We return as Some(()) this allows us to check above upon
        // upload if a tile failed to get added or not due to no more room.
        Some(())
    }
}