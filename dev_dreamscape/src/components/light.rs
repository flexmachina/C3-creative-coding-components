use bevy_ecs::prelude::*;

use crate::components::Transform;
use crate::math::Vec3f;


#[derive(Debug,Component)]
pub struct Light {
    pub color: Vec3f,
}

impl Light {
    pub fn spawn(mut commands: Commands) {
        commands.spawn((
            Light {
                color: Vec3f::new(1., 1., 1.)
            },
            Transform::from_position(Vec3f::new(-5., 10., -5.)),
        ));
        commands.spawn((
            Light {
                color: Vec3f::new(1., 0., 0.)
            },
            Transform::from_position(Vec3f::new(5., 10., -5.)),
        ));
        commands.spawn((
            Light {
                color: Vec3f::new(0., 1., 0.)
            },
            Transform::from_position(Vec3f::new(5., 10., 5.)),
        ));
        commands.spawn((
            Light {
                color: Vec3f::new(0., 0., 1.)
            },
            Transform::from_position(Vec3f::new(-5., 10., 5.)),
        ));
    }
}
