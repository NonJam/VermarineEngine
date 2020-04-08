use gdnative::*;
use legion::prelude::*;
use rpops_core::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Model {
    Creatures(CreatureModels),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum CreatureModels { 
    Slime,
    Zombie,
}

pub fn try_load_scene(scene_path: &str) -> Result<Template, String> {
    if let Some(scene) = ResourceLoader::godot_singleton().load(
        GodotString::from_str(format!("res://Scenes/{}.tscn", scene_path)),
        GodotString::from_str("PackedScene"),
        false,
    ) {
        if let Some(scene) = scene.cast::<PackedScene>() {
            return Ok(Template::Scene(scene));
        } else {
            return Err(format!("Could not cast {} to PackedScene", scene_path));
        }
    } else {
        return Err(format!("Could not find {}", scene_path));
    }
}

pub fn load_scene(scene_path: &str) -> Template {
    match try_load_scene(scene_path) {
        Err(e) => panic!(e),
        Ok(template) => template,
    }
}

fn add_models(mut models: Models<Model>) -> Models<Model> {
    use CreatureModels::*;

    models.insert(None, Some(Model::Creatures(Slime)), load_scene("Square"));
    models.insert(None, Some(Model::Creatures(Zombie)), load_scene("Rectangle"));

    models
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GDSpatial;

#[derive(NativeClass)]
#[inherit(Spatial)]
pub struct RPopsInstance {
    universe: Universe,
    world: LWorld,
    resources: Resources,
    executor: Executor,

    event_receiver: crossbeam_channel::Receiver<legion::event::Event>,
    spatials: HashMap<Entity, Spatial>,
}

// __One__ `impl` block can have the `#[methods]` attribute, which will generate
// code to automatically bind any exported methods to Godot.
#[methods]
impl RPopsInstance {
    
    /// The "constructor" of the class.
    fn _init(_owner: Spatial) -> Self {
        let universe = Universe::new();
        let mut world = universe.create_world();
        
        let mut resources = Resources::default();
        let models = Models::<Model>::default();
        resources.insert(add_models(models));
        
        let mut systems = vec![];
        systems.append(
            &mut create_systems()
        );
        let executor = Executor::new(systems);

        let (sender, receiver) = crossbeam_channel::unbounded();
        world.subscribe(sender, any());

        RPopsInstance {
            universe,
            world,
            resources,
            executor,
            event_receiver: receiver,
            spatials: HashMap::new(),
        }
    }
    
    // In order to make a method known to Godot, the #[export] attribute has to be used.
    // In Godot script-classes do not actually inherit the parent class.
    // Instead they are"attached" to the parent object, called the "owner".
    // The owner is passed to every single exposed method.
    #[export]
    fn _ready(&mut self, _owner: Spatial) {
        use CreatureModels::*;
        let models = self.resources.get::<Models<Model>>().unwrap();
        let slime_model_index = models.index_from_t(&Model::Creatures(Slime)).unwrap();
        let zombie_model_index = models.index_from_t(&Model::Creatures(Zombie)).unwrap();

        self.world.insert(
            (), 
            (0..1).map(|_| (Renderable { model: slime_model_index }, GDSpatial, Position { x: 0, y: 0 }, ))
        );

        self.world.insert(
            (), 
            (0..1).map(|_| (Renderable { model: zombie_model_index }, GDSpatial, Position { x: -10, y: 0 }, ))
        );

        self.world.insert(
            (),
            (0..1).map(|_| (Position { x: -10, y: 0 }, ))
        );
    }

    #[export]
    fn _physics_process(&mut self, mut owner: Spatial, _delta: f64) {
        self.executor.execute(&mut self.world, &mut self.resources);

        // Add and remove entities from hashmap
        for event in self.event_receiver.try_iter() {
            match event {
                legion::event::Event::EntityRemoved(e, _) => { 
                    if let None = self.world.get_component::<GDSpatial>(e) {
                        // Remove from hashmap
                        if let Some(spatial) = self.spatials.get_mut(&e) {
                            unsafe { spatial.free() };
                            self.spatials.remove(&e);
                            godot_print!("Stopped syncing from entity: {:?} to node", e.index())
                        }
                    }
                },
                legion::event::Event::EntityInserted(e, _) => { 
                    if let Some(_) = self.world.get_component::<GDSpatial>(e) {
                        // Add to hashmap if not already in there
                        if !self.spatials.contains_key(&e) {
                            if let Some(renderable) = self.world.get_component::<Renderable>(e) {
                                if let Some(models) = self.resources.get::<Models<Model>>() {
                                    if let Some(Template::Scene(packed_scene)) = (*models).get_model(renderable.model) {
                                        unsafe {
                                            let mut instance = packed_scene.instance(0).unwrap().cast::<Spatial>().unwrap();
                                            instance.set_name(GodotString::from_str("Node"));
                                            owner.add_child(Some(instance.to_node()), true);
                                            self.spatials.insert(e, instance);
                                        }
                                    }
                                }
                            }
                            godot_print!("Started syncing from entity: {:?} to node", e.index());
                        }
                    }
                },
                _ => (),
            };
        }
        // Query on changed components to update node positions
        let query = <(Read<Position>, Read<GDSpatial>)>::query()
            .filter(changed::<Position>());
        for (entity, (pos, _)) in query.iter_entities(&mut self.world) {
            if let Some(spatial) = self.spatials.get_mut(&entity) {
                // Calls to godot are inherently unsafe
                let mut transform = unsafe { spatial.get_translation() };
                transform.x = pos.x as f32;
                transform.y = pos.y as f32;
                unsafe { spatial.set_translation(transform) };
            }
        }
    }

    #[export]
    fn _input(&mut self, _owner: Spatial, _event: Option<InputEvent>) {
    
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