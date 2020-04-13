use crate::prelude::*;
use std::collections::HashMap;

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

pub struct RPopsEngine<T> where 
    T: Eq + std::hash::Hash + 'static {
    pub universe: Universe,
    pub world: LWorld,
    pub resources: Resources,
    pub executor: Executor,

    pub event_receiver: crossbeam_channel::Receiver<legion::event::Event>,
    pub spatials: HashMap<Entity, Node>,
    phantom: std::marker::PhantomData<T>,
}

impl<T> RPopsEngine<T> 
    where 
    T: Eq + std::hash::Hash {
    pub fn new(_owner: Node) -> Self {
        let universe = Universe::new();
        let mut world = universe.create_world();        
        let resources = Resources::default();
        let executor = Executor::new(vec![]);

        let (sender, receiver) = crossbeam_channel::unbounded();
        world.subscribe(sender, any());

        RPopsEngine {
            universe,
            world,
            resources,
            executor,
            event_receiver: receiver,
            spatials: HashMap::new(),
            phantom: std::marker::PhantomData,
        }
    }

    pub fn _ready(&mut self, _owner: Node) {
    }

    pub fn _physics_process(&mut self, mut owner: Node, _delta: f64) {
        self.executor.execute(&mut self.world, &mut self.resources);

        // Add and remove entities from hashmap
        for event in self.event_receiver.try_iter() {
            match event {
                legion::event::Event::EntityRemoved(e, _) => { 
                    if let None = self.world.get_component::<GDSpatial>(e) {
                        // Remove from hashmap
                        if let Some(node) = self.spatials.get_mut(&e) {
                            unsafe { node.free() };
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
                                if let Some(models) = self.resources.get::<Models<T>>() {
                                    if let Some(Template::Scene(packed_scene)) = (*models).get_model(renderable.model) {
                                        unsafe {
                                            let mut instance = packed_scene.instance(0).unwrap().cast::<Node>().unwrap();
                                            instance.set_name(GodotString::from_str("Node"));
                                            owner.add_child(Some(instance), true);
                                            self.spatials.insert(e, instance);
                                            godot_print!("Started syncing from entity: {:?} to node", e.index());   
                                        }
                                    }
                                }
                            }
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
            if let Some(node) = self.spatials.get_mut(&entity) {
                // Calls to godot are inherently unsafe
                if let Some(mut spatial) = unsafe { node.cast::<Spatial>() } {
                    let mut transform = unsafe { spatial.get_translation() };
                    transform.x = pos.x as f32;
                    transform.y = pos.y as f32;
                    unsafe { spatial.set_translation(transform) };
                } else if let Some(mut node2D) = unsafe { node.cast::<Node2D>() } {
                    let mut position = unsafe { node2D.get_position() };
                    position.x = pos.x as f32;
                    position.y = pos.y as f32;
                    unsafe { node2D.set_position(position) };
                }
            }
        }
    }

    pub fn _input(_owner: Node, _event: Option<InputEvent>) {
    }

    pub fn set_systems(&mut self, systems: Vec<Box<dyn Schedulable>>) {
        self.executor = Executor::new(systems);
    }
}