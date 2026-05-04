use eframe::egui;
use eframe::egui::{TextureHandle, TextureOptions};
use egui_extras::image;
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

                let image = image::load_image_bytes(file.contents()).map_err(|_| ())?;
                let tex_handle = ctx.load_texture(tex_file_name, image, TextureOptions::NEAREST);

                Ok(v.insert(tex_handle))
            }
        }
    }
}
