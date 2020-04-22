//! Vermarine is a game engine written in rust that attempts to use godot-rust to cut down on engine work while still retaining the performance and safety of rust
//! 
//! # General Overview of how to use Vermarine
//! The core of vermarine revolves around the state stack and legion ecs.
//! 
//! Each state has various methods that can be optionally implemented that will be automatically called from the engine once pushed to the stack.
//! In order to transition between different states on the stack there is a Trans enum to select which of the transitions to perform.
//! 
//! ### Getting your project setup
//! 1. Create a new crate and follow the godot-rust setup instructions https://github.com/GodotNativeTools/godot-rust until you have a working HelloWorld project
//! 2. Add vermarine-engine to your dependencies in cargo.toml
//! ```
//! [dependencies]
//! gdnative = "0.8"
//! vermarine-engine = { path = "PATH-HERE" }
//! ```
//! 3. Add the following code to your lib.rs to gain access to all of the engines types
//! ```
//! use vermarine_engine::prelude::*;
//! ```
//! 4. Add a VermarineEngine variable to your HelloWorld struct like so
//! ```
//! pub struct HelloWorld {
//!     engine: VermarineEngine<Renderables>,
//! }
//! ```
//! for now don't worry about the \<Renderables>, we'll get to that later
//! 
//! 5. Fix your _init(_owner: Node) method! Our struct has a variable in it now so we need to set that.
//! ```
//! fn _init(owner: Node) -> Self {
//!     let mut instance = HelloWorld { engine: VermarineEngine::<Renderables>::new(owner) };
//!
//!     // This is usually where we would add our initial state to the stack and insert any Resources
//!     // However since we're focussing on initial project setup we'll come back to this later
//!     // --snip
//! 
//!     instance
//! }
//! ```
//! 6. Hook up the godot methods to the engine
//! ```
//! // We need to change this from &self to &mut self
//! #[export]
//! fn _ready(&mut self, owner: Node) {
//!     // The engine has code that needs to be run for initialisation
//!     self.engine._ready(owner); 
//! }
//! 
//! // We need to add a physics_process method for our engine
//! #[export]
//! fn _physics_process(&mut self, owner: Node, delta: f64) {
//!    self.engine._physics_process(owner, delta);
//! }
//! ```
//! 
//! ### Getting started with your first state
//! Adding your first state to the stack is a bit unique as it's the only time you have to do it from outside of a system or state (More on this later)
//! The first thing to do is create a struct for us to implement the state trait on
//! 
//! ```
//! pub struct YourState { 
//!     pub count: i32,
//! }
//! 
//! impl State for YourState {
//!     fn update(&mut self, data: &mut StateData, resources: &mut Resources) {
//!         godot_print!("{}", count);
//!         count += 1;
//!     }
//! }
//! ```
//! In this example state we implement the update method which only gets called when the state is at the top of the stack.
//! Inside the update method we access some data specific to our state, print it to godot's console and then increment it by one.
//! 
//! If you try to run your project now you'll probably find that nothing happens!
//! This is because we need to push the state onto the stack so that the engine can call its methods.
//! To do this we will need to add some code to our init method in our rust native script class that we made in the last section
//! ```
//! fn _init(owner: Node) -> Self {
//!     // --snip
//! 
//!     // This pushes the state onto the stack
//!     instance.engine.push(Box::from(YourState { count: 0 }));
//! 
//!     // --snip
//! }
//! ```
//! 
//! There are a few other methods you can implement on your state, we won't go into too much definition here as they have more in depth
//! explanations in their own documentation.
//! 
//! 1.) update - This method gets called on a state when it is on the top of the stack
//! 
//! 2.) shadow_update - This method gets called on a state when it is not on the top of the stack
//! 
//! 3.) on_cover - This method gets called on a state when another state is pushed over it
//! 
//! 4.) on_push - This method gets called on a state when it is pushed onto the stack
//! 
//! 5.) on_uncover - This method gets called on a state when the state ontop of it in the stack is popped
//! 
//! 6.) on_pop - This method gets called on a state when it is popped off of the stack
//! 
//! 7.) get_name - This method is used in various debug information to identify a state and has no functional effect on the running of your state
//! 
//! 8.) is_node - This method is slightly more complicated to explain and deserves its own section so we'll only give a brief explanation here.
//! is_node() is used to specify a Models<T> to instance alongside the state, it can be accessed via data.statenode (for example usage see BaseState's shadow_update and is_node methods in scene-example).
//! 
//! ### Writing to the TransResource
//! When you have access to legion's resource set you can write to the TransResource.
//! 
//! The general places where you can do this are
//! 1. Inside of one of the state methods. All of these methods give access to Resources mutably.
//! 2. Inside of a system. Inside of a system you can access Resources mutably.
//! 
//! Example of sending inside of a state method:
//! ```
//! fn update(&mut self, data: &mut StateData, resources: &mut Resources) {
//!     // Sending a push
//!     let sender = resources.get::<TransResource>().unwrap();
//!     sender.trans.try_send(Box::from(|| Trans::Push(Box::new( /* State goes here */ )))).ok();
//! }
//! ```
//! or for sending a pop
//! ```
//! fn update(&mut self, data: &mut StateData, resources: &mut Resources) {
//!     // Sending a pop
//!     let sender = resources.get::<TransResource>().unwrap();
//!     sender.trans.try_send(Box::from(|| Trans::Pop)).ok();
//! }
//! ```
//! 
//! Example of sending inside of a system:
//! ```
//!     SystemBuilder::<()>::new("ExampleSystem")
//!         .write_resource::<TransResource>()
//!         .build(move |commands, world, resources, queries| {
//!             // Sending a push
//!             resources.trans.try_send(Box::from(|| Trans::Push(Box::new( /* State goes here */ )))).ok();
//!         })
//! ```
//! or for sending a pop
//! ```
//!     SystemBuilder::<()>::new("ExampleSystem")
//!         .write_resource::<TransResource>()
//!         .build(move |commands, world, resources, queries| {
//!             // Sending a pop
//!             resources.trans.try_send(Box::from(|| Trans::Pop)).ok();
//!         })
//! ```
//! If you attempt to send more than one Trans only the first one will be used

mod engine;
mod components;
mod models;
mod state;

pub use crate::engine::*;
pub use crate::components::*;
pub use crate::models::*;
pub use crate::state::*;

pub mod prelude {
    pub use crate::engine::*;
    pub use crate::components::*;
    pub use crate::models::*;
    pub use crate::state::*;
    pub use rand;
    pub use rand::Rng;
    pub use euclid;
    pub use gdnative::*;
    pub use legion::prelude::*;
    pub use legion::prelude::World as LWorld;
    pub use crossbeam_channel;
}