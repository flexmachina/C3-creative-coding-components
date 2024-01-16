use std::collections::HashMap;

use crate::components::{Camera, Skybox, Transform, Player, ModelSpec, Light, FloorBox};
use crate::assets::{Assets,Renderers};
use crate::model::Model;
use crate::renderers::{SkyboxPass, PhongConfig, PhongPass};

use crate::device::Device;
use bevy_ecs::prelude::*;

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
    mut renderers: ResMut<Renderers>,
    mut commands: Commands,
) {
    let webxr = false;
    renderers.skybox_renderer = Some(SkyboxPass::new(
        &device,
        &assets,
        device.surface_texture_format(),
        webxr
    ));

    renderers.phong_renderer = Some(PhongPass::new(
        &PhongConfig { wireframe: false },
        &device,
        device.surface_texture_format(),
        webxr
    ));

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
    assets: Res<Assets>,
    mut renderers: ResMut<Renderers>,
    camera_qry: Query<(&Camera, &Transform), With<Player>>,
    meshes_qry: Query<(&ModelSpec, &Transform)>,
    light_qry: Query<(&Light, &Transform)>,
    skyboxes_qry: Query<&Skybox>,
) {
    let camera = camera_qry.single();
    let light = light_qry.single();

    let mut nodes: Vec<(&Model, Vec<&Transform>)> = vec![];
    for (modelspec, transform) in meshes_qry.iter() {
        let model: &Model =  assets.model_store.get(&modelspec.modelname).unwrap();
        nodes.push((model, vec![transform]));
    }

    /*
    //
    // Gather models to render
    //
    // Group by ModelSpec
    // TODO use ModelSpec as key?
    let mut instances: HashMap<&String, Vec<&Transform>> = HashMap::new();        
    for (model_spec, transform) in meshes_qry.iter() {
            instances.entry(&model_spec.modelname)
                .or_insert_with(Vec::new)
                .push(transform);                    
    }

    // Lookup Model from ModelSpec and flatten to vector
    for (modelname, transforms) in instances.into_iter() {
        let model =  assets.model_store.get(modelname).unwrap();
        nodes.push((model, transforms));
    }
    */

    ///////////////

    //let mut encoder = new_bundle_encoder(device.into_inner(), player_cam.0.target().as_ref());
    //new_bundle_encoder<'a>(device: &'a Device) -> wgpu::RenderBundleEncoder<'a>
    let surface = device.surface(); 
    let surface_texture = surface.get_current_texture().unwrap();
    let color_view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

    let skybox_renderer = renderers.skybox_renderer.as_mut().unwrap();
    let skybox_cmd_buffer = skybox_renderer.draw(
        &color_view,
        &device,
        camera,
        &None,
        true
    );
    ///////////////////////////////////////////////////////////
    let phong_renderer = renderers.phong_renderer.as_mut().unwrap();
    let phong_cmd_buffer = phong_renderer.draw(
        &color_view,
        &device.depth_tex().view,
        &device,
        &device.queue(),
        &nodes,
        camera,
        light,
        &None, 
        false, 
        true,
    );


    device.queue().submit([skybox_cmd_buffer, phong_cmd_buffer]);    
    surface_texture.present();

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



