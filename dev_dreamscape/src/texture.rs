use crate::assets::load_binary;
use crate::device::Device;
use anyhow::*;
use bevy_ecs::prelude::Resource;
use image::GenericImageView;

pub type TextureSize = (u32, u32);

#[derive(Resource)]
pub struct Texture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    format: wgpu::TextureFormat,
}

fn new_sampler(
    device: &wgpu::Device,
    filter: wgpu::FilterMode,
    mipmap_filter: wgpu::FilterMode,
    compare: Option<wgpu::CompareFunction>,
) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: None,
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: filter,
        min_filter: filter,
        mipmap_filter,
        lod_min_clamp: 0.0,
        lod_max_clamp: 100.0,
        compare,
        ..Default::default()
    })
}

// TODO Reduce copypasta
impl Texture {
    pub fn new_depth(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        size: TextureSize,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[format],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = new_sampler(
            device,
            wgpu::FilterMode::Nearest,
            wgpu::FilterMode::Nearest,
            Some(wgpu::CompareFunction::LessEqual),
        );

        Self {
            texture,
            view,
            sampler,
            format,
        }
    }

    pub fn new_render_attachment(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        size: TextureSize,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = new_sampler(
            device,
            wgpu::FilterMode::Nearest,
            wgpu::FilterMode::Nearest,
            None,
        );

        Self {
            texture,
            view,
            sampler,
            format,
        }
    }

    pub async fn new_2d_from_file(file_name: &str, device: &Device) -> Result<Self> {
        let data = load_binary(file_name).await?;
        Self::new_2d_from_mem(device, device.queue(), &data)
    }

    pub async fn new_cube_from_file(file_name: &str, device: &Device) -> Result<Self> {
        let data = load_binary(file_name).await?;
        Self::new_cube_from_mem(device, device.queue(), &data)
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    pub fn size(&self) -> TextureSize {
        (self.texture.size().width, self.texture.size().height)
    }

    fn new_2d_from_mem(device: &wgpu::Device, queue: &wgpu::Queue, bytes: &[u8]) -> Result<Self> {
        let img = image::load_from_memory(bytes)?;
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();
        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;

        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: None,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            &rgba,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = new_sampler(
            device,
            wgpu::FilterMode::Nearest,
            wgpu::FilterMode::Nearest,
            None,
        );

        Ok(Self {
            texture: texture,
            view,
            sampler,
            format,
        })
    }

    fn new_cube_from_mem(device: &wgpu::Device, queue: &wgpu::Queue, bytes: &[u8]) -> Result<Self> {
        let image = ddsfile::Dds::read(&mut std::io::Cursor::new(&bytes)).unwrap();

        let format = wgpu::TextureFormat::Rgba8UnormSrgb; // TODO Configurable

        let size = wgpu::Extent3d {
            width: 128,
            height: 128,
            depth_or_array_layers: 6,
        };

        let layer_size = wgpu::Extent3d {
            depth_or_array_layers: 1,
            ..size
        };
        let max_mips = layer_size.max_mips(wgpu::TextureDimension::D2);

        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                size,
                mip_level_count: max_mips,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: None,
                view_formats: &[],
            },
            &image.data,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..wgpu::TextureViewDescriptor::default()
        });

        let sampler = new_sampler(
            device,
            wgpu::FilterMode::Linear,
            wgpu::FilterMode::Linear,
            None,
        );

        Ok(Self {
            texture: texture,
            view,
            sampler,
            format,
        })
    }
}
