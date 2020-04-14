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
        // Add systems
        instance.engine.set_systems(create_systems());
        
        // Add resources
        let renderables = Models::<Renderables>::default();
        instance.engine.resources.insert(load_renderables(renderables));
        
        instance
    }

    #[export]
    fn _ready(&mut self, _owner: Node) {
        // Call engine _ready
        self.engine._ready(_owner);

        use CreatureRenderables::*;
        let renderables = self.engine.resources.get::<Models<Renderables>>().unwrap();
        let player_renderable_index = renderables.index_from_t(&Renderables::Creatures(Player)).unwrap();

        self.engine.world.insert(
            (), 
            (0..1).map(|_| (
                Renderable { model: player_renderable_index }, 
                GDSpatial, 
                Position { x: 240f32, y: 450f32, rotation: euclid::Angle::radians(0f32) }, 
                TakesInput { speed: 400f32 / 60f32 }, 
                Velocity::default(),
                Collider { width: 25.0, height: 25.0, offset_x: 0.0, offset_y: -2.5},
                PlayerComp { },
            ))
        );
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