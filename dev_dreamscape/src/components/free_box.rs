use crate::assets::Assets;
use crate::components::transform::Transform;
use crate::components::{MeshRenderer, PhysicsBody, PhysicsBodyParams, Player, ShaderVariant};
use crate::device::Device;
use crate::input::Input;
use crate::math::Vec3;
use crate::mesh::Mesh;
use crate::physics_world::PhysicsWorld;
use crate::render_tags::RenderTags;
use crate::shaders::{DiffuseShader, DiffuseShaderParams};
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct FreeBox;

impl FreeBox {
    pub fn spawn(
        mut commands: Commands,
        device: Res<Device>,
        mut physics: ResMut<PhysicsWorld>,
        assets: Res<Assets>,
    ) {
        let pos = Vec3::y_axis().xyz() * 10.0;
        commands.spawn(Self::new_components(pos, &device, &mut physics, &assets));
    }

    pub fn spawn_by_player(
        player: Query<&Transform, With<Player>>,
        mut commands: Commands,
        device: Res<Device>,
        mut physics: ResMut<PhysicsWorld>,
        input: Res<Input>,
        assets: Res<Assets>,
    ) {
        if input.space_just_pressed {
            let player_transform = player.single();
            let pos = player_transform.position() + player_transform.forward().xyz() * 5.0;
            commands.spawn(Self::new_components(pos, &device, &mut physics, &assets));
        }
    }

    fn new_components(
        pos: Vec3,
        device: &Device,
        physics: &mut PhysicsWorld,
        assets: &Assets,
    ) -> (FreeBox, PhysicsBody, MeshRenderer, Transform) {
        let (shader, mesh) = pollster::block_on(async {
            let shader = DiffuseShader::new(
                device,
                DiffuseShaderParams {
                    texture: &assets.stone_tex,
                },
            )
            .await;
            let mesh = Mesh::from_file("cube.obj", device).await;
            (shader, mesh)
        });

        let scale = Vec3::from_element(1.0);

        let physics_body = PhysicsBody::new(
            PhysicsBodyParams {
                pos,
                scale,
                rotation_axis: Vec3::identity(),
                rotation_angle: 0.0,
                movable: true,
            },
            physics,
        );

        let renderer = MeshRenderer::new(mesh, ShaderVariant::Diffuse(shader), RenderTags::SCENE);

        let transform = Transform::new(pos, scale);

        (FreeBox, physics_body, renderer, transform)
    }
}
