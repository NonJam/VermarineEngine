use crate::prelude::*;

pub fn load_renderables(mut renderables: Models<Renderables>) -> Models<Renderables> {
    use Ui::*;

    renderables.insert(Some("Ui"), Some(Renderables::UI(Main)), load_scene("ui"), Template::Scene);

    renderables
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Renderables {
    UI(Ui),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Ui { 
    Main
}