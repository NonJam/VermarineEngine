use crate::prelude::*;
use std::collections::HashMap;

pub fn create_systems() -> Vec<Box<dyn Schedulable>> {
    vec![
        input_system(),
        move_system(),
        // spawn_item_system(),
    ]
}

pub(crate) fn input_system() -> Box<dyn Schedulable> {
    SystemBuilder::<()>::new("InputSystem")
        .write_component::<Renderable>()
        .with_query(<(Read<TakesInput>, Write<Velocity>)>::query())
        .build(move |_commands, world, _resource, queries| {
            for (entity, (takes_input, mut vel)) in queries.iter_entities_mut(&mut *world) {
                vel.x = 0i32;
                vel.y = 0i32;

                // i_x and i_y are temporarily not used due to a lack of animation
                // let mut i_x = 0i32;
                // let mut i_y = 0i32;
                for (key, x, y) in [
                    ("move_left", -1, 0),
                    ("move_right", 1, 0),
                    ("move_up", 0, -1),
                    ("move_down", 0, 1),
                ].iter() {
                    if Input::godot_singleton().is_action_pressed(GodotString::from(key)) {
                        // i_x += x;
                        // i_y += y;
                        vel.x += *x as i32; // set these to only change if different from current?
                        vel.y += *y as i32;
                    }
                }

                // if multiple keys are held down, then negate all input
                if vel.x != 0i32 && vel.y != 0i32 {
                    vel.x = 0i32;
                    vel.y = 0i32;
                }

                // set animation
                // if let Some(mut renderable) = world.get_component_mut::<Renderable>(entity) {
                // }
            }
        })
}

pub(crate) fn move_system() -> Box<dyn Schedulable> {
    SystemBuilder::<()>::new("MoveSystem")
        .with_query(<(Write<Position>, Read<Velocity>)>::query())
        .build(move |_commands, world, _resource, queries| {
            for (_entity, (mut pos, vel)) in queries.iter_entities_mut(&mut *world) {
                // let rot = euclid::Rotation2D::<f32, euclid::UnknownUnit, euclid::UnknownUnit>::new(pos.rotation);
                // let vel = rot.transform_vector(euclid::Vector2D::<f32, euclid::UnknownUnit>::new(vel.x, vel.y));
                pos.x += vel.x as f32; // temp
                pos.y += vel.y as f32; // temp
            } 
        })
}