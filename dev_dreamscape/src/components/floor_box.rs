use crate::components::transform::Transform;
use crate::components::{PhysicsBody, PhysicsBodyParams};
use crate::components::ModelSpec;
use crate::math::{Vec3f,UnitQuatf};
use crate::physics_world::PhysicsWorld;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct FloorBox;

impl FloorBox {
    pub fn spawn(
        mut commands: Commands,
        mut physics: ResMut<PhysicsWorld>,
    ) {
        let modelspec = ModelSpec::new(String::from("cube.obj"));

        let pos = Vec3f::new(0.0, -1., 0.0);
        let rot = UnitQuatf::identity();
        let scale = Vec3f::new(100.0, 0.5, 100.0);
        let transform = Transform::new(pos, rot, scale);

        let physics_body = PhysicsBody::new(
            PhysicsBodyParams {
                pos,
                scale,
                rotation_axis: Vec3f::from_element(0.0),
                rotation_angle: 0.0,
                movable: false,
            },
            &mut physics,
        );

        commands.spawn((FloorBox, modelspec, physics_body, transform));
    }   
}
