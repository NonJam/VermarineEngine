mod systems;
mod components;

pub use crate::systems::*;
pub use crate::components::*;
pub use rpops_shared::prelude::*;

pub mod prelude {
    pub use crate::systems::*;
    pub use crate::components::*;
    pub use gdnative::*;
    pub use legion::prelude::*;
    pub use legion::prelude::World as LWorld;
    pub use rpops_shared::prelude::*;
}