use crate::prelude::*;
use std::collections::HashMap;

pub struct Models<T> 
    where T: Eq + std::hash::Hash { 
    data: Vec<(PackedScene, Template, usize)>,
    name_data_lookup: HashMap<&'static str, usize>,
    t_data_lookup: HashMap<T, usize>,
}

unsafe impl<T> Sync for Models<T> where T: Eq + std::hash::Hash {}
unsafe impl<T> Send for Models<T> where T: Eq + std::hash::Hash {}

impl<T> Default for Models<T> 
    where T: Eq + std::hash::Hash {
    fn default() -> Self {
        Models::<T> {
            data: vec![],
            name_data_lookup: HashMap::new(),
            t_data_lookup: HashMap::new(),
        }
    }
}

impl<T> Models<T> 
    where T: Eq + std::hash::Hash {
    pub fn index_from_t(&self, index: &T) -> Option<usize> {
        match self.t_data_lookup.get(index) {
            Some(index) => Some(*index),
            None => None,
        }
    }

    pub fn index_from_alias(&self, index: &str) -> Option<usize> {
        match self.name_data_lookup.get(&index) {
            Some(index) => Some(*index),
            None => None,
        }
    }
    
    pub fn data_from_alias(&self, index: &str) -> Option<(Template, usize)> {
        if let Some(index) = self.index_from_alias(index) {
            return self.data_from_index(index);
        }
        None
    }

    pub fn data_from_t(&self, index: &T) -> Option<(Template, usize)> {
        if let Some(index) = self.index_from_t(index) {
            return self.data_from_index(index);
        }
        None
    }

    pub fn data_from_index(&self, index: usize) -> Option<(Template, usize)> {
        if let Some(data) = self.data.get(index) {
            return Some((data.1.clone(), index));
        }
        return None;
    }

    pub(crate) fn scene_from_index(&self, index: usize) -> Option<&PackedScene> {
        if let Some(data) = self.data.get(index) {
            return Some(&data.0);
        }
        None
    }

    pub fn insert(&mut self, alias: Option<&'static str>, t_key: Option<T>, scene: PackedScene, template: Template) -> Option<usize> {
        let index = self.data.len();
        let mut has_valid_key = false;

        if let Some(key) = alias {
            self.name_data_lookup.insert(key, index);
            has_valid_key = true;
        }
        if let Some(key) = t_key {
            self.t_data_lookup.insert(key, index);
            has_valid_key = true;
        }

        if has_valid_key {
            self.data.push((scene, template, index));
            return Some(index);
        }
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Template {
    None,
    Scene,
    ASprite(AnimSprite),
    APlayer(AnimPlayer),
    ATree(AnimTree),
}

impl Default for Template {
    fn default() -> Self {
        Template::None
    }    
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnimSprite {
    pub flip_h: bool,
    pub flip_v: bool,
    pub playing: bool,
    pub animation: &'static str,
}
impl Default for AnimSprite {
    fn default() -> Self {
        AnimSprite {
            flip_h: false,
            flip_v: false,
            playing: true,
            animation: "",
        }
    }
}
impl AnimSprite {
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct AnimPlayer {

}

impl AnimPlayer {

}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct AnimTree {

}

impl AnimTree {

}