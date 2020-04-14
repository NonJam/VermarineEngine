#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub rotation: euclid::Angle::<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Renderable {
    pub model: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GDSpatial;