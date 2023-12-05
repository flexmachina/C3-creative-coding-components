use bevy_ecs::prelude::*;

// Defines the priority in which components are rendered (lowest first).
// Default is 0.
#[derive(Component)]
pub struct RenderOrder(pub i32);
