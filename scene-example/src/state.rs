use crate::prelude::*;

pub struct BaseState {

}

impl State for BaseState {
    fn on_push(&mut self, data: &mut StateData, resources: &mut Resources) {
        let sender = resources.get::<Wrapper<crossbeam_channel::Sender<Box<(dyn FnOnce() -> Trans + 'static)>>>>().unwrap();
        (*sender).inner.try_send(Box::from(|| Trans::Push(Box::new(PrintState { output: "Bottom of stack" }))));
    }
}

pub struct PrintState {
    pub output: &'static str,
}

impl State for PrintState {
    fn update(&mut self, data: &mut StateData, resources: &mut Resources) {
        godot_print!("{}", self.output);
        if Input::godot_singleton().is_action_just_pressed(GodotString::from("a")) {
            let sender = resources.get::<Wrapper<crossbeam_channel::Sender<Box<(dyn FnOnce() -> Trans + 'static)>>>>().unwrap();
            (*sender).inner.try_send(Box::from(|| Trans::Push(Box::new(PrintState { output: "A" }))));
        } else if Input::godot_singleton().is_action_just_pressed(GodotString::from("b")) {
            let sender = resources.get::<Wrapper<crossbeam_channel::Sender<Box<(dyn FnOnce() -> Trans + 'static)>>>>().unwrap();
            (*sender).inner.try_send(Box::from(|| Trans::Push(Box::new(PrintState { output: "B" }))));
        } else if Input::godot_singleton().is_action_just_pressed(GodotString::from("c")) {
            let sender = resources.get::<Wrapper<crossbeam_channel::Sender<Box<(dyn FnOnce() -> Trans + 'static)>>>>().unwrap();
            (*sender).inner.try_send(Box::from(|| Trans::Push(Box::new(PrintState { output: "C" }))));
        }
    }

    fn shadow_update(&mut self, data: &mut StateData, resources: &mut Resources) {
        if Input::godot_singleton().is_action_just_pressed(GodotString::from("pop")) {
            let sender = resources.get::<Wrapper<crossbeam_channel::Sender<Box<(dyn FnOnce() -> Trans + 'static)>>>>().unwrap();
            (*sender).inner.try_send(Box::from(|| Trans::Pop));
        }
    }

    fn on_push(&mut self, data: &mut StateData, resources: &mut Resources) {
        let renderables = resources.get::<Models<Renderables>>().unwrap();
        let ui = renderables.data_from_t(&Renderables::UI(Ui::Main)).unwrap();
        
        data.world.insert(
            (),
            (0..1).map(|_| (
                Renderable { index: ui.1, template: ui.0 }, 
                GDSpatial, 
                Position { x: 0f32, y: 0f32, rotation: euclid::Angle::radians(0f32) }, 
            )),
        );
    }
}