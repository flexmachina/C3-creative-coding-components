use crate::assets::Assets;
use crate::components::transform::Transform;
use crate::components::{PhysicsBody, PhysicsBodyParams};
use crate::components::{ModelSpec, ShaderStage};
use crate::math::Vec3f;
use crate::mesh::Mesh;
use crate::physics_world::PhysicsWorld;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct FloorBox;

impl FloorBox {
    pub fn spawn(
        mut commands: Commands,
        mut physics: ResMut<PhysicsWorld>,
        assets: Res<Assets>,
    ) {
        let modelspec = ModelSpec::new(String::from("cube.obj"),vec![ShaderStage::Diffuse]);

        let pos = Vec3f::from_element(0.0);
        let scale = Vec3f::new(100.0, 0.5, 100.0);
        let transform = Transform::new(pos, scale);

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

        commands.spawn((FloorBox, physics_body, transform, modelspec));
    }
}
