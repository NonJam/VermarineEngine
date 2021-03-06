use crate::prelude::*;

/// The TransResource is used to send State transitions from within States and Systems
/// ### Writing to the TransResource
/// When you have access to legion's resource set you can write to the TransResource.
/// 
/// The general places where you can do this are
/// 1. Inside of one of the state methods. All of these methods give access to Resources mutably.
/// 2. Inside of a system. Inside of a system you can access Resources mutably.
/// 
/// Example of sending inside of a state method:
/// ```
/// fn update(&mut self, data: &mut StateData, resources: &mut Resources) {
///     // Sending a push
///     let sender = resources.get::<TransResource>().unwrap();
///     sender.trans.try_send(Box::from(|| Trans::Push(Box::new( /* State goes here */ )))).ok();
/// }
/// ```
/// or for sending a pop
/// ```
/// fn update(&mut self, data: &mut StateData, resources: &mut Resources) {
///     // Sending a pop
///     let sender = resources.get::<TransResource>().unwrap();
///     sender.trans.try_send(Box::from(|| Trans::Pop)).ok();
/// }
/// ```
/// 
/// Example of sending inside of a system:
/// ```
/// SystemBuilder::<()>::new("ExampleSystem")
///     .write_resource::<TransResource>()
///     .build(move |commands, world, resources, queries| {
///         // Sending a push
///         resources.trans.try_send(Box::from(|| Trans::Push(Box::new( /* State goes here */ )))).ok();
///     })
/// ```
/// or for sending a pop
/// ```
/// SystemBuilder::<()>::new("ExampleSystem")
///     .write_resource::<TransResource>()
///     .build(move |commands, world, resources, queries| {
///         // Sending a pop
///         resources.trans.try_send(Box::from(|| Trans::Pop)).ok();
///     })
/// ```
/// If you attempt to send more than one Trans only the first one will be used
pub struct TransResource {
    pub trans: crossbeam_channel::Sender<Box<(dyn FnOnce() -> Trans + 'static)>>,
}

unsafe impl Sync for TransResource {}
unsafe impl Send for TransResource {}

pub struct VermarineEngine<T> where 
    T: Eq + std::hash::Hash + 'static {
    universe: Universe,
    states: Vec<(StateData, Box<dyn State>)>,
    pub resources: Resources,
    owner: Node,
    trans_receiver: crossbeam_channel::Receiver<Box<dyn FnOnce() -> Trans>>,
    phantom: std::marker::PhantomData<T>,
}

impl<T> VermarineEngine<T> 
    where 
    T: Eq + std::hash::Hash + 'static {
    pub fn new(owner: Node) -> Self {
        let universe = Universe::new();
        let mut resources = Resources::default();
        let (sender, receiver) = crossbeam_channel::bounded(1);
        resources.insert::<>(TransResource { trans: sender });

        VermarineEngine {
            universe,
            states: Vec::new(),
            resources,
            trans_receiver: receiver,
            phantom: std::marker::PhantomData,
            owner: owner,
        }
    }

    pub fn _ready(&mut self, _owner: Node) {
        godot_print!("Vermarine: HelloWorld");
    }

    pub fn _physics_process(&mut self, _owner: Node, _delta: f64) {
        // Run methods on states in the stack
        let state_len = self.states.len();
        for i in (0..state_len).rev() {
            let (data, state) = self.states.get_mut(i).unwrap();
            
            if i == state_len - 1 {
                state.update(data, &mut self.resources);
            } else {
                state.shadow_update(data, &mut self.resources); 
            }
        }

        // Run a transition if one was sent
        if let Ok(trans) = self.trans_receiver.try_recv() {
            let trans = trans();
            self.run_state_trans(trans);
        }

        // Sync the top of the stack's state to godot
        let state_len = self.states.len();
        if state_len > 0 {
            let state = self.states.get_mut(state_len - 1).unwrap();
            sync_state::<T>(&mut self.resources, state);
        } else {
            godot_print!("Expected a state in the stack but one was not found");
        }
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

    /// Calls the on_cover method of the state at the top of the stack,
    /// then calls the on_push method of the passed in state,
    /// then pushes the passed in state onto the state stack.
    pub fn push(&mut self, mut state: Box<dyn State>) {
        // Send on_cover event to current top of stack if there is one
        let state_len = self.states.len();
        if state_len >= 1 {
            let state = self.states.get_mut(state_len - 1).unwrap();
            state.1.on_cover(&mut state.0, &mut self.resources);
        }

        // Create new world
        let world = self.universe.create_world();
        let mut data = StateData::new(world);

        unsafe {
            // Create rootnode
            let mut rootnode = Node::new();
            let name = state.get_name(&mut data, &mut self.resources);
            rootnode.set_name(format!("State{}: {}", state_len, name).into());
            self.owner.add_child(Some(rootnode), true);
            data.rootnode = Some(rootnode);

            // Create containernode
            let mut containernode = Node::new();
            containernode.set_name("RenderablesContainer".into());
            data.rootnode.unwrap().add_child(Some(containernode), true);
            data.containernode = Some(containernode);

            // Create statenode
            if let Some(index) = state.is_node(&mut data, &mut self.resources) {
                let models = self.resources.get::<Models<T>>().unwrap();
                if let Some(packed_scene) = (*models).scene_from_index(index) {
                    let instance = packed_scene.instance(0).unwrap().cast::<Node>().unwrap();
                    rootnode.add_child(Some(instance), true);
                    data.statenode = Some(instance);
                }
            }
        }

        // Actually push state onto the stack
        state.on_push(&mut data, &mut self.resources);
        self.states.push((data, state));
    }

    /// Calls the on_pop method of the state at the top of the stack,
    /// then calls the on_uncover method of the state below the state at the top of the stack,
    /// then pops the state at the top of the stack.
    pub fn pop(&mut self) {
        let state_len = self.states.len();

        if let Some(state) = self.states.last_mut() {            
            state.1.on_pop(&mut state.0, &mut self.resources);

            if let Some(node) = state.0.rootnode {
                unsafe { node.free(); }
            }

            if state_len >= 2 {
                let state = self.states.get_mut(state_len - 2).unwrap();
                state.1.on_uncover(&mut state.0, &mut self.resources);
            }
            self.states.pop();
        }
    }

    /// Swaps the topmost state with the passed in state.
    /// This is equivelent to calling pop then push.
    pub fn switch(&mut self, state: Box<dyn State>) {
        self.pop();
        self.push(state);
    }

    /// Replaces the entire stack with the passed in state.
    /// This is equivelent to calling new_stack() with only one state.
    pub fn replace(&mut self, state: Box<dyn State>) {
        self.new_stack(vec![state]);
    }

    /// Pops all the states off the stack without calling on_uncover.
    /// This means that calling new_stack is NOT the same as popping all of the states as new_stack does not allow any of the states to run code when they are popped.
    /// Then calls push() for each of the states passed in
    pub fn new_stack(&mut self, states: Vec<Box<dyn State>>) {
        while let Some(mut popped) = self.states.pop() {
            popped.1.on_pop(&mut popped.0, &mut self.resources);
        }

        for state in states.into_iter() {
            self.push(state);
        }
    }

    /// Executes a set of Trans in sequential order
    pub fn sequence(&mut self, sequence: Vec<Trans>) {
        for trans in sequence.into_iter() {
            self.run_state_trans(trans);
        }
    }
}

