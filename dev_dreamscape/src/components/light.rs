use bevy_ecs::prelude::*;
use crate::components::Transform;

use crate::assets::Assets;
use crate::math::Vec3f;


#[derive(Debug,Component)]
pub struct Light {
    pub color: Vec3f,
}

impl Light {
    pub fn spawn(mut commands: Commands, assets: Res<Assets>) {
        commands.spawn((
            Light {
                color: Vec3f::new(1., 1., 1.)
            },
            Transform::default()
        ));
    }
}
