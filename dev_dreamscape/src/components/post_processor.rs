use crate::components::{Camera, MeshRenderer, ShaderVariant, Player, RenderOrder, Transform};
use crate::device::Device;
use crate::mesh::Mesh;
use crate::render_tags::RenderTags;
use crate::shaders::{PostProcessShader, PostProcessShaderParams};
use bevy_ecs::prelude::*;
use crate::texture::TextureSize;

#[derive(Component)]
pub struct PostProcessor {
    size: TextureSize,
}

impl PostProcessor {
    pub fn spawn(
        mut commands: Commands,
        player: Query<&Camera, With<Player>>,
        device: Res<Device>,
    ) {
        // We know we need the player camera
        let source_camera_rt = player.single().target().as_ref().unwrap();

        let mesh = Mesh::quad(&device);

        let shader = pollster::block_on(async {
            PostProcessShader::new(
                &device,
                PostProcessShaderParams {
                    texture: source_camera_rt.color_tex(),
                },
            ).await
        });

        let renderer = MeshRenderer::new(
            mesh,
            ShaderVariant::PostProcess(shader),
            RenderTags::POST_PROCESS,
        );
        let transform = Transform::default();
        let pp = PostProcessor { size: source_camera_rt.color_tex().size() };

        commands.spawn((renderer, transform, pp));

        // Camera for rendering the quad (and debug UI for that matter)
        let camera = Camera::new(1.0, RenderTags::POST_PROCESS | RenderTags::DEBUG_UI, None);
        let transform = Transform::default();
        commands.spawn((RenderOrder(100), camera, transform));
    }

    pub fn update(
        device: Res<Device>,
        mut pp: Query<(&PostProcessor, &mut MeshRenderer)>,
        player_cam: Query<&Camera, With<Player>>
    ) {
        if let Some(pp) = pp.iter_mut().next().as_mut() {
            let source_camera_rt = player_cam.single().target().as_ref().unwrap();

            if source_camera_rt.color_tex().size() != pp.0.size {
                // TODO Better. We should NOT be re-creating the shader.
                let shader = pollster::block_on(async {
                    PostProcessShader::new(
                        &device,
                        PostProcessShaderParams {
                            texture: source_camera_rt.color_tex(),
                        },
                    ).await
                });

                pp.1.set_shader(ShaderVariant::PostProcess(shader));
            }
        }
    }
}
