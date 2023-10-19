use crate::device::Device;
use crate::mesh::MeshVertex;
use crate::shaders::utils::*;
use crate::texture::Texture;
use wgpu::{BindGroup, RenderPipeline};

pub struct PostProcessShader {
    pipeline: RenderPipeline,
    texture_bind_group: BindGroup,
}

pub struct PostProcessShaderParams<'a> {
    pub texture: &'a Texture,
}

impl PostProcessShader {
    pub async fn new(device: &Device, params: PostProcessShaderParams<'_>) -> Self {
        let (texture_bind_group_layout, texture_bind_group) =
            new_texture_bind_group(device, params.texture, wgpu::TextureViewDimension::D2);

        let shader_module = new_shader_module(device, "post-process.wgsl").await;

        let pipeline = new_render_pipeline(
            device,
            RenderPipelineParams {
                shader_module,
                depth_write: true,
                depth_enabled: true,
                bind_group_layouts: &[&texture_bind_group_layout],
                vertex_buffer_layouts: &[MeshVertex::buffer_layout()],
            },
        )
        .await;

        Self {
            pipeline,
            texture_bind_group,
        }
    }

    pub fn apply<'a>(&'a mut self, encoder: &mut wgpu::RenderBundleEncoder<'a>) {
        encoder.set_pipeline(&self.pipeline);
        encoder.set_bind_group(0, &self.texture_bind_group, &[]);
    }
}
