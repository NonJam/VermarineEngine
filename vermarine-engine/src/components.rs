use crate::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub rotation: euclid::Angle::<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Renderable {
    pub index: usize,
    pub template: Template,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GDSpatial;