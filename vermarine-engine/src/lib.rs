//! Vermarine is a game engine written in rust that attempts to use godot-rust to cut down on engine work while still retaining the performance and safety of rust
//! 
//! # Getting started tutorial
//! This tutorial goes from an empty folder to a basic wasd movement setup with a pause menu
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
//! There are a few other methods you can implement on your state, we won't go into too much detail here as they have more in depth
//! explanations in their own documentation.
//! 
//! 1.) shadow_update - This method gets called on a state when it is not on the top of the stack
//! 
//! 2.) on_cover - This method gets called on a state when another state is pushed over it
//! 
//! 3.) on_push - This method gets called on a state when it is pushed onto the stack
//! 
//! 4.) on_uncover - This method gets called on a state when the state ontop of it in the stack is popped
//! 
//! 5.) on_pop - This method gets called on a state when it is popped off of the stack
//! 
//! 6.) get_name - This method is used in various debug information to identify a state and has no functional effect on the running of your state
//! 
//! 7.) is_node - This method is slightly more complicated to explain and deserves its own section so we'll only give a brief explanation here.
//! is_node() is used to specify a Models\<T> to instance alongside the state, it can be accessed via data.statenode (for example usage see BaseState's shadow_update and is_node methods in scene-example).
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
//! We'll go into more detail about the stack later on.
//! 
//! ### How to get things rendering
//! 
//! So now that we have a hello world setup and a state on the stack to run our code in you might be wondering "How do I get stuff on the screen?"
//! 
//! The answer goes back to the \<i32> we saw when we setup VermarineEngine inside of our HelloWorld struct. Internally Vermarine draws to the screen by instancing
//! scenes in godot and letting godot draw them for us, this means that we have to store a set of all the scenes we can instance in godot. Vermarine handles this
//! by having a Models\<T> resource (We'll get to what the T is for later). The generic that VermarineEngine takes is so that it knows what the T of your Models\<T>
//! resource is, this means that it's important that when you insert your Models\<T> resource you set the T to be the same as whatever you set VermarineEngine\<T> to.
//! 
//! The actual use for the \<T> in Models\<T> is for a custom lookup of your model data. When you insert a model into Models\<T> you'll be able to set some optional
//! keys for your model so that you can access it again, the \<T> is a generic that lets you pick a type for use as a key, the three ways to access your model
//! after inserting are:
//! 
//! -- NOTE: If you do not set Models\<T> to the same T as VermarineEngine\<T> the engine will not be able to find the Models\<T> resource 
//! and as such nothing will get drawn.
//! 
//! 1. Looking up the model on the Models\<T> resource with a key of type T
//! 2. Looking up the model on the Models\<T> resource with a key of type &'static str
//! 3. Looking up the model on the Models\<T> resource with a key of type usize
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
//! Now that we know about the Models\<T> resource let's get on with inserting it with some model data
//! 
//! 1. First thing we're going to do is create an instance of Models\<T>
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
//! 2. Next up we need to insert some model data into our Models\<T> resource so to do that let's go into our godot project and create a /scenes/ folder
//! 
//! -- NOTE: It is important that this folder is named EXACTLY "scenes" and in all lowercase or the load_scene() calls to Models\<T> will not find anything
//! 
//! 3. Next lets create a new scene and call it "square", select 2D Scene as the root node and then add a Sprite node as a child of our root node.
//! 
//! 4. Then click on the Sprite node go over to the inspector on the right click on the Texture variable's value on the right which is currently labelled as [empty],
//! this should open a dropdown with a load button at the bottom. Click load and then select icon.png, you should see the godot icon in your scene now.
//! 
//! 5. Hit ctrl-s and then tab back into your editor of choice and lets load this scene into vermarine.
//! 
//! 6. To insert a scene into our Models\<T> resource we need to call .insert() on our Models\<T> like this:
//! ```
//! models.insert(alias: ..., t_key: ..., scene: ..., template: ...);
//! ```
//! This function call looks fairly unintuitive so we'll go over what each of the arguments do, if you want a more in depth explanation see the docs for the Models<T> type
//! 
//! alias: This is the optional &'static str key that we talked about earlier, if you want to specify a key then pass in Some("Your key") if not then just pass in None.
//! 
//! t_key: This is the optional key for whatever T you specified in Models\<T>, same syntax as alias if you want to specify a key then pass in
//! Some(/* instance of T here */) and if you dont want to specify a key then just pass in None
//! 
//! scene: This is the actual scene data we want to store in Models\<T> to get this there's a function that will be available through the vermarine_engine::prelude::*; call.
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
//! The first thing we want to do is get the Models\<T> resource:
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
//!             Renderable::new(Position::default(), square.1, square.0),
//!             Position::new(150f32, 150f32),
//!         ))
//!     );
//! }
//! ```
//! All three of the components we just added are from VermarineEngine so make sure you have a use statement somewhere ```use vermarine_engine::prelude::*;```
//! 
//! Renderables: this component stores a tree like structure of renderables from the Models\<T> resource along with positions to render each renderable at. 
//! The positions specified are relative to the parent's position. In the case of the root level renderable its position is relative to the Position component attached to the entity.
//! 
//! Position is fairly self explanatory but nevertheless, position stores the position of our entity and is what is used by the engine to determine the position of an entity.
//! This component is unnecessary for drawing to the screen as if it's not found the renderables assume the entity is at 0,0.
//! 
//! 8...  That's us done! If you play the game you'll see your square.tscn at whatever position specified in the Position component
//! 
//! ### Setting up state transitions
//! 
//! One of the big things you can do with vermarine is swap states around for loading levels, pausing the game etc. So in this section we'll be adding pause functionality
//! to our counter by adding a pause menu.
//! 
//! The way we'll do this is by having YourState check for the pause button being pressed in update and if it detects it push PauseState onto the stack.
//! Then in PauseState we'll check for the pause button being pressed in update and if it it detects it pop the stack to return back to YourState.
//! 
//! 1. Lets start off by making a new scene in godot that will show some text saying "Game is paused press ESC to unpause".
//! 
//! As with square.tscn we should place this scene in /scenes/ I called my scene "pause". Once you've created your new scene add a child node of type RichTextLabel
//! then click it, go to the inspector on the right and add to the text box whatever you want it to say. Feel free to set the size of your text box and move it around
//! until you're happy with how it looks, once we're done with that let's head back into our code.
//! 
//! 2. Finally before heading back to our code lets add the keybinds for pausing/unpausing. At the top left of godot go to Project>Project Settings and then
//! at the top of the UI that opens click on the Input Map tab, type "pause" into the action text box, hit the Add button on the right, click on the + button next
//! to your action at the bottom and then select a key, I'll be using ESC. Now that we're done in godot lets head back to our code.
//! 
//! 3. Next lets load our scene into our Models\<T> resource like we did with square, add this code to your _init() method
//! ```models.insert(Some("Pause"), None, load_scene("pause"), Template::Scene);```
//! 
//! 4. Inside update make YourState check if the pause key was pressed and then send a Push transition to push PauseState onto the stack
//! ```
//! if Input::godot_singleton().is_action_just_pressed(GodotString::from("pause")) {
//!     // Get the TransResource that allows us to send state transitions to the engine
//!     let sender = resources.get::<TransResource>().unwrap();
//!     // Send a closure that creates the Trans we want to execute
//!     sender.trans.try_send(Box::from(|| Trans::Push(Box::new(PauseState { }))).ok();
//! }
//! ```
//! 
//! 5. Create a new state called PauseState that implements on_push and update
//! ```
//! pub struct PauseState { }
//! 
//! impl State for PauseState {
//!     fn on_push(&mut self, data: &mut StateData, resources: &mut Resources) {
//!
//!     }
//!
//!     fn update(&mut self, data: &mut StateData, resources: &mut Resources) {
//!
//!     }
//! }
//! ```
//! 
//! 6. In update check if "pause" key is pressed and send a Pop transition if it is
//! ```
//! if Input::godot_singleton().is_action_just_pressed(GodotString::from("pause")) {
//!     // Get the TransResource that allows us to send state transitions to the engine
//!     let sender = resources.get::<TransResource>().unwrap();
//!     // Send a closure that creates the Trans we want to execute
//!     sender.trans.try_send(Box::from(|| Trans::Pop)).ok();
//! }
//! ```
//! 
//! 7. In on_push we want to create an entity with the renderable set to our pause scene. This is pretty quickly done by copy pasting our on_push method from
//! YourState and then making some small tweaks:
//! ```
//! // Retrieve our data from Models\<T>
//! let models = resources.get::<Models<i32>>().unwrap();
//! // Changed variable name to pause and set the Key to "Pause"
//! let pause = models.data_from_alias("Pause").unwrap();
//! 
//! // Insert adds an entity to the world
//! data.world.insert(
//!     (),
//!     (0..1).map(|_| (
//!         // What components we want our entity to have
//!         Renderable::new(Position::default(), pause.1, pause.0),
//!     ))
//! );
//! ```
//! 
//! If you run this now you should find that if you press pause the counter will stop printing out, the pause text will appear and if you hit pause again it will resume.

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