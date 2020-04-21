#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TakesInput {
    pub speed: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Collider {
    pub width: f32,
    pub height: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnemyComp { }

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerComp { }