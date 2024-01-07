use crate::assets::Assets;
use crate::components::mesh_renderer::ShaderVariant;
use crate::components::transform::Transform;
use crate::components::{MeshRenderer, PhysicsBody, PhysicsBodyParams};
use crate::components::{MeshSpec, ShaderStage};
use crate::device::Device;
use crate::math::Vec3;
use crate::mesh::Mesh;
use crate::physics_world::PhysicsWorld;
use crate::render_tags::RenderTags;
use crate::shaders::{DiffuseShader, DiffuseShaderParams};
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct FloorBox;

impl FloorBox {
    pub fn spawn(
        mut commands: Commands,
        device: Res<Device>,
        mut physics: ResMut<PhysicsWorld>,
        assets: Res<Assets>,
    ) {
        /*
        let (shader, mesh) = pollster::block_on(async {
            let shader = DiffuseShader::new(
                &device,
                DiffuseShaderParams {
                    texture: &assets.stone_tex,
                },
            )
            .await;
            let mesh = Mesh::from_file("cube.obj", &device).await;
            (shader, mesh)
        });
        */
        let shader = DiffuseShader::new(
            &device,
            DiffuseShaderParams {
                texture: &assets.stone_tex,
            },
        );
        let mesh = Mesh::from_string(assets.cube_mesh_string.clone(), &device);


        let renderer = MeshRenderer::new(mesh, ShaderVariant::Diffuse(shader), RenderTags::SCENE);


        let meshspec = MeshSpec::new(String::from("cube.obj"),
                                     vec![ShaderStage::Diffuse]
                       );


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



        commands.spawn((FloorBox, physics_body, renderer, transform, meshspec));
    }
}
