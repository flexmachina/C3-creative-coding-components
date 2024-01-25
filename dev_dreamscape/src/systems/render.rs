use std::collections::HashMap;

use crate::math::Rect;
use crate::components::{Camera, Transform, Player, ModelSpec, Light};
use crate::assets::{Assets,Renderers};
use crate::app::AppState;
use crate::model::Model;
use crate::renderers::{SkyboxPass, PhongConfig, PhongPass};

use crate::device::Device;
use bevy_ecs::prelude::*;


pub fn prepare_render_pipelines(
    device: Res<Device>,
    assets: Res<Assets>,
    appstate: Res<AppState>,
    mut renderers: ResMut<Renderers>,
) {
    let webxr = appstate.webxr;
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
}


pub fn render_to_texture(
                device: &Device,
                assets: Res<Assets>,
                mut renderers: ResMut<Renderers>,
                camera_qry: Query<(&Camera, &Transform), With<Player>>,
                meshes_qry: Query<(&ModelSpec, &Transform)>,
                lights_qry: Query<(&Light, &Transform)>,
                //                                         
                color_texture: &wgpu::Texture,
                depth_texture: Option<&wgpu::Texture>,
                viewport: Option<Rect>,
                clear: bool) {

    let camera = camera_qry.single();
  
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


    let color_view = color_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let depth_view = match depth_texture {
        Some(d) => d.create_view(&wgpu::TextureViewDescriptor::default()),
        _ => device.depth_tex().texture.create_view(&wgpu::TextureViewDescriptor::default()) 
            //Error because this returned a reference
            //device.depth_tex().view
    };


    // Skypass pass
    // TODO: Use Skybox Query to make skybox config dynamic
    let skybox_renderer = renderers.skybox_renderer.as_mut().unwrap();
    let skybox_cmd_buffer = skybox_renderer.draw(
        &color_view,
        &device,
        camera,
        &viewport,
        clear
    );

    // Phong pass
    let phong_renderer = renderers.phong_renderer.as_mut().unwrap();
    let phong_cmd_buffer = phong_renderer.draw(
        &color_view,
        &depth_view,
        &device,
        &device.queue(),
        &nodes,
        camera,
        &lights,
        light_model,
        &viewport, 
        false,
        clear,
    );

    device.queue().submit([skybox_cmd_buffer, phong_cmd_buffer]);    
}



pub fn render(
    device: Res<Device>,
    assets: Res<Assets>,
    renderers: ResMut<Renderers>,
    camera_qry: Query<(&Camera, &Transform), With<Player>>,
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
                meshes_qry,
                lights_qry,
                &surface_texture.texture,
                None,
                None,
                true);

    surface_texture.present();
}
