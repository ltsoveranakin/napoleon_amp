use std::collections::HashMap;
use symphonia::core::meta::{StandardTagKey, Tag, Value};

#[derive(Default)]
pub struct SongTags {
    std_key_tags: HashMap<StandardTagKey, Value>,
    no_std_key_tags: HashMap<String, Value>,
}

impl SongTags {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_tag(&mut self, tag: Tag) {
        if let Some(std_key) = tag.std_key {
            self.std_key_tags.insert(std_key, tag.value);
        } else {
            self.no_std_key_tags.insert(tag.key, tag.value);
        }
    }

    pub fn get_std(&self, std_key: &StandardTagKey) -> Option<&Value> {
        self.std_key_tags.get(&std_key)
    }
}