pub(crate) fn get_animator<T>(node: Node) -> Option<T> 
    where T: GodotObject {
    
    let mut i = 0;
    while let Some(child_node) = unsafe { node.get_child(i) } {
        if let Some(child_node) = unsafe { child_node.cast::<T>() } {
            return Some(child_node);
        }
        i += 1;
    }
    None
}

pub(crate) fn sync_state<T>(resources: &mut Resources, state: &mut (StateData, Box<(dyn State)>)) 
    where
    T: Eq + std::hash::Hash + 'static {
    
    let models = resources.get::<Models<T>>().unwrap();

    // Sync renderable tree
    let query = <Write<Renderable>>::query()
        .filter(changed::<Renderable>());
    for mut renderable in query.iter_mut(&mut state.0.world) {
        sync_renderable_recursive(&mut state.0.containernode.unwrap(), &mut renderable, &models);
    }

    // Sync entity position to renderable tree root
    let query = <(Read<Position>, Write<Renderable>)>::query()
        .filter(changed::<Position>());
    for (pos, mut renderable) in query.iter_mut(&mut state.0.world) {
        sync_transform_to_node(&pos, renderable.container_node.unwrap());
    }

    // Free godot nodes whos respective entities have been disposed
    for event in state.0.receiver.try_iter() {
        use legion::event::Event::*;
        match event {
            EntityRemoved(e, _) => {
                if let None = state.0.world.get_component::<Renderable>(e) {
                    if let Some(node) = state.0.node_lookup.get_mut(&e) {
                        unsafe { node.free(); }
                        state.0.node_lookup.remove(&e);
                    }
                }
            },
            EntityInserted(e, _) => {
                if let Some(renderable) = state.0.world.get_component::<Renderable>(e) {
                    if let Some(_) = state.0.world.get_component::<Position>(e) {
                        state.0.node_lookup.insert(e, renderable.container_node.unwrap());
                    }
                } 
            }
            _ => { }
        }
    }
}

