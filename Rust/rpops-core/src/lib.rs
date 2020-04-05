use gdnative::*;
use legion::prelude::*;

/// The HelloWorld "class"
#[derive(NativeClass)]
#[inherit(Spatial)]
pub struct RPopsInstance;

// __One__ `impl` block can have the `#[methods]` attribute, which will generate
// code to automatically bind any exported methods to Godot.
#[methods]
impl RPopsInstance {
    
    /// The "constructor" of the class.
    fn _init(_owner: Spatial) -> Self {
        RPopsInstance
    }
    
    // In order to make a method known to Godot, the #[export] attribute has to be used.
    // In Godot script-classes do not actually inherit the parent class.
    // Instead they are"attached" to the parent object, called the "owner".
    // The owner is passed to every single exposed method.
    #[export]
    fn _ready(&mut self, _owner: Spatial) {
    
    }

    #[export]
    fn _physics_process(&mut self, mut owner: Spatial, delta: f64) {
    
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