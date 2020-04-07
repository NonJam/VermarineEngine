use crate::prelude::*;

pub fn CreateSystems() -> Vec<Box<dyn Schedulable>> {
    vec![
        CreateTestSystem(),
    ]
}

pub(crate) fn CreateTestSystem() -> Box<dyn Schedulable> {
    SystemBuilder::<()>::new("TestSystem")
        .with_query(<(Write<Position>)>::query())
        .build(move |commands, world, resource, queries| {
            for (entity, (mut pos)) in queries.iter_entities_mut(&mut *world) {
                pos.x+=1;
                if pos.x > 100 {
                    commands.delete(entity);
                }
            }
        })
}