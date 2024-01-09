use crate::components::{Camera, MeshRenderer, RenderOrder, Transform};
use crate::device::Device;
use crate::render_target::RenderTarget;
use bevy_ecs::prelude::*;
use wgpu::RenderBundle;
use crate::render_tags::RenderTags;

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