use crate::prelude::*;

pub struct ExampleStateA {

}

impl State for ExampleStateA {
    fn update(&mut self, data: &mut StateData, resources: &mut Resources) {
        godot_print!("A");
    }
}

pub struct ExampleStateB {

}

impl State for ExampleStateB {
    fn update(&mut self, data: &mut StateData, resources: &mut Resources) {
        godot_print!("B");
    }
}

pub struct ExampleStateC {

}

impl State for ExampleStateC {
    fn update(&mut self, data: &mut StateData, resources: &mut Resources) {
        godot_print!("C");
    }
}