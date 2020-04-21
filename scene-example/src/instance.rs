use crate::prelude::*;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct SceneExampleInstance {
    engine: RPopsEngine<Renderables>,
}

#[methods]
impl SceneExampleInstance {
    fn _init(owner: Node) -> Self {
        let mut instance = SceneExampleInstance { engine: RPopsEngine::<Renderables>::new(owner) };

        // Add resources
        let renderables = Models::<Renderables>::default();
        instance.engine.resources.insert(load_renderables(renderables));
        
        instance.engine.push(Box::new(ExampleStateA { }) );
        instance.engine.push(Box::new(ExampleStateB { }) );
        instance.engine.push(Box::new(ExampleStateC { }) );

        instance
    }

    #[export]
    fn _ready(&mut self, _owner: Node) {
        self.engine._ready(_owner);
    }

    #[export]
    fn _physics_process(&mut self, owner: Node, delta: f64) {
        if Input::godot_singleton().is_action_just_pressed(GodotString::from("pop")) {
            let sender = self.engine.resources.get::<Wrapper<crossbeam_channel::Sender<Box<(dyn FnOnce() -> Trans + 'static)>>>>().unwrap();
            (*sender).inner.try_send(Box::from(|| Trans::Pop));
        } else if Input::godot_singleton().is_action_just_pressed(GodotString::from("a")) {
            let sender = self.engine.resources.get::<Wrapper<crossbeam_channel::Sender<Box<(dyn FnOnce() -> Trans + 'static)>>>>().unwrap();
            (*sender).inner.try_send(Box::from(|| Trans::Push(Box::new(ExampleStateA { }))));
        } else if Input::godot_singleton().is_action_just_pressed(GodotString::from("b")) {
            let sender = self.engine.resources.get::<Wrapper<crossbeam_channel::Sender<Box<(dyn FnOnce() -> Trans + 'static)>>>>().unwrap();
            (*sender).inner.try_send(Box::from(|| Trans::Push(Box::new(ExampleStateB { }))));
        } else if Input::godot_singleton().is_action_just_pressed(GodotString::from("c")) {
            let sender = self.engine.resources.get::<Wrapper<crossbeam_channel::Sender<Box<(dyn FnOnce() -> Trans + 'static)>>>>().unwrap();
            (*sender).inner.try_send(Box::from(|| Trans::Push(Box::new(ExampleStateC { }))));
        }

        self.engine._physics_process(owner, delta);
    }
}

// Function that registers all exposed classes to Godot
fn init(handle: gdnative::init::InitHandle) {
    handle.add_class::<SceneExampleInstance>();
}

// macros that create the entry-points of the dynamic library.
godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();