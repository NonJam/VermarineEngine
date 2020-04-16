use crate::prelude::*;

pub fn add_models(mut models: Models<Model>) -> Models<Model> {
    use CreatureModels::*;

    models.insert(None, Some(Model::Creatures(Slime)), load_scene("Square"), Template::Scene);
    models.insert(None, Some(Model::Creatures(Zombie)), load_scene("Rectangle"), Template::Scene);

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