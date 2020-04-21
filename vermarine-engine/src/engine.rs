use crate::prelude::*;

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
    universe: Universe,
    states: Vec<(StateData, Box<dyn State>)>,
    pub resources: Resources,
    owner: Node,
    phantom: std::marker::PhantomData<T>,
}

impl<T> RPopsEngine<T> 
    where 
    T: Eq + std::hash::Hash + 'static {
    pub fn new(owner: Node) -> Self {
        let universe = Universe::new();
        let resources = Resources::default();

        RPopsEngine {
            universe,
            states: Vec::new(),
            resources,
            phantom: std::marker::PhantomData,
            owner: owner,
        }
    }

    pub fn _ready(&mut self, _owner: Node) {
        godot_print!("HelloWorld");
    }

    pub fn _physics_process(&mut self, _owner: Node, _delta: f64) {
        // Run methods on states in the stack
        let state_len = self.states.len();
        for i in 0..state_len {
            let (data, state) = self.states.get_mut(i).unwrap();
            
            if i == state_len - 1 {
                let trans = state.update(data, &mut self.resources);
                self.run_state_trans(trans);
            } else {
                state.shadow_update(data, &mut self.resources); 
            }
        }

        let state_len = self.states.len();
        let state = self.states.get_mut(state_len - 1).unwrap();

        sync_state_to_godot::<T>(&mut self.resources, state);
    }

    pub fn _input(_owner: Node, _event: Option<InputEvent>) {
    }

    pub(crate) fn run_state_trans(&mut self, trans: Trans) {
        match trans {
            Trans::None => {},
            Trans::Push(state) => { self.push(state) },
            Trans::Pop => { self.pop() },
            Trans::Switch(state) => { self.switch(state) },
            Trans::Replace(state) => { self.replace(state) },
            Trans::NewStack(stack) => { self.new_stack(stack) },
            Trans::Sequence(sequence) => { self.sequence(sequence) },
            Trans::Quit => {},
        }
    }

    pub fn push(&mut self, mut state: Box<dyn State>) {        
        let state_len = self.states.len();
        if state_len >= 1 {
            let state = self.states.get_mut(state_len - 1).unwrap();
            state.1.on_cover(&mut state.0, &mut self.resources);
        }

        let world = self.universe.create_world();
        let mut data = StateData::new(world);

        let mut node = Node::new();
        unsafe {
        node.set_name(GodotString::from_str("StateNode"));
        self.owner.add_child(Some(node), true);
        data.node = Some(node);
        }

        state.on_push(&mut data, &mut self.resources);
        self.states.push((data, state));
    }

    pub fn pop(&mut self) {
        let state_len = self.states.len() * 1;

        if let Some(state) = self.states.last_mut() {            
            state.1.on_pop(&mut state.0, &mut self.resources);

            if let Some(node) = state.0.node {
                unsafe { node.free(); }
            }

            if state_len >= 2 {
                let state = self.states.get_mut(state_len - 2).unwrap();
                state.1.on_uncover(&mut state.0, &mut self.resources);
            }
            self.states.pop();
        }
    }

    pub fn switch(&mut self, state: Box<dyn State>) {
        self.pop();
        self.push(state);
    }

    pub fn replace(&mut self, state: Box<dyn State>) {
        self.new_stack(vec![state]);
    }

    pub fn new_stack(&mut self, mut states: Vec<Box<dyn State>>) {
        while let Some(mut popped) = self.states.pop() {
            popped.1.on_pop(&mut popped.0, &mut self.resources);
        }

        while let Some(popped) = states.pop() {
            self.push(popped);
        }
    }

    pub fn sequence(&mut self, sequence: Vec<Trans>) {
        for trans in sequence.into_iter() {
            self.run_state_trans(trans);
        }
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
}

pub(crate) fn sync_state_to_godot<T>(resources: &mut Resources, state: &mut (StateData, Box<(dyn State)>)) 
    where 
    T: Eq + std::hash::Hash + 'static {
    // Add and remove entities from hashmap
    for event in state.0.receiver.try_iter() {
        match event {
            legion::event::Event::EntityRemoved(e, _) => { 
                if let None = state.0.world.get_component::<GDSpatial>(e) {
                    // Remove from hashmap
                    if let Some(node) = state.0.node_lookup.get_mut(&e) {
                        unsafe { node.free() };
                        state.0.node_lookup.remove(&e);
                        godot_print!("Stopped syncing from entity: {:?} to node", e.index())
                    }
                }
            },
            legion::event::Event::EntityInserted(e, _) => { 
                if let Some(_) = state.0.world.get_component::<GDSpatial>(e) {
                    // Add to hashmap if not already in there
                    if !state.0.node_lookup.contains_key(&e) {
                        if let Some(renderable) = state.0.world.get_component::<Renderable>(e) {
                            if let Some(models) = resources.get::<Models<T>>() {
                                if let Some(packed_scene) = (*models).scene_from_index(renderable.index) {
                                    unsafe {
                                        let mut instance = packed_scene.instance(0).unwrap().cast::<Node>().unwrap();
                                        instance.set_name(GodotString::from_str("Node"));
                                        state.0.node.unwrap().add_child(Some(instance), true);
                                        state.0.node_lookup.insert(e, instance);
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
    for (entity, (pos, _)) in query.iter_entities(&mut state.0.world) {
        if let Some(node) = state.0.node_lookup.get_mut(&entity) {
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
            } else if let Some(mut node2d) = unsafe { node.cast::<Node2D>() } {
                let mut position = unsafe { node2d.get_position() };
                position.x = pos.x as f32;
                position.y = pos.y as f32;
                unsafe { node2d.set_position(position) };
                unsafe { node2d.set_rotation(pos.rotation.get() as f64) };
            }
        }
    }

    // Update animation
    let query = <(Read<GDSpatial>, Read<Renderable>)>::query();
    for (entity, (_, renderable)) in query.iter_entities(&mut state.0.world) {
        if let Some(node) = state.0.node_lookup.get_mut(&entity) {
            if let Some(node) = unsafe { node.cast::<Node>() } {
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
                    Template::APlayer(_state) => {
                        if let Some(_sprite) = get_animator::<AnimationTree>(node) {
                            // Update node from state
                        }
                    },
                    Template::ATree(_state) => {
                        if let Some(_sprite) = get_animator::<AnimationTree>(node) {
                            // Update node from state
                        }
                    },
                    _ => {}
                }
            }
        }
    }
}
