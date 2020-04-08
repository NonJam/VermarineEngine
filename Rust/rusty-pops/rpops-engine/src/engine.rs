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

pub struct RPopsEngine {
    pub universe: Universe,
    pub world: LWorld,
    pub resources: Resources,
    pub executor: Executor,

    pub event_receiver: crossbeam_channel::Receiver<legion::event::Event>,
    pub spatials: HashMap<Entity, Spatial>,
}

impl RPopsEngine {
    pub fn new(_owner: Spatial) -> Self {
        let universe = Universe::new();
        let mut world = universe.create_world();
        
        let mut resources = Resources::default();
        let models = Models::<Model>::default();
        resources.insert(add_models(models));
        
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
        }
    }

    pub fn _ready(&mut self, _owner: Spatial) {
    }

    pub fn _physics_process(&mut self, mut owner: Spatial, _delta: f64) {
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

    pub fn _input(_owner: Spatial, _event: Option<InputEvent>) {
    }

    pub fn set_systems(&mut self, systems: Vec<Box<dyn Schedulable>>) {
        self.executor = Executor::new(systems);
    }
}