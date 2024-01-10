use crate::components::{Camera, RenderOrder, Transform, Player, MeshSpec, SkyboxSpec};
use crate::mesh::Mesh;
use crate::assets::Assets;

use crate::device::Device;
use bevy_ecs::prelude::*;
use wgpu::RenderBundle;

/*
fn render_pass(
    device: &Device,
    bundles: &[RenderBundle],
) {
    let surface_tex = device
            .surface()
            .get_current_texture()
            .expect("Missing surface texture");
    let surface_tex_view = surface_tex
        .as_ref()
        .map(|t| t.texture.create_view(&wgpu::TextureViewDescriptor::default()));

    let color_tex_view = surface_tex_view.as_ref().unwrap();
    let color_attachment = Some(wgpu::RenderPassColorAttachment {
        view: color_tex_view,
        resolve_target: None,
        ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::RED),
            store: true,
        },
    });

    let depth_tex_view = device.depth_tex().view();
    let depth_attachment = Some(wgpu::RenderPassDepthStencilAttachment {
        view: depth_tex_view,
        depth_ops: Some(wgpu::Operations {
            load: wgpu::LoadOp::Clear(1.0),
            store: true,
        }),
        stencil_ops: None,
    });

    let cmd_buffer = {
        let mut encoder = device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[color_attachment],
                depth_stencil_attachment: depth_attachment,
            });

            pass.execute_bundles(bundles.iter());
        }

        encoder.finish()
    };

    device.queue().submit(Some(cmd_buffer));
    if let Some(t) = surface_tex { t.present() }
}

fn new_bundle_encoder<'a>(device: &'a Device) -> wgpu::RenderBundleEncoder<'a> {
    let color_format = device.surface_texture_format();
    let depth_format = device.depth_texture_format();

    device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
        label: None,
        multiview: None,
        sample_count: 1,
        color_formats: &[Some(color_format)],
        depth_stencil: Some(wgpu::RenderBundleDepthStencil {
            format: depth_format,
            depth_read_only: false,
            stencil_read_only: false,
        }),
    })
}
*/


pub fn prepare_render_pipelines(
    device: Res<Device>,
    assets: Res<Assets>,
    mut commands: Commands,
    player_cam_qry: Query<(&Camera, &Transform), With<Player>>,
) {

    /*    
    //Leave as single pipeline to render skybox
    let skybox_shader = SkyboxShader::new(&device,SkyboxShaderParams {texture: &assets.skybox_tex,});
    let skybox_mesh = Mesh::quad(&device);
    let skybox_renderer = MeshRenderer::new(skybox_mesh, ShaderVariant::Skybox(skybox_shader), RenderTags::SCENE);
    let skybox_transform = Transform::default();


    //Make a pipeline for rendering all other mesh objects?
    let diffuse_shader = DiffuseShader::new(&device,DiffuseShaderParams {texture: &assets.stone_tex,});
    let diffuse_mesh = Mesh::from_string(assets.cube_mesh_string.clone(), &device);
    let diffuse_renderer = MeshRenderer::new(diffuse_mesh, ShaderVariant::Diffuse(diffuse_shader), RenderTags::SCENE);
    */
}




pub fn render(
    device: Res<Device>,
    player_cam_qry: Query<(&Camera, &Transform), With<Player>>,
    mut meshes_qry: Query<(&MeshSpec, &Transform, Option<&RenderOrder>)>,
    mut skyboxes_qry: Query<(&SkyboxSpec, &Transform, Option<&RenderOrder>)>,
) {

    let player_cam = player_cam_qry.single();
    //let mut encoder = new_bundle_encoder(device.into_inner(), player_cam.0.target().as_ref());

    let (skybox_spec, skybox_transform, skybox_renderorder) = skyboxes_qry.single();
   

    /*
    let mut encoder = new_bundle_encoder(device, camera.0.target().as_ref());    
    
    let player_cam = player_cam_qry.single_mut();

    let mut meshes_to_render = renderers
        .iter_mut()
        .map(|(r, t, o)| (r.into_inner(), t, o))
        .collect::<Vec<_>>();
    */



    /*
     * Update the skybox based on 
     *
     */

    /*
     * For each mesh key in assets, make a pipeline to render all instances of the mesh
     *
     */


    /*




    match self.shader {
        ShaderVariant::Diffuse(ref mut diffuse) => {
            diffuse.update_uniforms(device, camera, transform);
            diffuse.apply(encoder);
        }
        ShaderVariant::Skybox(ref mut skybox) => {
            skybox.update_uniforms(device, camera);
            skybox.apply(encoder);
        }
        ShaderVariant::PostProcess(ref mut pp) => {
            pp.apply(encoder);
        }
    }
    encoder.draw_mesh(&self.mesh);

    */

}



