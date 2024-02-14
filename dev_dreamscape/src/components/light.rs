use bevy_ecs::prelude::*;

use crate::components::Transform;
use crate::math::{Vec3f, s_rgbtolinear_rgb};


#[derive(Debug,Component)]
pub struct Light {
    pub color: Vec3f,
}

impl Light {
    pub fn spawn(mut commands: Commands) {
        commands.spawn((
            Light {
                color: Vec3f::new(10.,0.,0.)
            },
            Transform::from_position(Vec3f::new(-5., 2., -5.)),
        ));
        commands.spawn((
            Light {
                color: Vec3f::new(0.0,10.,0.)
            },
            Transform::from_position(Vec3f::new(5., 2., -5.)),
        ));
        commands.spawn((
            Light {
                color: Vec3f::new(5.,0.,5.)
            },
            Transform::from_position(Vec3f::new(5., 2., 5.)),
        ));
        commands.spawn((
            Light {
                color: Vec3f::new(5.,5.,5.)
            },
            Transform::from_position(Vec3f::new(-5., 2., 5.)),
        ));
    }
}
