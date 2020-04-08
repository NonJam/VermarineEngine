use crate::prelude::*;
use std::collections::HashMap;

pub struct Models<T> 
    where T: Eq + std::hash::Hash { 
    data: Vec<Template>,
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
    
    pub fn get_model(&self, index: usize) -> Option<&Template> {
        self.data.get(index)
    }

    pub fn insert(&mut self, alias: Option<&'static str>, t_key: Option<T>, data: Template) -> Option<usize> {
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
            self.data.push(data);
            return Some(index);
        }
        None
    }
}

pub fn add_models(mut models: Models<Model>) -> Models<Model> {
    use CreatureModels::*;

    models.insert(None, Some(Model::Creatures(Slime)), load_scene("Square"));
    models.insert(None, Some(Model::Creatures(Zombie)), load_scene("Rectangle"));

    models
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Model {
    Creatures(CreatureModels),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum CreatureModels { 
    Slime,
    Zombie,
}

pub enum Template {
    None,
    Scene(PackedScene),
}

impl Default for Template {
    fn default() -> Self {
        Template::None
    }    
}