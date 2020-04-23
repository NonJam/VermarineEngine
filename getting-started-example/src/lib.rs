use gdnative::*;
use vermarine_engine::prelude::*;

/// The HelloWorld "class"
#[derive(NativeClass)]
#[inherit(Node)]
pub struct HelloWorld {
    engine: VermarineEngine<i32>,
}

// __One__ `impl` block can have the `#[methods]` attribute, which will generate
// code to automatically bind any exported methods to Godot.
#[methods]
impl HelloWorld {
    
    /// The "constructor" of the class.
    fn _init(owner: Node) -> Self {
        let mut instance = HelloWorld { engine: VermarineEngine::<i32>::new(owner)};

        // Set up Models<T> resource
        let mut models = Models::<i32>::default();
        models.insert(Some("StringKey"), None, load_scene("square"), Template::Scene);
        models.insert(Some("Pause"), None, load_scene("pause"), Template::Scene);
        instance.engine.resources.insert(models);

        instance.engine.push(Box::new(YourState { count: 0 }));
        
        instance
    }
    
    // We need to change this from &self to &mut self
    #[export]
    fn _ready(&mut self, owner: Node) {
        // The engine has code that needs to be run for initialisation
        self.engine._ready(owner); 
    }
    
    // We need to add a physics_process method for our engine
    #[export]
    fn _physics_process(&mut self, owner: Node, delta: f64) {
        self.engine._physics_process(owner, delta);
    }
}

// Function that registers all exposed classes to Godot
fn init(handle: gdnative::init::InitHandle) {
    handle.add_class::<HelloWorld>();
}

// macros that create the entry-points of the dynamic library.
godot_gdnative_init!();
godot_nativescript_init!(init);
godot_gdnative_terminate!();

pub struct YourState {
    pub count: i32,
}

impl State for YourState {
    fn on_push(&mut self, data: &mut StateData, resources: &mut Resources) {
        // Retrieve our data from Models<T>
        let models = resources.get::<Models<i32>>().unwrap();
        let square = models.data_from_alias("StringKey").unwrap();

        // Insert adds an entity to the world
        data.world.insert(
            (),
            (0..1).map(|_| (
                // What components we want our entity to have
                GDSpatial,
                Renderable { index: square.1, template: square.0 },
                Position::new(150f32, 150f32),
            ))
        );
    }

    fn update(&mut self, _data: &mut StateData, resources: &mut Resources) {
        godot_print!("{}", self.count);
        self.count += 1;

        if Input::godot_singleton().is_action_just_pressed(GodotString::from("pause")) {
            // Get the TransResource that allows us to send state transitions to the engine
            let sender = resources.get::<TransResource>().unwrap();
            // Send a closure that creates the Trans we want to execute
            sender.trans.try_send(Box::from(|| Trans::Push(Box::new(PauseState { })))).ok();
        }
    }
}

pub struct PauseState { }

impl State for PauseState {
    fn on_push(&mut self, data: &mut StateData, resources: &mut Resources) {
        // Retrieve our data from Models<T>
        let models = resources.get::<Models<i32>>().unwrap();
        let pause = models.data_from_alias("Pause").unwrap();

        // Insert adds an entity to the world
        data.world.insert(
            (),
            (0..1).map(|_| (
                // What components we want our entity to have
                GDSpatial,
                Renderable { index: pause.1, template: pause.0 },
                Position::new(0f32, 0f32),
            ))
        );
    }

    fn update(&mut self, _data: &mut StateData, resources: &mut Resources) {
        if Input::godot_singleton().is_action_just_pressed(GodotString::from("pause")) {
            // Get the TransResource that allows us to send state transitions to the engine
            let sender = resources.get::<TransResource>().unwrap();
            // Send a closure that creates the Trans we want to execute
            sender.trans.try_send(Box::from(|| Trans::Pop)).ok();
        }
    }
}