use crate::prelude::*;

pub struct MainState {
    pub executor: Executor,
}

impl State for MainState {
    fn on_push(&mut self, data: &mut StateData, resources: &mut Resources) {
        use CreatureRenderables::*;
        let renderables = resources.get::<Models<Renderables>>().unwrap();
        let player = renderables.data_from_t(&Renderables::Creatures(Player)).unwrap();
        
        data.world.insert(
            (), 
            (0..1).map(|_| (
                Renderable::new(Position::default(), player.1, player.0),
                Position::new(240f32, 450f32), 
                TakesInput { speed: 400f32 / 60f32 }, 
                Velocity::default(),
                Collider { width: 25.0, height: 25.0, offset_x: 0.0, offset_y: -2.5},
                PlayerComp { },
            ))
        );
    }

    fn update(&mut self, data: &mut StateData, resources: &mut Resources) {
        self.executor.execute(&mut data.world, resources);
    }
}