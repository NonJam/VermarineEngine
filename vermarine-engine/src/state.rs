use std::fmt::{Debug, Formatter, Result as FmtResult};
use crate::prelude::*;
use std::collections::HashMap;

pub enum Trans {
    /// Continue as normal
    None,

    /// Add a new state to the top of the stack
    Push(Box<dyn State>),

    /// Remove the state at the top of the stack
    Pop,

    /// Set the the state at the top of the stack to the given state
    Switch(Box<dyn State>),

    /// Replaces the stack with the given state
    Replace(Box<dyn State>),

    /// Replaces the stack with the given stack
    NewStack(Vec<Box<dyn State>>),

    /// Executes a sequence of StateTrans'
    Sequence(Vec<Trans>),

    /// Quit out of the engine
    Quit,
}

impl Debug for Trans {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Trans::None => f.write_str("None"),
            Trans::Pop => f.write_str("Pop"),
            Trans::Push(_) => f.write_str("Push"),
            Trans::Switch(_) => f.write_str("Switch"),
            Trans::Replace(_) => f.write_str("Replace"),
            Trans::NewStack(_) => f.write_str("NewStack"),
            Trans::Sequence(sequence) => f.write_str(&format!("Sequence {:?}", sequence)),
            Trans::Quit => f.write_str("Quit"),
        }
    }
}

pub struct StateData {
    pub(crate) receiver: crossbeam_channel::Receiver<legion::event::Event>,
    pub(crate) node: Option<Node>,
    pub(crate) node_lookup: HashMap<Entity, Node>,
    pub world: LWorld,
}

impl<'a> StateData {
    pub(crate) fn new(mut world: LWorld) -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        world.subscribe(sender, any());
        StateData { 
            world,
            receiver,
            node: None,
            node_lookup: HashMap::new(),
        }
    }
}

pub trait State {
    fn on_push(&mut self, _data: &mut StateData, _resources: &mut Resources) { }
    fn on_pop(&mut self, _data: &mut StateData, _resources: &mut Resources) { }
    fn on_cover(&mut self, _data: &mut StateData, _resources: &mut Resources) { }
    fn on_uncover(&mut self, _data: &mut StateData, _resources: &mut Resources) { }
    fn update(&mut self, _data: &mut StateData, _resources: &mut Resources) { }
    fn shadow_update(&mut self, _data: &mut StateData, _resources: &mut Resources) { }
}