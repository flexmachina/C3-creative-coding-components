use crate::assets::Assets;
use crate::components::transform::Transform;
use crate::components::{PhysicsBody, PhysicsBodyParams};
use crate::components::{MeshSpec, ShaderStage};
use crate::math::Vec3;
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
        let meshspec = MeshSpec::new(String::from("cube.obj"),vec![ShaderStage::Diffuse]);

        let pos = Vec3::from_element(0.0);
        let scale = Vec3::new(100.0, 0.5, 100.0);
        let transform = Transform::new(pos, scale);

        let physics_body = PhysicsBody::new(
            PhysicsBodyParams {
                pos,
                scale,
                rotation_axis: Vec3::from_element(0.0),
                rotation_angle: 0.0,
                movable: false,
            },
            &mut physics,
        );

        commands.spawn((FloorBox, physics_body, transform, meshspec));
    }
}
