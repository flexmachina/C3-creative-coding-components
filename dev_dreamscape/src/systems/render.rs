use std::collections::HashMap;

use crate::components::{Camera, Skybox, Transform, Player, ModelSpec, Light};
use crate::assets::{Assets,Renderers};
use crate::app::{AppState};
use crate::model::Model;
use crate::renderers::{SkyboxPass, PhongConfig, PhongPass};

use crate::device::Device;
use bevy_ecs::prelude::*;


pub fn prepare_render_pipelines(
    device: Res<Device>,
    assets: Res<Assets>,
    appstate: Res<AppState>,
    mut renderers: ResMut<Renderers>,
    mut commands: Commands,
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
    let mut nodes: Vec<(&Model, Vec<&Transform>)> = vec![];
    for (modelname, transforms) in instances.into_iter() {
        let model =  assets.model_store.get(modelname).unwrap();
        nodes.push((model, transforms));
    }

    //
    // Render passes
    //

    let surface = device.surface(); 
    let surface_texture = surface.get_current_texture().unwrap();
    let color_view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

    // Skypass pass
    let skybox_renderer = renderers.skybox_renderer.as_mut().unwrap();
    let skybox_cmd_buffer = skybox_renderer.draw(
        &color_view,
        &device,
        camera,
        &None,
        true
    );

    // Phong pass
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
}
