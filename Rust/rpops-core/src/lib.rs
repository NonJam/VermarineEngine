use gdnative::*;
use legion::prelude::*;
use legion::prelude::World as LWorld;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Counter {
    num: i32,
}

/// The HelloWorld "class"
#[derive(NativeClass)]
#[inherit(Spatial)]
pub struct RPopsInstance {
    universe: Universe,
    world: LWorld,
    resources: Resources,
    executor: Executor,
}

// __One__ `impl` block can have the `#[methods]` attribute, which will generate
// code to automatically bind any exported methods to Godot.
#[methods]
impl RPopsInstance {
    
    /// The "constructor" of the class.
    fn _init(_owner: Spatial) -> Self {
        let universe = Universe::new();
        let world = universe.create_world();
        let resources = Resources::default();
        let executor = Executor::new(vec![
            SystemBuilder::<()>::new("TestSystem")
                .with_query(<(Write<Counter>)>::query())
                .build(move |commands, world, resource, queries| {
                    let mut to_print: Option<i32> = None;
                    let mut entities: i32 = 0;
                    for (entity, mut counter) in queries.iter_entities_mut(&mut *world) {
                        counter.num+=1;
                        entities+=1;
                        to_print = Some(counter.num);
                    }

                    godot_print!("Entities: {:?}", entities);
                    if let Some(num) = to_print {
                        godot_print!("{:?}", num);
                    }
                }),
        ]);

        RPopsInstance {
            universe,
            world,
            resources,
            executor,
        }
    }
    
    // In order to make a method known to Godot, the #[export] attribute has to be used.
    // In Godot script-classes do not actually inherit the parent class.
    // Instead they are"attached" to the parent object, called the "owner".
    // The owner is passed to every single exposed method.
    #[export]
    fn _ready(&mut self, _owner: Spatial) {
        self.world.insert(
            (), 
            (0..100).map(|_| (Counter { num: 0 },))
        );
    }

    #[export]
    fn _physics_process(&mut self, mut owner: Spatial, delta: f64) {
        self.executor.run_systems(&mut self.world, &mut self.resources);
    }

    #[export]
    fn _input(&mut self, mut owner: Spatial, event: Option<InputEvent>) {
    
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