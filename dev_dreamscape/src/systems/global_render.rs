use crate::components::{Camera, MeshRenderer, RenderOrder, Transform, Player, MeshSpec, SkyboxSpec};
use crate::mesh::Mesh;
use crate::assets::Assets;
use crate::components::{ShaderVariant};

use crate::device::Device;
use crate::render_target::RenderTarget;
use bevy_ecs::prelude::*;
use wgpu::RenderBundle;
use crate::render_tags::RenderTags;

use crate::shaders::{SkyboxShader, SkyboxShaderParams};
use crate::shaders::{PostProcessShader, PostProcessShaderParams};
use crate::shaders::{DiffuseShader, DiffuseShaderParams};


fn render_pass(
    device: &Device,
    bundles: &[RenderBundle],
    target: Option<&RenderTarget>,
) {
    let surface_tex = target.is_none().then(|| {
        device
            .surface()
            .get_current_texture()
            .expect("Missing surface texture")
    });
    let surface_tex_view = surface_tex
        .as_ref()
        .map(|t| t.texture.create_view(&wgpu::TextureViewDescriptor::default()));

    let color_tex_view = target
        .map(|t| t.color_tex().view())
        .or(surface_tex_view.as_ref())
        .unwrap();
    let color_attachment = Some(wgpu::RenderPassColorAttachment {
        view: color_tex_view,
        resolve_target: None,
        ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::RED),
            store: true,
        },
    });

    let depth_tex_view = target
        .map(|t| t.depth_tex().view())
        .unwrap_or(device.depth_tex().view());
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

fn new_bundle_encoder<'a>(device: &'a Device, target: Option<&RenderTarget>) -> wgpu::RenderBundleEncoder<'a> {
    let color_format = target
        .map_or(device.surface_texture_format(), |t| t.color_tex().format());
    let depth_format = target
        .map_or(device.depth_texture_format(), |t| t.depth_tex().format());

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

fn build_render_bundles<'a>(
    renderers: &mut [(&'a mut MeshRenderer, &'a Transform)],
    camera: (&Camera, &Transform),
    device: &Device,
) -> Vec<RenderBundle> {
    // Couldn't make it work with a single bundler encoder due to lifetimes
    renderers
        .iter_mut()
        .filter(|(r, _)| camera.0.should_render(r.tags()))
        .map(|(ref mut r, tr)| {
            let mut encoder = new_bundle_encoder(device, camera.0.target().as_ref());
            // TODO Create render bundle inside the function?
            r.render(device, camera, tr, &mut encoder);
            encoder.finish(&wgpu::RenderBundleDescriptor { label: None })
        })
        .collect::<Vec<_>>()
}

pub fn render(
    cameras: Query<(&Camera, &Transform, Option<&RenderOrder>)>,
    mut renderers: Query<(&mut MeshRenderer, &Transform, Option<&RenderOrder>)>,
    device: Res<Device>,
) {
    let mut cameras = cameras.into_iter().collect::<Vec<_>>();
    cameras.sort_by_key(|(.., order)| order.map_or(0, |o| o.0));
    let cameras = cameras.iter().map(|(c, t, ..)| (*c, *t));


    let mut renderers = renderers
        .iter_mut()
        .map(|(r, t, o)| (r.into_inner(), t, o))
        .collect::<Vec<_>>();
    renderers.sort_by_key(|(.., order)| order.map_or(0, |o| o.0));
    let mut renderers = renderers
        .into_iter()
        .map(|(r, t, ..)| (r, t))
        .collect::<Vec<_>>();

    println!("---");
    for camera in cameras {
        println!("Have a camera!");
        let bundles = build_render_bundles(&mut renderers, camera, &device);
        render_pass(
            &device,
            &bundles,
            camera.0.target().as_ref()
        );
    }
}






pub fn global_prepare_render_pipelines(
    device: Res<Device>,
    assets: Res<Assets>,
    mut commands: Commands,
    player_cam_qry: Query<(&Camera, &Transform), With<Player>>,
) {

    //Leave as single pipeline to render skybox
    let skybox_shader = SkyboxShader::new(&device,SkyboxShaderParams {texture: &assets.skybox_tex,});
    let skybox_mesh = Mesh::quad(&device);
    let skybox_renderer = MeshRenderer::new(skybox_mesh, ShaderVariant::Skybox(skybox_shader), RenderTags::SCENE);
    let skybox_transform = Transform::default();


    //Make a pipeline for rendering all other mesh objects?
    let diffuse_shader = DiffuseShader::new(&device,DiffuseShaderParams {texture: &assets.stone_tex,});
    let diffuse_mesh = Mesh::from_string(assets.cube_mesh_string.clone(), &device);
    let diffuse_renderer = MeshRenderer::new(diffuse_mesh, ShaderVariant::Diffuse(diffuse_shader), RenderTags::SCENE);

    /*
    //leave as is for Post Processing, can get rid of post process shader camera?
    let source_camera_rt = player_cam_qry.single().target().as_ref().unwrap();

    let mesh = Mesh::quad(&device);
    let shader = PostProcessShader::new(&device,PostProcessShaderParams {
                    texture: source_camera_rt.color_tex()
                 });
    let renderer = MeshRenderer::new(
        mesh,
        ShaderVariant::PostProcess(shader),
        RenderTags::POST_PROCESS,
    );
    let transform = Transform::default();
    let pp = PostProcessor { size: source_camera_rt.color_tex().size() };
    commands.spawn((renderer, transform, pp));
    // Camera for rendering the quad
    let camera = Camera::new(1.0, RenderTags::POST_PROCESS, None);
    let transform = Transform::default();
    commands.spawn((RenderOrder(100), camera, transform));
    */
}




pub fn global_render(
    device: Res<Device>,
    player_cam_qry: Query<(&Camera, &Transform), With<Player>>,
    mut meshes_qry: Query<(&MeshSpec, &Transform, Option<&RenderOrder>)>,
    mut skyboxes_qry: Query<(&SkyboxSpec, &Transform, Option<&RenderOrder>)>,
) {

    let player_cam = player_cam_qry.single();
    let mut encoder = new_bundle_encoder(device.into_inner(), player_cam.0.target().as_ref());

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



