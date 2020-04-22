use crate::prelude::*;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct SceneExampleInstance {
    engine: VermarineEngine<Renderables>,
}

#[methods]
impl SceneExampleInstance {
    fn _init(owner: Node) -> Self {
        let mut instance = SceneExampleInstance { engine: VermarineEngine::<Renderables>::new(owner) };

        // Add resources
        let renderables = Models::<Renderables>::default();
        instance.engine.resources.insert(load_renderables(renderables));
        instance.engine.push(Box::from(BaseState { }));

        instance
    }

    #[export]
    fn _ready(&mut self, owner: Node) {
        self.engine._ready(owner);
    }

    #[export]
    fn _physics_process(&mut self, owner: Node, delta: f64) {
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