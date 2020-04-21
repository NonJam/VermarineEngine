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
        .write_component::<Renderable>()
        .write_resource::<Wrapper<HashMap<Entity, Node>>>()
        .with_query(<(Read<TakesInput>, Write<Velocity>)>::query())
        .build(move |_commands, world, _resource, queries| {
            for (entity, (takes_input, mut vel)) in queries.iter_entities_mut(&mut *world) {
                vel.x = 0f32;
                vel.y = 0f32;

                let mut i_x = 0i32;
                let mut i_y = 0i32;
                for (key, x, y) in [
                    ("move_left", -1, 0),
                    ("move_right", 1, 0),
                    ("move_up", 0, -1),
                    ("move_down", 0, 1),
                ].iter() {
                    if Input::godot_singleton().is_action_pressed(GodotString::from(key)) {
                        i_x += x;
                        i_y += y;
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

                if let Some(mut renderable) = world.get_component_mut::<Renderable>(entity) {
                    if let Template::ASprite(mut a_sprite) = renderable.template {
                        a_sprite.playing = true;
                        if i_x != 0i32 {
                            a_sprite.animation = "right";
                            a_sprite.flip_v = false;
                            a_sprite.flip_h = i_x < 0i32;
                        } else if i_y != 0i32 {
                            a_sprite.animation = "up";
                            a_sprite.flip_v = i_y > 0i32;
                        } else {
                            a_sprite.playing = false;
                        }
                        renderable.template = Template::ASprite(a_sprite)
                    }
                }
            }
        })
}

pub(crate) fn move_system() -> Box<dyn Schedulable> {
    SystemBuilder::<()>::new("MoveSystem")
        .with_query(<(Write<Position>, Read<Velocity>)>::query())
        .build(move |_commands, world, _resource, queries| {
            for (_entity, (mut pos, vel)) in queries.iter_entities_mut(&mut *world) {
                let rot = euclid::Rotation2D::<f32, euclid::UnknownUnit, euclid::UnknownUnit>::new(pos.rotation);
                let vel = rot.transform_vector(euclid::Vector2D::<f32, euclid::UnknownUnit>::new(vel.x, vel.y));
                pos.x += vel.x;
                pos.y += vel.y;
            } 
        })
}

pub(crate) fn spawn_enemy_system() -> Box<dyn Schedulable> {
    let mut counter: i32 = 1;
    SystemBuilder::<()>::new("SpawnEnemySystem")
        .write_resource::<Models<Renderables>>()
        .build(move |commands, _world, resource, _queries| {
            if counter % 30 == 0 {
                counter = 0;

                let mut enemy = resource.data_from_t(&Renderables::Creatures(CreatureRenderables::Enemy)).unwrap();
                let mut rand = rand::thread_rng();

                let anim = rand.gen_range(0, 3);
                if let Template::ASprite(mut a_sprite) = enemy.0 {
                    a_sprite.animation = match anim {
                        0 => {"swim"},
                        1 => {"fly"},
                        2 => {"walk"}
                        // This wont happen
                        _ => {""}
                    };
                    enemy.0 = Template::ASprite(a_sprite);
                }

                commands.insert(
                    (),
                    (0..1).map(move |_| {
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
                        Renderable { index: enemy.1, template: enemy.0 }, 
                        GDSpatial, 
                        position, 
                        Velocity { x: (rand.gen::<f32>() * 1.7f32) + 2.5f32, y: 0f32 },
                        Collider { width: 12.0, height: 12.0, offset_x: 14.0, offset_y: 0.0 },
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
        .build(move |_commands, world, _resource, queries| {
            
            for (_entity, (pos, col, _)) in queries.0.iter_entities(&mut *world) {
                for (_entity2, (pos2, col2, _)) in queries.1.iter_entities(&mut *world) {
                    let rot = euclid::Rotation2D::<f32, euclid::UnknownUnit, euclid::UnknownUnit>::new(pos.rotation);
                    let mut a1 = rot.transform_vector(euclid::Vector2D::<f32, euclid::UnknownUnit>::new(
                        col.offset_x, 
                        col.offset_y
                    ));
                    let mut a2 = rot.transform_vector(euclid::Vector2D::<f32, euclid::UnknownUnit>::new(
                        col.offset_x, 
                        col.offset_y
                    ));
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