use std::collections::HashMap;

use crate::math::Rect;
use crate::components::{Camera, Light, ModelSpec, Player, Skybox, Transform};
use crate::assets::Assets;
use crate::model::Model;
use crate::renderers::{HdrPipeline, SkyboxPass, PhongConfig, PhongPass};

use crate::device::Device;
use bevy_ecs::prelude::*;


// TODO Load also shaders, meshes, etc.
#[derive(Resource)]
pub struct Renderers {
    pub skybox_renderer: SkyboxPass,
    pub phong_renderer: PhongPass,
    pub hdr_pipeline: HdrPipeline,
}

impl Renderers {
    pub fn new(device: &Device, webxr: bool) -> Self {
        
        let hdr_pipeline = HdrPipeline::new(
            &device,
            device.surface_size().width,
            device.surface_size().height,
            device.surface_texture_format(),
            webxr
        );
    
        let color_format = hdr_pipeline.format();
    
        let skybox_renderer = SkyboxPass::new(
            &device,
            color_format,
        );
    
        let phong_renderer = PhongPass::new(
            &PhongConfig { wireframe: false },
            &device,
            color_format,
        );

        Self {
            skybox_renderer, 
            phong_renderer,
            hdr_pipeline,
        }
    }
}

pub fn render_to_texture(
    device: &Device,
    assets: Res<Assets>,
    mut renderers: ResMut<Renderers>,
    camera_qry: Query<(&Camera, &Transform), With<Player>>,
    skybox_qry: Query<&Skybox>,
    meshes_qry: Query<(&ModelSpec, &Transform)>,
    lights_qry: Query<(&Light, &Transform)>,
    color_texture: &wgpu::Texture,
    viewport: Option<Rect>,
    clear: bool) {

    let camera = camera_qry.single();
    let skybox = skybox_qry.single();
    
    // Get skybox texture
    let skybox_texture =  assets.texture_store.get(&skybox.texture_name).unwrap();

    //
    // Gather models to render
    //

    // Group by ModelSpec
    // TODO: use ModelSpec as key?
    let mut instances: HashMap<&String, Vec<&Transform>> = HashMap::new();        
    for (model_spec, transform) in meshes_qry.iter() {
        instances.entry(&model_spec.modelname)
            .or_insert_with(Vec::new)
            .push(transform);
    }

    // Lookup Model from ModelSpec and flatten to vector
    let mut nodes: Vec<(&Model, &String, Vec<&Transform>)> = vec![];
    for (modelname, transforms) in instances.into_iter() {
        //println!("nodes modelname: {:?}",modelname);
        let model =  assets.model_store.get(modelname).unwrap();
        nodes.push((model, modelname, transforms));
    }

    // Gather light models
    let mut lights: Vec<(&Light, &Transform)> = vec![];
    for (light, transform) in lights_qry.iter() {
        lights.push((light, transform));
    }
    // TODO: don't hardcode. We rely on the same mode for all lights for instancing atm.
    let light_model = assets.model_store.get("sphere.obj").unwrap();

    //
    // Render passes
    //

    // Resize hdr_pipeline texture if needed to match viewport (if present), or
    // else the entire colour buffer.
    {
        let hdr_pipeline = &mut renderers.hdr_pipeline;
        let (target_width, target_height) = match &viewport {
            Some(vp) => (vp.w as u32, vp.h as u32),
            None => (color_texture.width(), color_texture.height())
        };
        if target_width == 0 || target_height == 0 {
            return;
        }
        if (hdr_pipeline.texture().width(), hdr_pipeline.texture().height()) != (target_width, target_height) {
            hdr_pipeline.resize(device, target_width, target_height);
        } 
    }

    // Need to create new views as the borrow checker complains about about multiple refs.
    // TODO: find a better solution
    let hdr_view = renderers.hdr_pipeline.texture().create_view(&wgpu::TextureViewDescriptor::default());
    let color_view = color_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let depth_view = renderers.hdr_pipeline.depth_texture().create_view(&wgpu::TextureViewDescriptor::default());

    // Skypass pass
    // TODO: Use Skybox Query to make skybox config dynamic
    let skybox_cmd_buffer = renderers.skybox_renderer.draw(
        &hdr_view,
        &device,
        camera,
        (&skybox.texture_name, &skybox_texture),
        true,
    );

    // Phong pass
    let phong_cmd_buffer = renderers.phong_renderer.draw(
        &hdr_view,
        &depth_view,
        &device,
        &device.queue(),
        &nodes,
        camera,
        &lights,
        light_model,
        false,
        true,
    );

    let hdr_cmd_buffer = renderers.hdr_pipeline.process(&device, &color_view, viewport);

    device.queue().submit([
        skybox_cmd_buffer,
        phong_cmd_buffer,
        hdr_cmd_buffer
    ]);
}



pub fn render(
    device: ResMut<Device>,
    assets: Res<Assets>,
    renderers: ResMut<Renderers>,
    camera_qry: Query<(&Camera, &Transform), With<Player>>,
    skybox_qry: Query<&Skybox>,
    meshes_qry: Query<(&ModelSpec, &Transform)>,
    lights_qry: Query<(&Light, &Transform)>,
) {
    let surface = device.surface(); 
    let surface_texture = surface.get_current_texture().unwrap();
    
    render_to_texture(
                &device,
                assets,
                renderers,
                camera_qry,
                skybox_qry,
                meshes_qry,
                lights_qry,
                &surface_texture.texture,
                None,
                true);

    surface_texture.present();
}
