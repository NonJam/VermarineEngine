use crate::prelude::*;

pub fn create_systems() -> Vec<Box<dyn Schedulable>> {
    vec![
        create_test_system(),
    ]
}

pub(crate) fn create_test_system() -> Box<dyn Schedulable> {
    SystemBuilder::<()>::new("TestSystem")
        .with_query(<Write<Position>>::query())
        .build(move |commands, world, _resource, queries| {
            for (entity, mut pos) in queries.iter_entities_mut(&mut *world) {
                pos.x+=1f32;
                if pos.x > 100f32 {
                    commands.delete(entity);
                }
            }
        })
}