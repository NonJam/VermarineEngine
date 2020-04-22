use crate::prelude::*;

pub fn load_renderables(mut renderables: Models<Renderables>) -> Models<Renderables> {
    use BlockRenderables::*;

    renderables.insert(Some("PlayerSegment"), Some(Renderables::Blocks(PlayerSegment)), load_scene("PlayerSegment"), Template::Scene);
    renderables.insert(Some("Tile"), Some(Renderables::Blocks(Tile)), load_scene("Tile"), Template::Scene);

    renderables
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Renderables {
    Blocks(BlockRenderables),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum BlockRenderables { 
    PlayerSegment,
    Tile
}