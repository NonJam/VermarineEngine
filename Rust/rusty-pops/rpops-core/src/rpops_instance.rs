use crate::prelude::*;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct RPopsInstance {
    engine: RPopsEngine<Model>,
}

#[methods]
impl RPopsInstance {
    fn _init(owner: Node) -> Self {
        let mut instance = RPopsInstance { engine: RPopsEngine::<Model>::new(owner) };
        
        // Add systems
        instance.engine.set_systems(create_systems());
        
        // Add resources
        let models = Models::<Model>::default();
        instance.engine.resources.insert(add_models(models));
        
        instance
    }

    #[export]
    fn _ready(&mut self, _owner: Node) {
        // Call engine _ready
        self.engine._ready(_owner);

        use CreatureModels::*;
        let models = self.engine.resources.get::<Models<Model>>().unwrap();
        let slime_model_index = models.index_from_t(&Model::Creatures(Slime)).unwrap();
        let zombie_model_index = models.index_from_t(&Model::Creatures(Zombie)).unwrap();

        self.engine.world.insert(
            (), 
            (0..1).map(|_| (Renderable { model: slime_model_index }, GDSpatial, Position { x: 0f32, y: 0f32, rotation: euclid::Angle::radians(0f32) }, ))
        );

        self.engine.world.insert(
            (), 
            (0..1).map(|_| (Renderable { model: zombie_model_index }, GDSpatial, Position { x: -10f32, y: 0f32, rotation: euclid::Angle::radians(0f32) }, ))
        );

        self.engine.world.insert(
            (),
            (0..1).map(|_| (Position { x: -10f32, y: 0f32, rotation: euclid::Angle::radians(0f32) }, ))
        );
    }

    #[export]
    fn _physics_process(&mut self, owner: Node, delta: f64) {
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