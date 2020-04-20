use crate::prelude::*;
use std::collections::HashMap;

pub fn try_load_scene(scene_path: &str) -> Result<PackedScene, String> {
    if let Some(scene) = ResourceLoader::godot_singleton().load(
        GodotString::from_str(format!("res://scenes/{}.tscn", scene_path)),
        GodotString::from_str("PackedScene"),
        false,
    ) {
        if let Some(scene) = scene.cast::<PackedScene>() {
            return Ok(scene);
        } else {
            return Err(format!("Could not cast {} to PackedScene", scene_path));
        }
    } else {
        return Err(format!("Could not find {}", scene_path));
    }
}

pub fn load_scene(scene_path: &str) -> PackedScene {
    match try_load_scene(scene_path) {
        Err(e) => panic!(e),
        Ok(template) => template,
    }
}

pub struct Wrapper<T> {
    pub inner: T
}

unsafe impl<T> Sync for Wrapper<T> {}
unsafe impl<T> Send for Wrapper<T> {}

pub struct RPopsEngine<T> where 
    T: Eq + std::hash::Hash + 'static {
    pub universe: Universe,
    pub world: LWorld,
    pub resources: Resources,
    pub executor: Executor,

    pub event_receiver: crossbeam_channel::Receiver<legion::event::Event>,
    phantom: std::marker::PhantomData<T>,
}

impl<T> RPopsEngine<T> 
    where 
    T: Eq + std::hash::Hash {
    pub fn new(_owner: Node) -> Self {
        let universe = Universe::new();
        let mut world = universe.create_world();        
        let mut resources = Resources::default();
        resources.insert(Wrapper { inner: HashMap::<Entity, Node>::new() } );
        let executor = Executor::new(vec![]);

        let (sender, receiver) = crossbeam_channel::unbounded();
        world.subscribe(sender, any());

        RPopsEngine {
            universe,
            world,
            resources,
            executor,
            event_receiver: receiver,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn _ready(&mut self, _owner: Node) {
        godot_print!("HelloWorld");
    }

    pub fn _physics_process(&mut self, mut owner: Node, _delta: f64) {
        self.executor.execute(&mut self.world, &mut self.resources);

        // Add and remove entities from hashmap
        for event in self.event_receiver.try_iter() {
            match event {
                legion::event::Event::EntityRemoved(e, _) => { 
                    if let None = self.world.get_component::<GDSpatial>(e) {
                        if let Some(mut wrapped) = self.resources.get_mut::<Wrapper<HashMap<Entity, Node>>>() {
                            // Remove from hashmap
                            if let Some(node) = wrapped.inner.get_mut(&e) {
                                unsafe { node.free() };
                                wrapped.inner.remove(&e);
                                godot_print!("Stopped syncing from entity: {:?} to node", e.index())
                            }
                        }
                    }
                },
                legion::event::Event::EntityInserted(e, _) => { 
                    if let Some(_) = self.world.get_component::<GDSpatial>(e) {
                        // Add to hashmap if not already in there
                        if let Some(mut wrapped) = self.resources.get_mut::<Wrapper<HashMap<Entity, Node>>>() {
                            if !wrapped.inner.contains_key(&e) {
                                if let Some(renderable) = self.world.get_component::<Renderable>(e) {
                                    if let Some(models) = self.resources.get::<Models<T>>() {
                                        if let Some(packed_scene) = (*models).scene_from_index(renderable.index) {
                                            unsafe {
                                                let mut instance = packed_scene.instance(0).unwrap().cast::<Node>().unwrap();
                                                instance.set_name(GodotString::from_str("Node"));
                                                owner.add_child(Some(instance), true);
                                                wrapped.inner.insert(e, instance);
                                                godot_print!("Started syncing from entity: {:?} to node", e.index());   
                                            }
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
        if let Some(mut wrapped) = self.resources.get_mut::<Wrapper<HashMap<Entity, Node>>>() {
            for (entity, (pos, _)) in query.iter_entities(&mut self.world) {
                if let Some(node) = wrapped.inner.get_mut(&entity) {
                    // Calls to godot are inherently unsafe
                    if let Some(mut spatial) = unsafe { node.cast::<Spatial>() } {
                        // Position
                        let mut transform = unsafe { spatial.get_translation() };
                        transform.x = pos.x as f32;
                        transform.y = pos.y as f32;
                        unsafe { spatial.set_translation(transform) };

                        // Rotation
                        let mut rotation = unsafe { spatial.get_rotation() };
                        rotation.y = pos.rotation.get();
                        unsafe { spatial.set_rotation(rotation) };
                    } else if let Some(mut node2D) = unsafe { node.cast::<Node2D>() } {
                        let mut position = unsafe { node2D.get_position() };
                        position.x = pos.x as f32;
                        position.y = pos.y as f32;
                        unsafe { node2D.set_position(position) };
                        unsafe { node2D.set_rotation(pos.rotation.get() as f64) };
                    }
                }
            }
        }

        // Update animation
        let query = <(Read<GDSpatial>, Read<Renderable>)>::query();
        if let Some(mut wrapped) = self.resources.get_mut::<Wrapper<HashMap<Entity, Node>>>() {
            for (entity, (_, renderable)) in query.iter_entities(&mut self.world) {
                if let Some(node) = wrapped.inner.get_mut(&entity) {
                    if let Some(mut node) = unsafe { node.cast::<Node>() } {
                        match renderable.template {
                            Template::ASprite(state) => {
                                if let Some(mut sprite) = get_animator::<AnimatedSprite>(node) {
                                    // Update node from state
                                    unsafe {
                                        sprite._set_playing(state.playing);
                                        let gd_string = GodotString::from(state.animation);
                                        sprite.play(gd_string, false);
                                        sprite.set_flip_h(state.flip_h);
                                        sprite.set_flip_v(state.flip_v);
                                    }
                                }
                            },
                            Template::APlayer(state) => {
                                if let Some(sprite) = get_animator::<AnimationTree>(node) {
                                    // Update node from state
                                }
                            },
                            Template::ATree(state) => {
                                if let Some(sprite) = get_animator::<AnimationTree>(node) {
                                    // Update node from state
                                }
                            },
                            _ => {}
                        }
                    }
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

pub(crate) fn get_animator<T>(node: Node) -> Option<T> 
    where T: GodotObject {
    let mut i = 0;
    loop {
        if let Some(child_node) = unsafe { node.get_child(i) } {
            if let Some(child_node) = unsafe { child_node.cast::<T>() } {
                return Some(child_node);
            }
        } else {
            return None;
        }

        i += 1;
    }
    None
}