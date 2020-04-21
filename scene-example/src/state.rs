use crate::prelude::*;

pub struct ExampleStateA {

}

impl State for ExampleStateA {
    fn update(&mut self, data: &mut StateData, resources: &mut Resources) -> Trans {
        godot_print!("A");
        Trans::None
    }
}

pub struct ExampleStateB {

}

impl State for ExampleStateB {
    fn update(&mut self, data: &mut StateData, resources: &mut Resources) -> Trans {
        godot_print!("B");
        Trans::None
    }
}

pub struct ExampleStateC {

}

impl State for ExampleStateC {
    fn update(&mut self, data: &mut StateData, resources: &mut Resources) -> Trans {
        godot_print!("C");
        Trans::None
    }
}