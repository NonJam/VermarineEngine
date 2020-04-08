use crate::prelude::*;

#[derive(NativeClass)]
#[inherit(Spatial)]
pub struct RPopsInstance {
    engine: RPopsEngine,
}

#[methods]
impl RPopsInstance {
    fn _init(owner: Spatial) -> Self {
        let mut instance = RPopsInstance { engine: RPopsEngine::new(owner) };
        instance.engine.set_systems(create_systems());
        instance
    }

    #[export]
    fn _ready(&mut self, _owner: Spatial) {
        // Call engine _ready
        self.engine._ready(_owner);

        use CreatureModels::*;
        let models = self.engine.resources.get::<Models<Model>>().unwrap();
        let slime_model_index = models.index_from_t(&Model::Creatures(Slime)).unwrap();
        let zombie_model_index = models.index_from_t(&Model::Creatures(Zombie)).unwrap();

        self.engine.world.insert(
            (), 
            (0..1).map(|_| (Renderable { model: slime_model_index }, GDSpatial, Position { x: 0, y: 0 }, ))
        );

        self.engine.world.insert(
            (), 
            (0..1).map(|_| (Renderable { model: zombie_model_index }, GDSpatial, Position { x: -10, y: 0 }, ))
        );

        self.engine.world.insert(
            (),
            (0..1).map(|_| (Position { x: -10, y: 0 }, ))
        );
    }

    #[export]
    fn _physics_process(&mut self, owner: Spatial, delta: f64) {
        self.engine._physics_process(owner, delta);
    }
}

// Function that registers all exposed classes to Godot
fn init(handle: gdnative::init::InitHandle) {
    handle.add_class::<RPopsInstance>();
}

// macros that create the entry-points of the dynamic library.
godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();