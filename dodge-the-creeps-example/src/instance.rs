use crate::prelude::*;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct DodgeTheCreepsInstance {
    engine: RPopsEngine<Renderables>,
}

#[methods]
impl DodgeTheCreepsInstance {
    fn _init(owner: Node) -> Self {
        let mut instance = DodgeTheCreepsInstance { engine: RPopsEngine::<Renderables>::new(owner) };
        
        // Add resources
        let renderables = Models::<Renderables>::default();
        instance.engine.resources.insert(load_renderables(renderables));
        
        instance.engine.push(Box::new(MainState {
            executor: Executor::new(create_systems()),
        }));

        instance
    }

    #[export]
    fn _ready(&mut self, _owner: Node) {
        // Call engine _ready
        self.engine._ready(_owner);
    }

    #[export]
    fn _physics_process(&mut self, owner: Node, delta: f64) {
        self.engine._physics_process(owner, delta);
    }
}

// Function that registers all exposed classes to Godot
fn init(handle: gdnative::init::InitHandle) {
    handle.add_class::<DodgeTheCreepsInstance>();
}

// macros that create the entry-points of the dynamic library.
godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();