use crate::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub rotation: euclid::Angle::<f32>,
}

impl Default for Position {
    fn default() -> Self {
        Position {
            x: 0f32,
            y: 0f32,
            rotation: euclid::Angle::radians(0f32),
        }
    }
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Position {
            x: x,
            y: y,
            rotation: euclid::Angle::radians(0f32),
        }
    }
}

//#[derive(Clone, Debug, PartialEq)]
pub enum RenderableCommand {
    Delete(Option<Node>),
}

//#[derive(Clone, Debug, PartialEq, Default)]
pub struct Renderable {
    pub spatial: Option<GDSpatial>,
    pub transform: Position,

    pub renderable_id: Option<usize>,
    pub template: Option<Template>,

    pub(crate) container_node: Option<Node>,
    pub(crate) children_node: Option<Node>,
    pub(crate) renderable_node: Option<Node>,

    pub(crate) children: Vec<Renderable>,
    pub(crate) orphans: Vec<Option<Node>>,
}

unsafe impl Send for Renderable {}
unsafe impl Sync for Renderable {}

impl Default for Renderable {
    fn default() -> Self {
        Renderable {
            spatial: Some(GDSpatial::new()),
            transform: Position::default(),

            renderable_id: None,
            template: None,

            container_node: None,
            children_node: None,
            renderable_node: None,

            children: vec![],
            orphans: vec![],
        }
    }
}

impl Renderable {
    pub fn new(transform: Position, renderable_id: usize, template: Template) -> Self {
        let mut res = Renderable::default();
        res.transform = transform;
        res.renderable_id = Some(renderable_id);
        res.template = Some(template);
        res
    }

    pub fn get_children(&self) -> &Vec<Renderable> {
        &self.children
    }

    pub fn get_children_mut(&mut self) -> &mut [Renderable] {
        &mut self.children[..]
    }

    pub fn try_get_child_mut(&mut self, index: usize) -> Option<&mut Renderable> {
        self.children.get_mut(index)
    }

    pub fn get_child_mut(&mut self, index: usize) -> &mut Renderable {
        match self.try_get_child_mut(index) {
            Some(value) => value,
            None => panic!("Attempt to access Renderable in MultiRenderable index {} failed", index),
        } 
    }

    pub fn try_get_child(&self, index: usize) -> Option<&Renderable> {
        self.children.get(index)
    }

    pub fn get_child(&self, index: usize) -> &Renderable {
        match self.try_get_child(index) {
            Some(value) => value,
            None => panic!("Attempt to access Renderable in MultiRenderable index {} failed", index),
        } 
    }

    pub fn remove_child(&mut self, index: usize) {
        self.orphans.push(self.children.get(index).unwrap().renderable_node);
        self.children.remove(index);
    }

    pub fn insert_child(&mut self, index: usize, item: Renderable) {
        self.children.insert(index, item);
    }

    pub fn push_child(&mut self, item: Renderable) {
        self.children.push(item);
    }

    pub fn pop_child(&mut self) -> Option<Renderable> {
        self.orphans.push(self.renderable_node);
        self.children.pop()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GDSpatial {
    pub(crate) prev_id: Option<usize>,
}

impl GDSpatial {
    pub fn new() -> Self {
        GDSpatial {
            prev_id: None,
        }
    }

    pub(crate) fn is_dirty(&self, renderable: &Renderable) -> bool {
        if let Some(prev_id) = self.prev_id {
            if prev_id != renderable.renderable_id.unwrap() {
                return true;
            }
            return false;
        } else {
            return true;
        }
    }
}