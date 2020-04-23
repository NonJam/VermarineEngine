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
//!     engine: VermarineEngine<i32>,
//! }
//! ```
//! for now don't worry about the \<i32>, we'll get to that later
//! 
//! 5. Fix your _init(_owner: Node) method! Our struct has a variable in it now so we need to set that.
//! ```
//! fn _init(owner: Node) -> Self {
//!     let mut instance = HelloWorld { engine: VermarineEngine::<i32>::new(owner) };
//!
//!     // This is usually where we would add our initial state to the stack and insert any Resources
//!     // However since we're focusing on initial project setup we'll come back to this later
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
//!         godot_print!("{}", self.count);
//!         self.count += 1;
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
//!     let mut instance = HelloWorld { engine: VermarineEngine::<Renderables>::new(owner) };
//! 
//!     // This pushes the state onto the stack
//!     instance.engine.push(Box::new(YourState { count: 0 }));
//! 
//!     instance
//! }
//! ```
//! 
//! There are a few other methods you can implement on your state, we won't go into too much detail here as they have more in depth
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
//! 
//! ### How to get things rendering
//! 
//! So now that we have a hello world setup and a state on the stack to run our code in you might be wondering "How do I get stuff on the screen?"
//! 
//! The answer goes back to the <i32> we saw when we setup VermarineEngine inside of our HelloWorld struct. Internally Vermarine draws to the screen by instancing
//! scenes in godot and letting godot draw them for us, this means that we have to store a set of all the scenes we can instance in godot. Vermarine handles this
//! by having a Models<T> resource (We'll get to what the T is for later). The generic that VermarineEngine takes is so that it knows what the T of your Models<T>
//! resource is, this means that it's important that when you insert your Models<T> resource you set the T to be the same as whatever you set VermarineEngine<T> to.
//! 
//! The actual use for the <T> in Models<T> is for a custom lookup of your model data. When you insert a model into Models<T> you'll be able to set some optional
//! keys for your model so that you can access it again, the <T> is a generic that lets you pick a type for use as a key, the three ways to access your model
//! after inserting are:
//! 
//! -- NOTE: If you do not set Models<T> to the same T as VermarineEngine<T> the engine will not be able to find the Models<T> resource 
//! and as such nothing will get drawn.
//! 
//! 1. Looking up the model on the Models<T> resource with a key of type T
//! 2. Looking up the model on the Models<T> resource with a key of type &'static str
//! 3. Looking up the model on the Models<T> resource with a key of type usize
//! 
//! the methods used for this are respectively:
//! ```
//! Models.data_from_t()
//! Models.data_from_alias()
//! Models.data_from_index()
//! ```
//! 
//! -- NOTE: the usize key is not specifiable on model insertion, the usize key is the index of your model inside of the underlying storage type of Models<T>,
//! there are a few helper methods for retrieving this index from either of the other two keys.
//! 
//! Now that we know about the Models<T> resource let's get on with inserting it with some model data
//! 
//! 1. First thing we're going to do is create an instance of Models<T>
//! ```
//! fn _init(owner: Node) -> Self {
//!     let mut instance = HelloWorld { engine: VermarineEngine::<i32>::new(owner)};
//!
//!     // Once again you can replace <i32> with any <T> that you want as long as it is the same as VermarineEngine::<T>
//!     // Set up Models<T> resource
//!     let mut models = Models::<i32>::default();
//!     instance.engine.resources.insert(models);
//!
//!     instance.engine.push(Box::new(YourState { count: 0 }));
//! 
//!     instance
//! }
//! ```
//! 
//! 2. Next up we need to insert some model data into our Models<T> resource so to do that let's go into our godot project and create a /scenes/ folder
//! 
//! -- NOTE: It is important that this folder is named EXACTLY "scenes" and in all lowercase or the load_scene() calls to Models<T> will not find anything
//! 
//! 3. Next lets create a new scene and call it "square", select 2D Scene as the root node and then add a Sprite node as a child of our root node.
//! 
//! 4. Then click on the Sprite node go over to the inspector on the right click on the Texture variable's value on the right which is currently labelled as [empty],
//! this should open a dropdown with a load button at the bottom. Click load and then select icon.png, you should see the godot icon in your scene now.
//! 
//! 5. Hit ctrl-s and then tab back into your editor of choice and lets load this scene into vermarine.
//! 
//! 6. To insert a scene into our Models<T> resource we need to call .insert() on our Models<T> like this:
//! ```
//! models.insert(alias: ..., t_key: ..., scene: ..., template: ...);
//! ```
//! This function call looks fairly unintuitive so we'll go over what each of the arguments do, if you want a more in depth explanation see the docs for the Models<T> type
//! 
//! alias: This is the optional &'static str key that we talked about earlier, if you want to specify a key then pass in Some("Your key") if not then just pass in None.
//! 
//! t_key: This is the optional key for whatever T you specified in Models<T>, same syntax as alias if you want to specify a key then pass in
//! Some(/* instance of T here */) and if you dont want to specify a key then just pass in None
//! 
//! scene: This is the actual scene data we want to store in Models<T> to get this there's a function that will be available through the vermarine_engine::prelude::*; call.
//! You can either call load_scene() or try_load_scene(), try_load_scene will return an error if it could not find your scene wheras load_scene() will just call the panic macro.
//! These functions take one argument, a string path to your scene, the path to our scene is res://scenes/square.tscn so we'll just pass in square. If your path was
//! res://scenes/shapes/square.tscn we would pass in the path shapes/square.
//! 
//! -- NOTE: paths are caps sensitive it is EXTREMELY important that your path is IDENTICAL in every way to the actual path in godot
//! 
//! template: Template specifies what kind of scene we're storing currently this is just used for animation but since we're not using any animation yet we can just use Template::Scene
//! 
//! And finally here is the _init() method with the code we need in order to insert our Models<T> resource and add our scene to it:
//! ```
//! // Set up Models<T> resource
//! let mut models = Models::<i32>::default();
//! 
//! // This is new
//! models.insert(Some("StringKey"), None, load_scene("square"), Template::Scene);
//! 
//! instance.engine.resources.insert(models);
//! ```
//! 
//! 7. Next we need to create an entity and set the Renderable component so we can get our scene instanced.
//! To do this lets head back to YourState and implement the on_push method so that we can add an entity when our state gets pushed onto the stack
//! 
//! To implement the on_push method add this to our impl State for YourState { }
//! ```
//! fn on_push(&mut self, data: &mut StateData, resources: &mut Resources) {
//! }
//! ```
//! The first thing we want to do is get the Models<T> resource:
//! ```
//! fn on_push(&mut self, data: &mut StateData, resources: &mut Resources) {
//!     let models = resources.get::<Models<i32>>().unwrap();
//! }
//! ```
//! Then we want to get our square data from it like this:
//! ```
//! fn on_push(&mut self, data: &mut StateData, resources: &mut Resources) {
//!     let models = resources.get::<Models<i32>>().unwrap();
//!     let square = models.data_from_alias("StringKey").unwrap();
//! }
//! ```
//! And then finally we'll actually add the entity to the world:
//! ```
//! fn on_push(&mut self, data: &mut StateData, resources: &mut Resources) {
//!     // Retrieve our data from Models<T>
//!     let models = resources.get::<Models<i32>>().unwrap();
//!     let square = models.data_from_alias("StringKey").unwrap();
//!
//!     // Insert adds an entity to the world
//!     data.world.insert(
//!         (),
//!         (0..1).map(|_| (
//!             // What components we want our entity to have
//!             GDSpatial,
//!             Renderable { index: square.1, template: square.0 },
//!             Position::new(150f32, 150f32),
//!         ))
//!     );
//! }
//! ```
//! All three of the components we just added are from VermarineEngine so make sure you have a use statement somewhere ```use vermarine_engine::prelude::*;```
//! 
//! GDSpatial is a component that marks our entity as one that needs to be instanced in godot aswell. This component won't do anything if you don't have a Renderable
//! component on the entity aswell.
//! 
//! Renderable is a component that stores the usize key into Models<T> along with the Template.
//! 
//! Position is fairly self explanatory but nevertheless, position stores the position of our entity and is what is used by the engine to set the position of GDSpatials
//! 
//! 8...  That's us done! If you play the game you'll see your square.tscn at whatever position specified in the Position component

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