pub(crate) fn sync_renderable_recursive<T>(parent: &mut Node, renderable: &mut Renderable, models: &Models<T>)
    where
    T: Eq + std::hash::Hash + 'static {
    // Create container node
    if let None = renderable.container_node {
        unsafe {
            let mut node = Node2D::new().cast::<Node>().unwrap();
            node.set_name("Renderable".into());
            parent.add_child(Some(node), true);
            renderable.container_node = Some(node);
        }
    }
    
    // Create children container
    if let None = renderable.children_node {
        unsafe {
            let mut node = Node2D::new().cast::<Node>().unwrap();
            node.set_name("Children".into());
            renderable.container_node.unwrap().add_child(Some(node), true);
            renderable.children_node = Some(node);
        }
    }

    // Free renderable node if dirty
    if let (Some(spatial), Some(node)) = (renderable.spatial, renderable.renderable_node) {
        if spatial.is_id_dirty(&renderable) {
            unsafe { node.free(); }
        }
    }

    // Instance node 
    if renderable.renderable_node.is_none() && 
        renderable.renderable_id.is_some() && 
        renderable.template.is_some() 
    {
        unsafe {
            let scene = models.scene_from_index(renderable.renderable_id.unwrap()).unwrap();
            let mut instance = scene.instance(0).unwrap().cast::<Node>().unwrap();
            instance.set_name("Node".into());
            renderable.container_node.unwrap().add_child(Some(instance), true);
            renderable.renderable_node = Some(instance);
            if let Some(_) = renderable.spatial {
                renderable.spatial = Some(GDSpatial { prev_id: renderable.renderable_id, prev_pos: None });
            }
        }
    }

    // Update visibility
    unsafe {
        if let Some(_) = renderable.spatial {
            renderable.container_node.unwrap().cast::<Node2D>().unwrap().set_visible(true);
        } else {
            renderable.container_node.unwrap().cast::<Node2D>().unwrap().set_visible(false);
            return;
        }
    }

    // If our current renderable actually has stuff to render
    if let Some(node) = renderable.renderable_node {
        // Sync position to childrens parent node and to renderable node
        if renderable.spatial.unwrap().is_pos_dirty(&renderable) {
            sync_transform_to_node(&renderable.transform, node);
            if let Some(node) = renderable.children_node {
                sync_transform_to_node2d(&renderable.transform, unsafe{node.cast::<Node2D>().unwrap()});
            }
            renderable.spatial = Some(GDSpatial { prev_id: renderable.renderable_id, prev_pos: Some(renderable.transform)});
        }

        // Animations
        match renderable.template.unwrap() {
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

    // Delete orphans
    while let Some(orphan) = renderable.orphans.pop() {
        if let Some(orphan) = orphan {
            unsafe { orphan.free(); }
        }
    }

    // Call recursively on children
    for child in renderable.children.iter_mut() {
        let mut res = None;
        if child.container_node.is_none() {
            // Try find a parent node
            for node in renderable.children_containers.iter() {
                unsafe { 
                    if node.get_child_count() < 500 {
                        res = Some(*node);
                        break;
                    }
                }
            }
    
            // Make parent node if we couldnt find one
            if let None = res {
                unsafe {
                    res = Some(Node2D::new().cast::<Node>().unwrap());
                    res.unwrap().set_name("NodeBatch".into());
                    renderable.children_node.unwrap().add_child(res, true);
                }
                renderable.children_containers.push(res.unwrap());
            }
        } else {
            unsafe {
                res = Some(child.container_node.unwrap().get_parent().unwrap());
            }
        }

        //sync_renderable_recursive(&mut renderable.children_node.unwrap(), child, models)
        sync_renderable_recursive(&mut res.unwrap(), child, models);
    }
}

pub(crate) fn sync_transform_to_node(pos: &Position, node: Node) {
    if let Some(spatial) = unsafe { node.cast::<Spatial>() } {
        sync_transform_to_spatial(pos, spatial);
    } else if let Some(node2d) = unsafe { node.cast::<Node2D>() } {
        sync_transform_to_node2d(pos, node2d);
    }
}

pub(crate) fn sync_transform_to_spatial(pos: &Position, mut spatial: Spatial) {
    // Position
    let mut transform = unsafe { spatial.get_translation() };
    transform.x = pos.x as f32;
    transform.z = pos.y as f32;
    unsafe { spatial.set_translation(transform) };

    // Rotation
    let mut rotation = unsafe { spatial.get_rotation() };
    rotation.y = pos.rotation.get();
}

pub(crate) fn sync_transform_to_node2d(pos: &Position, mut node2d: Node2D) {
    let mut position = unsafe { node2d.get_position() };
    position.x = pos.x as f32;
    position.y = pos.y as f32;
    unsafe { node2d.set_position(position) };
    unsafe { node2d.set_rotation(pos.rotation.get() as f64) };
}
/// Takes a path to a scene prepends res://scenes/ and appends .tscn then attempts to load the scene
/// 
/// this path is CAPS SENSITIVE it is EXTREMELY important that your scenes folder is ALL lowercase and your specified path is correctly cased or else it WILL NOT WORK ON LINUX
/// 
/// # Errors
/// 
/// This can return an error if there was not a scene at the specified file path
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

/// Takes a path to a scene prepends res://scenes/ and appends .tscn then attempts to load the scene
///
/// this path is CAPS SENSITIVE it is EXTREMELY important that your scenes folder is ALL lowercase and your specified path is correctly cased or else it WILL NOT WORK ON LINUX
/// 
/// # Panics
/// 
/// This can panic if there was not a scene at the specified file path
pub fn load_scene(scene_path: &str) -> PackedScene {
    match try_load_scene(scene_path) {
        Err(e) => panic!(e),
        Ok(template) => template,
    }
}