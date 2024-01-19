use anyhow::*;
use image::{GenericImageView, RgbaImage};
use wgpu::{AstcBlock, AstcChannel};

use crate::assets;
use crate::utils::wgpu_ext::{DeviceExt, TextureDataOrder};


pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn create_depth_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[Self::DEPTH_FORMAT],
        };
        let texture = device.create_texture(&desc);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }

    #[allow(dead_code)]
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
        is_normal_map: bool,
    ) -> Result<Self> {
        let img = image::load_from_memory(bytes)?;
        Self::from_image(device, queue, &img, Some(label), is_normal_map)
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
        is_normal_map: bool,
    ) -> Result<Self> {
        let dimensions = img.dimensions();
        let rgba = img.to_rgba8();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let format = if is_normal_map {
            wgpu::TextureFormat::Rgba8Unorm
        } else {
            wgpu::TextureFormat::Rgba8UnormSrgb
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }

    #[allow(dead_code)]
    pub async fn load_cubemap_from_ktx2(dir: &str, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let device_features = device.features();

        let format = if device_features.contains(wgpu::Features::TEXTURE_COMPRESSION_ASTC) {
            wgpu::TextureFormat::Astc {
                block: AstcBlock::B4x4,
                channel: AstcChannel::UnormSrgb,
            }
        } else if device_features.contains(wgpu::Features::TEXTURE_COMPRESSION_ETC2) {
            wgpu::TextureFormat::Etc2Rgb8A1UnormSrgb
        } else if device_features.contains(wgpu::Features::TEXTURE_COMPRESSION_BC) {
            wgpu::TextureFormat::Bc7RgbaUnormSrgb
        } else {
            wgpu::TextureFormat::Rgba8UnormSrgb
        };

        let filename_prefix = match format {
            wgpu::TextureFormat::Astc {
                block: AstcBlock::B4x4,
                channel: AstcChannel::UnormSrgb,
            } => "astc",
            wgpu::TextureFormat::Etc2Rgb8A1UnormSrgb => "etc2",
            wgpu::TextureFormat::Bc7RgbaUnormSrgb => "bc7",
            wgpu::TextureFormat::Rgba8UnormSrgb => "rgba8",
            _ => unreachable!(),
        };
        
        let filepath = format!("{dir}/{filename_prefix}.ktx2");
        //log::error!("Loading skybox: {filepath}");
        let bytes = assets::load_binary(filepath.as_str()).await.unwrap();

        let reader = ktx2::Reader::new(bytes).unwrap();
        let header = reader.header();

        let size = wgpu::Extent3d {
            width: header.pixel_width,
            height: header.pixel_height,
            depth_or_array_layers: 6,
        };

        let layer_size = wgpu::Extent3d {
            depth_or_array_layers: 1,
            ..size
        };
        let max_mips = layer_size.max_mips(wgpu::TextureDimension::D2);

        /*
        log::debug!(
            "Copying {:?} skybox images of size {}, {}, 6 with {} mips to gpu",
            format,
            header.pixel_width,
            header.pixel_height,
            max_mips,
        );
        */

        let mut image = Vec::with_capacity(reader.data().len());
        for level in reader.levels() {
            image.extend_from_slice(level);
        }

        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                size,
                mip_level_count: header.level_count,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: None,
                view_formats: &[],
            },
            // KTX2 stores mip levels in mip major order 
            // Specifying this order is only available in wgpu 0.18
            // but we're not ready to upgrade yet so a workaround is to backport
            // this functionality (see crate::wgpu_ext::device).
            TextureDataOrder::MipMajor,
            &image,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..wgpu::TextureViewDescriptor::default()
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }

    pub async fn load_cubemap_from_pngs(dir: &str, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        // Names of cube faces to load in order
        let faces = ["px", "nx", "py", "ny", "pz", "nz"]; 
        let images = {
            let mut imgs: Vec<RgbaImage> = Vec::new();
            for face in faces {
                let filepath = format!("{dir}/{face}.png");
                let bytes = assets::load_binary(filepath.as_str()).await.unwrap();
                let image = image::load_from_memory(&bytes).unwrap().to_rgba8();   
                imgs.push(image);         
            }
            imgs
        };
        
        let width = images[0].width();
        let height = images[0].height();

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 6,
        };

        let format = wgpu::TextureFormat::Rgba8UnormSrgb;

        /*
        log::debug!(
            "Copying {:?} skybox images of size {}, {}, 6 with no mips to gpu",
            format,
            width,
            height
        );
        */

        let texture = Self::create_texture_with_image_array(
            device,
            queue,
            &wgpu::TextureDescriptor {
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: None,
                view_formats: &[],
            },
            images
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..wgpu::TextureViewDescriptor::default()
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }

    fn create_texture_with_image_array(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        desc: &wgpu::TextureDescriptor<'_>,
        images: Vec<RgbaImage>,
    ) -> wgpu::Texture {
        // Implicitly add the COPY_DST usage
        let mut desc = desc.to_owned();
        desc.usage |= wgpu::TextureUsages::COPY_DST;
        let texture = device.create_texture(&desc);

        for layer in 0..desc.array_layer_count() {
            let image = &images[layer as usize];
            let width = image.width();
            let height = image.height();

            let size = wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            };
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: layer,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &image,
                wgpu::ImageDataLayout {
                    offset: 0,
                    // Assuming we have an RGBA image with 1 byte per channel
                    // TODO: Can we make this RGB to save some memory?
                    bytes_per_row: Some(image.width() * 4),
                    rows_per_image: Some(image.height()),
                },
                size,
            );
        }

        texture
    }


    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}
