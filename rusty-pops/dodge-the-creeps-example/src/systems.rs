use crate::prelude::*;
use std::collections::HashMap;

pub fn create_systems() -> Vec<Box<dyn Schedulable>> {
    vec![
        input_system(),
        move_system(),
        spawn_enemy_system(),
        collider_system(),
    ]
}

pub(crate) fn input_system() -> Box<dyn Schedulable> {
    SystemBuilder::<()>::new("InputSystem")
        .write_resource::<Wrapper<HashMap<Entity, Node>>>()
        .with_query(<(Read<TakesInput>, Write<Velocity>)>::query())
        .build(move |commands, world, mut resource, queries| {
            for (entity, (takes_input, mut vel)) in queries.iter_entities_mut(&mut *world) {
                vel.x = 0f32;
                vel.y = 0f32;

                for (key, x, y) in [
                    ("move_left", -1, 0),
                    ("move_right", 1, 0),
                    ("move_up", 0, -1),
                    ("move_down", 0, 1),
                ].iter() {
                    if Input::godot_singleton().is_action_pressed(GodotString::from(key)) {
                        vel.x += *x as f32;
                        vel.y += *y as f32;
                    }
                }

                if vel.x != 0f32 || vel.y != 0f32 {
                    let vec = euclid::Vector2D::<f32, euclid::UnknownUnit>::new(vel.x as f32, vel.y as f32);
                    let vec = vec.normalize() * takes_input.speed;
                    vel.x = vec.x;
                    vel.y = vec.y;
                }

                if let Some(animator) = get_animator(entity, &mut resource.inner) {
                    match animator {
                        Animator::ASprite(mut node) => {
                            unsafe { node._set_playing(true); }
                            if vel.x != 0f32 {
                                unsafe { node.play(GodotString::from("right"), false); }
                                unsafe { node.set_flip_v(false); }
                                unsafe { node.set_flip_h(vel.x < 0f32); }
                            } else if vel.y != 0f32 {
                                unsafe { node.play(GodotString::from("up"), false); }
                                unsafe { node.set_flip_v(vel.y > 0f32); }
                            } else {
                                unsafe { node._set_playing(false); }
                            }
                        },
                        _ => {},
                    }
                }
            }
        })
}

pub(crate) fn move_system() -> Box<dyn Schedulable> {
    SystemBuilder::<()>::new("MoveSystem")
        .with_query(<(Write<Position>, Read<Velocity>)>::query())
        .build(move |commands, world, resource, queries| {
            for (entity, (mut pos, vel)) in queries.iter_entities_mut(&mut *world) {
                let rot = euclid::Rotation2D::<f32, euclid::UnknownUnit, euclid::UnknownUnit>::new(pos.rotation);
                let vel = rot.transform_vector(euclid::Vector2D::<f32, euclid::UnknownUnit>::new(vel.x, vel.y));
                pos.x += vel.x;
                pos.y += vel.y;
            } 
        })
}

pub(crate) fn spawn_enemy_system() -> Box<dyn Schedulable> {
    let mut counter: i32 = 999;
    SystemBuilder::<()>::new("SpawnEnemySystem")
        .read_resource::<Models<Renderables>>()
        .build(move |commands, world, resource, queries| {
            if counter % 1000 == 0 {
                counter = 0;

                let enemy_renderable_index = resource.index_from_t(&Renderables::Creatures(CreatureRenderables::Enemy)).unwrap();

                commands.insert(
                    (),
                    (0..1).map(move |_| {
                        
                        let mut rand = rand::thread_rng();

                        let mut position = if rand::random() {
                            // Spawn horizontal
                            let y = (rand.gen_range(0, 2) * 719) as f32;
                            let x = rand.gen_range(0, 480) as f32;
                            Position {
                                x: x,
                                y: y,
                                rotation: euclid::Angle::radians(0f32),
                            }
                        } else {
                            // Spawn vertical
                            let x = (rand.gen_range(0, 2) * 479) as f32;
                            let y = rand.gen_range(0, 720) as f32;
                            Position {
                                x: x,
                                y: y,
                                rotation: euclid::Angle::radians(0f32),
                            }
                        };

                        let new_angle = euclid::Vector2D::<f32, euclid::UnknownUnit>::new(position.x - 240f32, position.y - 360f32).angle_from_x_axis() + euclid::Angle::pi();
                        let variance = if rand::random() {
                            euclid::Angle::radians(-0.3f32)
                        } else {
                            euclid::Angle::radians(0.3f32)
                        };
                        position.rotation = new_angle + variance;
                        
                        return (
                        EnemyComp { },
                        Renderable { model: enemy_renderable_index }, 
                        GDSpatial, 
                        position, 
                        Velocity { x: 0f32, y: 0f32 },
                        Collider { width: 0.0, height: 25.0, offset_x: 14.0, offset_y: 0.0 },
                    )}
                ));
            }
            counter+=1;
        })
} 

