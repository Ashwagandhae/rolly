use futures::future::join_all;
use macroquad::prelude::*;
use std::collections::HashMap;

type Output = ((usize, usize), Texture2D);
pub struct TextureHolder(HashMap<String, Output>);

include!(concat!(env!("OUT_DIR"), "/texture_codegen.rs"));

impl TextureHolder {
    pub async fn new() -> Self {
        let texture_paths = TEXTURE_FILENAMES
            .iter()
            .map(|(filename, _)| format!("assets/textures/{}", filename))
            .collect::<Vec<_>>();
        let texture_load = texture_paths.iter().map(|filename| load_texture(filename));
        let textures = join_all(texture_load).await;
        let map = TEXTURE_FILENAMES
            .iter()
            .map(|(filename, size)| (filename.to_string(), *size))
            .zip(textures.into_iter().map(|texture| texture.unwrap()))
            .map(|((filename, size), texture)| (filename, (size, texture)))
            .collect::<HashMap<_, _>>();
        Self(map)
    }
}

impl TextureHolder {
    pub fn get(&self, index: &str) -> Option<&Output> {
        self.0.get(index)
    }
    pub fn get_mut(&mut self, index: &str) -> Option<&mut Output> {
        self.0.get_mut(index)
    }
}

impl std::ops::Index<&str> for TextureHolder {
    type Output = Output;

    fn index(&self, index: &str) -> &Self::Output {
        self.0.get(index).unwrap()
    }
}

impl std::ops::IndexMut<&str> for TextureHolder {
    fn index_mut(&mut self, index: &str) -> &mut Self::Output {
        self.0.get_mut(index).unwrap()
    }
}
