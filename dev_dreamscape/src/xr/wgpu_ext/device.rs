// Backported from wgpu 0.18.0 which adds the parameter `order: TextureDataOrder` to 
// create_texture_with_data. This is needed to support KT2X which uses TextureDataOrder::MipMajor
// TODO: remove this code after upgrading

/// Describes a [Buffer](wgpu::Buffer) when allocating.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BufferInitDescriptor<'a> {
    /// Debug label of a buffer. This will show up in graphics debuggers for easy identification.
    pub label: wgpu::Label<'a>,
    /// Contents of a buffer on creation.
    pub contents: &'a [u8],
    /// Usages of a buffer. If the buffer is used in any way that isn't specified here, the operation
    /// will panic.
    pub usage: wgpu::BufferUsages,
}

/// Order in which TextureData is laid out in memory.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
pub enum TextureDataOrder {
    /// The texture is laid out densely in memory as:
    ///
    /// ```text
    /// Layer0Mip0 Layer0Mip1 Layer0Mip2
    /// Layer1Mip0 Layer1Mip1 Layer1Mip2
    /// Layer2Mip0 Layer2Mip1 Layer2Mip2
    /// ````
    ///
    /// This is the layout used by dds files.
    ///
    /// This was the previous behavior of [`DeviceExt::create_texture_with_data`].
    #[default]
    LayerMajor,
    /// The texture is laid out densely in memory as:
    ///
    /// ```text
    /// Layer0Mip0 Layer1Mip0 Layer2Mip0
    /// Layer0Mip1 Layer1Mip1 Layer2Mip1
    /// Layer0Mip2 Layer1Mip2 Layer2Mip2
    /// ```
    ///
    /// This is the layout used by ktx and ktx2 files.
    MipMajor,
}

/// Utility methods not meant to be in the main API.
pub trait DeviceExt {
    /// Upload an entire texture and its mipmaps from a source buffer.
    ///
    /// Expects all mipmaps to be tightly packed in the data buffer.
    ///
    /// See [`TextureDataOrder`] for the order in which the data is laid out in memory.
    ///
    /// Implicitly adds the `COPY_DST` usage if it is not present in the descriptor,
    /// as it is required to be able to upload the data to the gpu.
    fn create_texture_with_data(
        &self,
        queue: &wgpu::Queue,
        desc: &wgpu::TextureDescriptor<'_>,
        order: TextureDataOrder,
        data: &[u8],
    ) -> wgpu::Texture;
}

impl DeviceExt for wgpu::Device {
    fn create_texture_with_data(
        &self,
        queue: &wgpu::Queue,
        desc: &wgpu::TextureDescriptor<'_>,
        order: TextureDataOrder,
        data: &[u8],
    ) -> wgpu::Texture {
        // Implicitly add the COPY_DST usage
        let mut desc = desc.to_owned();
        desc.usage |= wgpu::TextureUsages::COPY_DST;
        let texture = self.create_texture(&desc);

        // Will return None only if it's a combined depth-stencil format
        // If so, default to 4, validation will fail later anyway since the depth or stencil
        // aspect needs to be written to individually
        let block_size = desc.format.block_size(None).unwrap_or(4);
        let (block_width, block_height) = desc.format.block_dimensions();
        let layer_iterations = desc.array_layer_count();

        let outer_iteration;
        let inner_iteration;
        match order {
            TextureDataOrder::LayerMajor => {
                outer_iteration = layer_iterations;
                inner_iteration = desc.mip_level_count;
            }
            TextureDataOrder::MipMajor => {
                outer_iteration = desc.mip_level_count;
                inner_iteration = layer_iterations;
            }
        }

        let mut binary_offset = 0;
        for outer in 0..outer_iteration {
            for inner in 0..inner_iteration {
                let (layer, mip) = match order {
                    TextureDataOrder::LayerMajor => (outer, inner),
                    TextureDataOrder::MipMajor => (inner, outer),
                };

                let mut mip_size = desc.mip_level_size(mip).unwrap();
                // copying layers separately
                if desc.dimension != wgpu::TextureDimension::D3 {
                    mip_size.depth_or_array_layers = 1;
                }

                // When uploading mips of compressed textures and the mip is supposed to be
                // a size that isn't a multiple of the block size, the mip needs to be uploaded
                // as its "physical size" which is the size rounded up to the nearest block size.
                let mip_physical = mip_size.physical_size(desc.format);

                // All these calculations are performed on the physical size as that's the
                // data that exists in the buffer.
                let width_blocks = mip_physical.width / block_width;
                let height_blocks = mip_physical.height / block_height;

                let bytes_per_row = width_blocks * block_size;
                let data_size = bytes_per_row * height_blocks * mip_size.depth_or_array_layers;

                let end_offset = binary_offset + data_size as usize;

                queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: &texture,
                        mip_level: mip,
                        origin: wgpu::Origin3d {
                            x: 0,
                            y: 0,
                            z: layer,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    &data[binary_offset..end_offset],
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(bytes_per_row),
                        rows_per_image: Some(height_blocks),
                    },
                    mip_physical,
                );

                binary_offset = end_offset;
            }
        }

        texture
    }
}