pub(crate) fn collider_system() -> Box<dyn Schedulable> {
    SystemBuilder::<()>::new("ColliderSystem")
        .read_component::<Collider>()
        .read_component::<Position>()
        .read_component::<PlayerComp>()
        .read_component::<EnemyComp>()
        .with_query(<(Read<Position>, Read<Collider>, Read<EnemyComp>)>::query())
        .with_query(<(Read<Position>, Read<Collider>, Read<PlayerComp>)>::query())
        .build(move |commands, world, resource, queries| {
            
            for (entity, (pos, col, _)) in queries.0.iter_entities(&mut *world) {
                for (entity2, (pos2, col2, _)) in queries.1.iter_entities(&mut *world) {
                    let rot = euclid::Rotation2D::<f32, euclid::UnknownUnit, euclid::UnknownUnit>::new(pos.rotation);
                    let mut a1 = rot.transform_vector(euclid::Vector2D::<f32, euclid::UnknownUnit>::new(
                        col.offset_x, 
                        col.offset_y
                    ));
                    let mut a2 = rot.transform_vector(euclid::Vector2D::<f32, euclid::UnknownUnit>::new(
                        col.offset_x, 
                        col.offset_y
                    ));
                    //godot_print!("Angle: {} pre-rotated [{}, {}], post-rotated [{}. {}]", pos.rotation.get(), col.offset_x + (col.width / 2.0), col.offset_y + (col.height / 2.0), a2.x, a2.y);
                    a1.x += pos.x - (col.width / 2.0);
                    a1.y += pos.y - (col.height / 2.0);
                    a2.x += pos.x + (col.width / 2.0);
                    a2.y += pos.y + (col.height / 2.0);

                    let rot = euclid::Rotation2D::<f32, euclid::UnknownUnit, euclid::UnknownUnit>::new(pos2.rotation);
                    let mut b1 = rot.transform_vector(euclid::Vector2D::<f32, euclid::UnknownUnit>::new(
                        col2.offset_x,
                        col2.offset_y
                    ));
                    let mut b2 = rot.transform_vector(euclid::Vector2D::<f32, euclid::UnknownUnit>::new(
                        col2.offset_x,
                        col2.offset_y
                    ));
                    b1.x += pos2.x - (col2.width / 2.0);
                    b1.y += pos2.y - (col2.height / 2.0);
                    b2.x += pos2.x + (col2.width / 2.0);
                    b2.y += pos2.y + (col2.height / 2.0);

                    godot_print!("are overlapping?\n[{:?}, {:?}]\n[{:?}, {:?}]\n", a1, a2, b1, b2);
                    if (b1.x >= a1.x && b1.x <= a2.x) || (b2.x >= a1.x && b2.x <= a2.x) {
                        if (b1.y >= a1.y && b1.y <= a2.y) || (b2.y >= a1.y && b2.y <= a2.y) {
                            // We have a collision
                            godot_print!("Collision");
                        }
                    }
                }   
            }
        })
}