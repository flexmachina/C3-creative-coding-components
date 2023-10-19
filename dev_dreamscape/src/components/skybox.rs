use crate::assets::Assets;
use crate::components::transform::Transform;
use crate::components::{MeshRenderer, RenderOrder, ShaderVariant};
use crate::device::Device;
use crate::mesh::Mesh;
use crate::render_tags::RenderTags;
use crate::shaders::{SkyboxShader, SkyboxShaderParams};
use crate::texture::Texture;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Skybox;

impl Skybox {
    pub fn spawn(mut commands: Commands, device: Res<Device>, assets: Res<Assets>) {
        let (shader, mesh) = pollster::block_on(async {
            let shader = SkyboxShader::new(
                &device,
                SkyboxShaderParams {
                    texture: &assets.skybox_tex,
                },
            )
            .await;
            let mesh = Mesh::quad(&device);
            (shader, mesh)
        });

        let renderer = MeshRenderer::new(mesh, ShaderVariant::Skybox(shader), RenderTags::SCENE);

        let transform = Transform::default();

        commands.spawn((Skybox, RenderOrder(-100), renderer, transform));
    }
}
