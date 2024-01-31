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
                color: s_rgbtolinear_rgb(Vec3f::new(179.,56.,56.))
            },
            Transform::from_position(Vec3f::new(-5., 10., -5.)),
        ));
        commands.spawn((
            Light {
                color: s_rgbtolinear_rgb(Vec3f::new(227.,181.,164.))
            },
            Transform::from_position(Vec3f::new(5., 10., -5.)),
        ));
        commands.spawn((
            Light {
                color: s_rgbtolinear_rgb(Vec3f::new(59.,195.,132.))
            },
            Transform::from_position(Vec3f::new(5., 10., 5.)),
        ));
        commands.spawn((
            Light {
                color: s_rgbtolinear_rgb(Vec3f::new(243.,152.,68.))
            },
            Transform::from_position(Vec3f::new(-5., 10., 5.)),
        ));
    }
}
