#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Renderable {
    pub model: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GDSpatial;