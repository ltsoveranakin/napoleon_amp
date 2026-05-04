use eframe::egui;
use eframe::egui::{TextureHandle, TextureOptions};
use napoleon_amp_core::assets::ASSETS_DIR;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

pub(crate) struct TexturePool {
    texture_map: HashMap<String, TextureHandle>,
}

impl TexturePool {
    pub(crate) fn new() -> Self {
        Self {
            texture_map: HashMap::new(),
        }
    }

    pub(crate) fn get_tex(
        &mut self,
        tex_file_name: &str,
        ctx: &egui::Context,
    ) -> Result<&TextureHandle, ()> {
        match self.texture_map.entry(tex_file_name.to_string()) {
            Entry::Occupied(o) => Ok(o.into_mut()),

            Entry::Vacant(v) => {
                let file = ASSETS_DIR
                    .get_file(format!("sprites/{}", tex_file_name))
                    .ok_or(())?;

                let im = image::load_from_memory(file.contents()).map_err(|_| ())?;

                let image_buffer = im.to_rgba8();
                let pixels = image_buffer.as_flat_samples();

                let col_image = egui::ColorImage::from_rgba_unmultiplied(
                    [im.width() as usize, im.height() as usize],
                    pixels.as_slice(),
                );

                let tex_handle =
                    ctx.load_texture(tex_file_name, col_image, TextureOptions::NEAREST);

                Ok(v.insert(tex_handle))
            }
        }
    }
}
